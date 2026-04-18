// ai/transposition.rs - Transposition table for caching AI evaluations
use crate::core::{Board, Player};
use crate::ai::evaluation::BoardEvaluator;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use parking_lot::RwLock;
use std::sync::Arc;

/// Transposition table entry
#[derive(Debug, Clone, Copy)]
pub struct TranspositionEntry {
    /// Zobrist hash of the position
    pub hash: u64,
    /// Best move for this position
    pub best_move: Option<usize>,
    /// Score for this position
    pub score: i32,
    /// Depth of the search
    pub depth: u8,
    /// Entry type (exact, lower bound, upper bound)
    pub entry_type: EntryType,
    /// Age of the entry (for replacement)
    pub age: u8,
}

impl TranspositionEntry {
    /// Create a new transposition entry
    pub fn new(
        hash: u64,
        best_move: Option<usize>,
        score: i32,
        depth: u8,
        entry_type: EntryType,
        age: u8,
    ) -> Self {
        Self {
            hash,
            best_move,
            score,
            depth,
            entry_type,
            age,
        }
    }

    /// Check if this entry can replace another
    pub fn should_replace(&self, other: &Self, current_age: u8) -> bool {
        // Prefer entries from current age
        if self.age == current_age && other.age != current_age {
            return true;
        }

        // Prefer deeper entries
        if self.depth > other.depth {
            return true;
        }

        // Prefer exact entries over bounds
        if matches!(self.entry_type, EntryType::Exact) && !matches!(other.entry_type, EntryType::Exact) {
            return true;
        }

        false
    }
}

/// Type of transposition table entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    /// Exact score
    Exact,
    /// Lower bound (score is at least this high)
    LowerBound,
    /// Upper bound (score is at most this high)
    UpperBound,
}

impl EntryType {
    /// Create entry type from alpha-beta bounds
    pub fn from_bounds(score: i32, alpha: i32, beta: i32) -> Self {
        if score <= alpha {
            Self::UpperBound
        } else if score >= beta {
            Self::LowerBound
        } else {
            Self::Exact
        }
    }

    /// Check if this entry type matches search bounds
    pub fn matches_bounds(&self, score: i32, alpha: i32, beta: i32) -> bool {
        match self {
            Self::Exact => score > alpha && score < beta,
            Self::LowerBound => score >= beta,
            Self::UpperBound => score <= alpha,
        }
    }
}

/// Transposition table configuration
#[derive(Debug, Clone)]
pub struct TranspositionConfig {
    /// Maximum number of entries in the table
    pub max_entries: usize,
    /// Enable age-based replacement
    pub enable_aging: bool,
    /// Enable replacement strategy
    pub enable_replacement: bool,
    /// Replacement strategy
    pub replacement_strategy: ReplacementStrategy,
}

impl Default for TranspositionConfig {
    fn default() -> Self {
        Self {
            max_entries: 1_000_000,
            enable_aging: true,
            enable_replacement: true,
            replacement_strategy: ReplacementStrategy::DepthPreferred,
        }
    }
}

impl TranspositionConfig {
    /// Create config for small transposition table
    pub fn small() -> Self {
        Self {
            max_entries: 100_000,
            ..Default::default()
        }
    }

    /// Create config for medium transposition table
    pub fn medium() -> Self {
        Self {
            max_entries: 1_000_000,
            ..Default::default()
        }
    }

    /// Create config for large transposition table
    pub fn large() -> Self {
        Self {
            max_entries: 10_000_000,
            ..Default::default()
        }
    }
}

/// Replacement strategy for transposition table
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementStrategy {
    /// Always replace (no strategy)
    AlwaysReplace,
    /// Replace entries with lower depth
    DepthPreferred,
    /// Replace old entries
    AgePreferred,
    /// Two-tier replacement (deep + shallow tables)
    TwoTier,
}

/// Transposition table statistics
#[derive(Debug, Clone, Default)]
pub struct TranspositionStats {
    /// Total lookups performed
    pub lookups: usize,
    /// Successful cache hits
    pub hits: usize,
    /// Cache misses
    pub misses: usize,
    /// Number of entries stored
    pub stores: usize,
    /// Number of replacements
    pub replacements: usize,
    /// Number of hash collisions
    pub collisions: usize,
    /// Current age
    pub current_age: u8,
    /// Hit rate (hits / lookups)
    pub hit_rate: f64,
}

impl TranspositionStats {
    /// Calculate and update hit rate
    pub fn update_hit_rate(&mut self) {
        self.hit_rate = if self.lookups > 0 {
            self.hits as f64 / self.lookups as f64
        } else {
            0.0
        };
    }

    /// Record a lookup
    pub fn record_lookup(&mut self) {
        self.lookups += 1;
    }

    /// Record a hit
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Record a miss
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// Record a store
    pub fn record_store(&mut self) {
        self.stores += 1;
    }

    /// Record a replacement
    pub fn record_replacement(&mut self) {
        self.replacements += 1;
    }

    /// Record a collision
    pub fn record_collision(&mut self) {
        self.collisions += 1;
    }
}

/// Transposition table for caching positions
pub struct TranspositionTable {
    /// Table entries
    entries: Vec<Option<TranspositionEntry>>,
    /// Configuration
    config: TranspositionConfig,
    /// Statistics
    stats: Arc<RwLock<TranspositionStats>>,
    /// Current age for replacement
    current_age: u8,
}

impl TranspositionTable {
    /// Create a new transposition table
    pub fn new(config: TranspositionConfig) -> Self {
        let size = config.max_entries.next_power_of_two();
        Self {
            entries: vec![None; size],
            config,
            stats: Arc::new(RwLock::new(TranspositionStats::default())),
            current_age: 0,
        }
    }

    /// Create with default config
    pub fn default() -> Self {
        Self::new(TranspositionConfig::default())
    }

    /// Look up a position in the transposition table
    pub fn lookup(&self, hash: u64, depth: u8, alpha: i32, beta: i32) -> Option<TranspositionEntry> {
        let mut stats = self.stats.write();
        stats.record_lookup();

        let index = self.index(hash);

        if let Some(entry) = self.entries[index] {
            if entry.hash == hash && entry.depth >= depth {
                // Check if the entry type matches the bounds
                if entry.entry_type.matches_bounds(entry.score, alpha, beta) {
                    stats.record_hit();
                    stats.update_hit_rate();
                    return Some(entry);
                }
            }
        }

        stats.record_miss();
        stats.update_hit_rate();
        None
    }

    /// Store a position in the transposition table
    pub fn store(&mut self, entry: TranspositionEntry) {
        let index = self.index(entry.hash);
        let mut stats = self.stats.write();

        match &mut self.entries[index] {
            Some(existing) => {
                if existing.hash == entry.hash {
                    // Same position - update if better
                    if entry.should_replace(existing, self.current_age) {
                        *existing = entry;
                        stats.record_replacement();
                    }
                } else {
                    // Hash collision - check replacement strategy
                    if self.config.enable_replacement {
                        if entry.should_replace(existing, self.current_age) {
                            *existing = entry;
                            stats.record_collision();
                            stats.record_replacement();
                        }
                    }
                }
            }
            None => {
                // Empty slot - store entry
                self.entries[index] = Some(entry);
                stats.record_store();
            }
        }
    }

    /// Get index for hash
    fn index(&self, hash: u64) -> usize {
        (hash as usize) & (self.entries.len() - 1)
    }

    /// Clear the transposition table
    pub fn clear(&mut self) {
        self.entries = vec![None; self.entries.len()];
        let mut stats = self.stats.write();
        *stats = TranspositionStats::default();
        self.current_age = 0;
    }

    /// Increment age for replacement
    pub fn increment_age(&mut self) {
        if self.config.enable_aging {
            self.current_age = self.current_age.wrapping_add(1);
            self.stats.write().current_age = self.current_age;
        }
    }

    /// Get current age
    pub fn current_age(&self) -> u8 {
        self.current_age
    }

    /// Get statistics
    pub fn stats(&self) -> TranspositionStats {
        let stats = self.stats.read();
        stats.clone()
    }

    /// Get number of entries
    pub fn entry_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_some()).count()
    }

    /// Get load factor
    pub fn load_factor(&self) -> f64 {
        self.entry_count() as f64 / self.entries.len() as f64
    }

    /// Resize the transposition table
    pub fn resize(&mut self, new_size: usize) {
        let size = new_size.next_power_of_two();
        let mut new_entries = vec![None; size];

        for entry in self.entries.drain(..) {
            if let Some(e) = entry {
                let index = (e.hash as usize) & (size - 1);
                new_entries[index] = Some(e);
            }
        }

        self.entries = new_entries;
        self.config.max_entries = new_size;
    }
}

/// Thread-safe transposition table using Arc<RwLock>
#[derive(Clone)]
pub struct SharedTranspositionTable {
    table: Arc<RwLock<TranspositionTable>>,
}

impl SharedTranspositionTable {
    /// Create a new shared transposition table
    pub fn new(config: TranspositionConfig) -> Self {
        Self {
            table: Arc::new(RwLock::new(TranspositionTable::new(config))),
        }
    }

    /// Create with default config
    pub fn default() -> Self {
        Self::new(TranspositionConfig::default())
    }

    /// Look up a position
    pub fn lookup(&self, hash: u64, depth: u8, alpha: i32, beta: i32) -> Option<TranspositionEntry> {
        let table = self.table.read();
        table.lookup(hash, depth, alpha, beta)
    }

    /// Store a position
    pub fn store(&self, entry: TranspositionEntry) {
        let mut table = self.table.write();
        table.store(entry);
    }

    /// Clear the table
    pub fn clear(&self) {
        let mut table = self.table.write();
        table.clear();
    }

    /// Increment age
    pub fn increment_age(&self) {
        let mut table = self.table.write();
        table.increment_age();
    }

    /// Get statistics
    pub fn stats(&self) -> TranspositionStats {
        let table = self.table.read();
        table.stats()
    }

    /// Resize the table
    pub fn resize(&self, new_size: usize) {
        let mut table = self.table.write();
        table.resize(new_size);
    }
}

/// Transposition-aware AI evaluator
pub struct TranspositionAwareEvaluator<E: BoardEvaluator> {
    /// Base evaluator
    evaluator: E,
    /// Transposition table
    table: SharedTranspositionTable,
    /// Zobrist keys for hashing
    zobrist: ZobristKeys,
}

impl<E: BoardEvaluator> TranspositionAwareEvaluator<E> {
    /// Create a new transposition-aware evaluator
    pub fn new(evaluator: E, table: SharedTranspositionTable) -> Self {
        Self {
            evaluator,
            table,
            zobrist: ZobristKeys::new(),
        }
    }

    /// Create with default table
    pub fn with_default_table(evaluator: E) -> Self {
        Self::new(evaluator, SharedTranspositionTable::default())
    }

    /// Evaluate with transposition lookup
    pub fn evaluate(&mut self, board: &Board, player: Player) -> f64 {
        let hash = self.zobrist.hash(board, player);

        // Try to get from transposition table
        if let Some(entry) = self.table.lookup(hash, 0, i32::MIN, i32::MAX) {
            return entry.score as f64;
        }

        // Evaluate using base evaluator
        let score = self.evaluator.evaluate(board, player);

        // Store in transposition table
        let entry = TranspositionEntry::new(
            hash,
            None,
            score as i32,
            0,
            EntryType::Exact,
            self.table.stats().current_age,
        );
        self.table.store(entry);

        score
    }

    /// Get the transposition table
    pub fn table(&self) -> &SharedTranspositionTable {
        &self.table
    }

    /// Get the base evaluator
    pub fn evaluator(&self) -> &E {
        &self.evaluator
    }

    /// Get mutable reference to base evaluator
    pub fn evaluator_mut(&mut self) -> &mut E {
        &mut self.evaluator
    }
}

impl<E: BoardEvaluator> BoardEvaluator for TranspositionAwareEvaluator<E> {
    fn evaluate(&mut self, board: &Board, player: Player) -> f64 {
        self.evaluate(board, player)
    }
}

/// Zobrist hashing keys
pub struct ZobristKeys {
    /// Keys for board positions
    board_keys: Vec<Vec<u64>>,
    /// Key for player to move
    player_keys: [u64; 2],
}

impl ZobristKeys {
    /// Create new random Zobrist keys
    pub fn new() -> Self {
        let mut board_keys = Vec::with_capacity(6);
        for _ in 0..6 {
            let mut row_keys = Vec::with_capacity(7);
            for _ in 0..7 {
                row_keys.push(Self::random_u64());
            }
            board_keys.push(row_keys);
        }

        Self {
            board_keys,
            player_keys: [Self::random_u64(), Self::random_u64()],
        }
    }

    /// Generate random 64-bit number
    fn random_u64() -> u64 {
        use std::collections::hash_map::RandomState;
        let hasher = RandomState::new();
        let mut h = DefaultHasher::new();
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut h);
        h.finish()
    }

    /// Calculate hash for a board position
    pub fn hash(&self, board: &Board, player: Player) -> u64 {
        let mut hash = self.player_keys[player as usize];

        for row in 0..board.rows() {
            for col in 0..board.cols() {
                if let Some(cell) = board.get_opt(row, col) {
                    if !cell.is_empty() {
                        let cell_value = cell as u64;
                        hash ^= self.board_keys[row][col] * (cell_value + 1);
                    }
                }
            }
        }

        hash
    }

    /// Calculate hash incrementally after a move
    pub fn update_hash(&self, hash: u64, row: usize, col: usize, player: Player) -> u64 {
        let player_idx = player as usize;
        hash ^ self.board_keys[row][col] * (player_idx as u64 + 1)
    }
}

/// Transposition table benchmarking
pub struct TranspositionBenchmark {
    table: TranspositionTable,
}

impl TranspositionBenchmark {
    /// Create a new benchmark instance
    pub fn new(config: TranspositionConfig) -> Self {
        Self {
            table: TranspositionTable::new(config),
        }
    }

    /// Run lookup benchmark
    pub fn benchmark_lookup(&mut self, iterations: usize) -> BenchmarkResult {
        let start = std::time::Instant::now();
        let mut lookups = 0;
        let mut hits = 0;

        for i in 0..iterations {
            let hash = i as u64;
            if self.table.lookup(hash, 0, i32::MIN, i32::MAX).is_some() {
                hits += 1;
            }
            lookups += 1;
        }

        let elapsed = start.elapsed();

        BenchmarkResult {
            operation: "lookup".to_string(),
            iterations,
            duration: elapsed,
            throughput: iterations as f64 / elapsed.as_secs_f64(),
            hits,
            lookups,
        }
    }

    /// Run storage benchmark
    pub fn benchmark_storage(&mut self, entries: usize) -> BenchmarkResult {
        let start = std::time::Instant::now();
        let mut stores = 0;

        for i in 0..entries {
            let entry = TranspositionEntry::new(
                i as u64,
                Some(i % 7),
                (i % 100) as i32 - 50,
                (i % 10) as u8,
                EntryType::Exact,
                0,
            );
            self.table.store(entry);
            stores += 1;
        }

        let elapsed = start.elapsed();

        BenchmarkResult {
            operation: "storage".to_string(),
            iterations: entries,
            duration: elapsed,
            throughput: entries as f64 / elapsed.as_secs_f64(),
            hits: 0,
            lookups: 0,
        }
    }

    /// Run mixed benchmark
    pub fn benchmark_mixed(&mut self, iterations: usize) -> BenchmarkResult {
        let start = std::time::Instant::now();
        let mut operations = 0;
        let mut hits = 0;

        for i in 0..iterations {
            if i % 2 == 0 {
                // Lookup
                let hash = (i / 2) as u64;
                if self.table.lookup(hash, 0, i32::MIN, i32::MAX).is_some() {
                    hits += 1;
                }
            } else {
                // Store
                let entry = TranspositionEntry::new(
                    (i / 2) as u64,
                    Some((i / 2) % 7),
                    ((i / 2) % 100) as i32 - 50,
                    ((i / 2) % 10) as u8,
                    EntryType::Exact,
                    0,
                );
                self.table.store(entry);
            }
            operations += 1;
        }

        let elapsed = start.elapsed();

        BenchmarkResult {
            operation: "mixed".to_string(),
            iterations: operations,
            duration: elapsed,
            throughput: operations as f64 / elapsed.as_secs_f64(),
            hits,
            lookups: operations / 2,
        }
    }
}

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Operation performed
    pub operation: String,
    /// Number of iterations
    pub iterations: usize,
    /// Duration of benchmark
    pub duration: std::time::Duration,
    /// Throughput (operations per second)
    pub throughput: f64,
    /// Number of hits
    pub hits: usize,
    /// Number of lookups
    pub lookups: usize,
}

impl BenchmarkResult {
    /// Display benchmark result
    pub fn display(&self) {
        println!("{} Benchmark:", self.operation);
        println!("  Iterations: {}", self.iterations);
        println!("  Duration: {:?}", self.duration);
        println!("  Throughput: {:.2} ops/sec", self.throughput);
        if self.lookups > 0 {
            let hit_rate = (self.hits as f64 / self.lookups as f64) * 100.0;
            println!("  Hit rate: {:.2}%", hit_rate);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transposition_entry_creation() {
        let entry = TranspositionEntry::new(
            12345,
            Some(3),
            100,
            5,
            EntryType::Exact,
            0,
        );

        assert_eq!(entry.hash, 12345);
        assert_eq!(entry.best_move, Some(3));
        assert_eq!(entry.score, 100);
        assert_eq!(entry.depth, 5);
    }

    #[test]
    fn test_entry_type_from_bounds() {
        assert_eq!(
            EntryType::from_bounds(50, 0, 100),
            EntryType::Exact
        );
        assert_eq!(
            EntryType::from_bounds(-100, 0, 100),
            EntryType::UpperBound
        );
        assert_eq!(
            EntryType::from_bounds(200, 0, 100),
            EntryType::LowerBound
        );
    }

    #[test]
    fn test_transposition_config_sizes() {
        let small = TranspositionConfig::small();
        assert_eq!(small.max_entries, 100_000);

        let medium = TranspositionConfig::medium();
        assert_eq!(medium.max_entries, 1_000_000);

        let large = TranspositionConfig::large();
        assert_eq!(large.max_entries, 10_000_000);
    }

    #[test]
    fn test_transposition_table_creation() {
        let table = TranspositionTable::default();
        assert_eq!(table.entry_count(), 0);
        assert_eq!(table.load_factor(), 0.0);
    }

    #[test]
    fn test_transposition_table_store_lookup() {
        let mut table = TranspositionTable::default();

        let entry = TranspositionEntry::new(
            12345,
            Some(3),
            100,
            5,
            EntryType::Exact,
            0,
        );

        table.store(entry);

        let result = table.lookup(12345, 5, i32::MIN, i32::MAX);
        assert!(result.is_some());
        assert_eq!(result.unwrap().best_move, Some(3));
    }

    #[test]
    fn test_transposition_table_clear() {
        let mut table = TranspositionTable::default();

        let entry = TranspositionEntry::new(
            12345,
            Some(3),
            100,
            5,
            EntryType::Exact,
            0,
        );

        table.store(entry);
        assert_eq!(table.entry_count(), 1);

        table.clear();
        assert_eq!(table.entry_count(), 0);
    }

    #[test]
    fn test_transposition_stats() {
        let mut stats = TranspositionStats::default();

        stats.record_lookup();
        stats.record_hit();
        stats.record_lookup();
        stats.record_miss();

        stats.update_hit_rate();

        assert_eq!(stats.lookups, 2);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }

    #[test]
    fn test_zobrist_hash() {
        let zobrist = ZobristKeys::new();

        // Hash should be deterministic for same board
        let hash1 = zobrist.hash(&Board::new(6, 7), Player::Player1);
        let hash2 = zobrist.hash(&Board::new(6, 7), Player::Player1);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_shared_transposition_table() {
        let table = SharedTranspositionTable::default();

        let entry = TranspositionEntry::new(
            12345,
            Some(3),
            100,
            5,
            EntryType::Exact,
            0,
        );

        table.store(entry.clone());

        let result = table.lookup(12345, 5, i32::MIN, i32::MAX);
        assert!(result.is_some());

        let stats = table.stats();
        assert_eq!(stats.lookups, 1);
        assert_eq!(stats.hits, 1);
    }

    #[test]
    fn test_transposition_table_resize() {
        let mut table = TranspositionTable::new(TranspositionConfig::small());

        let entry = TranspositionEntry::new(
            12345,
            Some(3),
            100,
            5,
            EntryType::Exact,
            0,
        );

        table.store(entry);

        table.resize(200_000);
        assert_eq!(table.entry_count(), 1);
    }
}
