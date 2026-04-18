// ai/zobrist.rs - Zobrist hashing for game positions
use crate::core::{Board, Player, Cell};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Zobrist hashing keys for the game
#[derive(Debug, Clone)]
pub struct ZobristKeys {
    /// Keys for each board position and piece type
    position_keys: Vec<Vec<[u64; 2]>>,
    /// Key for player to move
    player_keys: [u64; 2],
    /// Keys for special piece states
    special_keys: [u64; 8],
}

impl ZobristKeys {
    /// Create new Zobrist keys with random values
    pub fn new() -> Self {
        Self {
            position_keys: Self::generate_position_keys(6, 7),
            player_keys: [Self::random_u64(), Self::random_u64()],
            special_keys: Self::generate_special_keys(),
        }
    }

    /// Create Zobrist keys with a specific seed (for reproducibility)
    pub fn with_seed(seed: u64) -> Self {
        Self {
            position_keys: Self::generate_position_keys_seeded(6, 7, seed),
            player_keys: [
                Self::seeded_random(seed),
                Self::seeded_random(seed.wrapping_add(1)),
            ],
            special_keys: Self::generate_special_keys_seeded(seed.wrapping_add(2)),
        }
    }

    /// Generate position keys for all board cells
    fn generate_position_keys(rows: usize, cols: usize) -> Vec<Vec<[u64; 2]>> {
        let mut keys = Vec::with_capacity(rows);
        for _ in 0..rows {
            let mut row_keys = Vec::with_capacity(cols);
            for _ in 0..cols {
                row_keys.push([Self::random_u64(), Self::random_u64()]);
            }
            keys.push(row_keys);
        }
        keys
    }

    /// Generate position keys with a specific seed
    fn generate_position_keys_seeded(rows: usize, cols: usize, seed: u64) -> Vec<Vec<[u64; 2]>> {
        let mut keys = Vec::with_capacity(rows);
        let mut current_seed = seed;

        for _ in 0..rows {
            let mut row_keys = Vec::with_capacity(cols);
            for _ in 0..cols {
                row_keys.push([
                    Self::seeded_random(current_seed),
                    Self::seeded_random(current_seed.wrapping_add(1)),
                ]);
                current_seed = current_seed.wrapping_add(2);
            }
            keys.push(row_keys);
        }

        keys
    }

    /// Generate special piece state keys
    fn generate_special_keys() -> [u64; 8] {
        [
            Self::random_u64(),
            Self::random_u64(),
            Self::random_u64(),
            Self::random_u64(),
            Self::random_u64(),
            Self::random_u64(),
            Self::random_u64(),
            Self::random_u64(),
        ]
    }

    /// Generate special piece state keys with seed
    fn generate_special_keys_seeded(seed: u64) -> [u64; 8] {
        [
            Self::seeded_random(seed),
            Self::seeded_random(seed.wrapping_add(1)),
            Self::seeded_random(seed.wrapping_add(2)),
            Self::seeded_random(seed.wrapping_add(3)),
            Self::seeded_random(seed.wrapping_add(4)),
            Self::seeded_random(seed.wrapping_add(5)),
            Self::seeded_random(seed.wrapping_add(6)),
            Self::seeded_random(seed.wrapping_add(7)),
        ]
    }

    /// Generate a random 64-bit number
    fn random_u64() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut hasher = DefaultHasher::new();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        (nanos as u64).hash(&mut hasher);
        hasher.finish()
    }

    /// Generate a random 64-bit number from a seed
    fn seeded_random(seed: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        hasher.finish()
    }

    /// Calculate Zobrist hash for a board position
    pub fn hash(&self, board: &Board, player: Player) -> u64 {
        let mut hash = self.player_keys[player as usize];

        for row in 0..board.rows() {
            for col in 0..board.cols() {
                if let Some(cell) = board.get_opt(row, col) {
                    if !cell.is_empty() {
                        let player_idx = if cell == Cell::Player1 { 0 } else { 1 };
                        hash ^= self.position_keys[row][col][player_idx];

                        // Add special piece modifier
                        let special_type = self.cell_to_special_type(cell);
                        if special_type > 0 {
                            hash ^= self.special_keys[special_type as usize];
                        }
                    }
                }
            }
        }

        hash
    }

    /// Incrementally update hash after a move
    pub fn update_hash(
        &self,
        hash: u64,
        row: usize,
        col: usize,
        player: Player,
        cell: Cell,
    ) -> u64 {
        let player_idx = if player == Player::Player1 { 0 } else { 1 };
        let mut new_hash = hash ^ self.position_keys[row][col][player_idx];

        // Add special piece modifier
        let special_type = self.cell_to_special_type(cell);
        if special_type > 0 {
            new_hash ^= self.special_keys[special_type as usize];
        }

        new_hash
    }

    /// Incrementally update hash when removing a piece
    pub fn remove_hash(&self, hash: u64, row: usize, col: usize, player: Player, cell: Cell) -> u64 {
        let player_idx = if player == Player::Player1 { 0 } else { 1 };
        let mut new_hash = hash ^ self.position_keys[row][col][player_idx];

        // Remove special piece modifier
        let special_type = self.cell_to_special_type(cell);
        if special_type > 0 {
            new_hash ^= self.special_keys[special_type as usize];
        }

        new_hash
    }

    /// Convert cell type to special type index
    fn cell_to_special_type(&self, cell: Cell) -> u8 {
        match cell {
            Cell::Empty => 0,
            Cell::Player1 | Cell::Player2 => 0,
            Cell::Blocker => 1,
            Cell::Explosive => 2,
            Cell::Wall => 3,
            Cell::Bomb => 4,
            Cell::Wild => 5,
        }
    }

    /// Get position keys (for testing or advanced usage)
    pub fn position_keys(&self) -> &Vec<Vec<[u64; 2]>> {
        &self.position_keys
    }

    /// Get player keys (for testing or advanced usage)
    pub fn player_keys(&self) -> &[u64; 2] {
        &self.player_keys
    }

    /// Get special keys (for testing or advanced usage)
    pub fn special_keys(&self) -> &[u64; 8] {
        &self.special_keys
    }
}

impl Default for ZobristKeys {
    fn default() -> Self {
        Self::new()
    }
}

/// Zobrist hash with additional metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZobristHash {
    /// The actual hash value
    pub hash: u64,
    /// Board dimensions
    pub rows: u8,
    pub cols: u8,
    /// Optional metadata
    pub metadata: u64,
}

impl ZobristHash {
    /// Create a new Zobrist hash
    pub fn new(hash: u64, rows: u8, cols: u8) -> Self {
        Self {
            hash,
            rows,
            cols,
            metadata: 0,
        }
    }

    /// Create with metadata
    pub fn with_metadata(hash: u64, rows: u8, cols: u8, metadata: u64) -> Self {
        Self {
            hash,
            rows,
            cols,
            metadata,
        }
    }

    /// Set metadata
    pub fn set_metadata(&mut self, metadata: u64) {
        self.metadata = metadata;
    }

    /// Get combined hash (including metadata)
    pub fn combined_hash(&self) -> u64 {
        self.hash.wrapping_mul(31).wrapping_add(self.metadata)
    }
}

/// Hash history for detecting threefold repetition
#[derive(Debug, Clone)]
pub struct HashHistory {
    /// History of hashes
    hashes: Vec<ZobristHash>,
    /// Maximum history length
    max_length: usize,
}

impl HashHistory {
    /// Create a new hash history
    pub fn new(max_length: usize) -> Self {
        Self {
            hashes: Vec::with_capacity(max_length),
            max_length,
        }
    }

    /// Create with default length (typical for Connect Four)
    pub fn default() -> Self {
        Self::new(100)
    }

    /// Add a hash to history
    pub fn push(&mut self, hash: ZobristHash) {
        self.hashes.push(hash);

        // Trim if exceeding max length
        if self.hashes.len() > self.max_length {
            self.hashes.remove(0);
        }
    }

    /// Check for repetition (threefold or more)
    pub fn check_repetition(&self, hash: ZobristHash) -> bool {
        self.count_occurrences(hash) >= 3
    }

    /// Count occurrences of a hash in history
    pub fn count_occurrences(&self, hash: ZobristHash) -> usize {
        self.hashes.iter()
            .filter(|h| h.hash == hash.hash)
            .count()
    }

    /// Check for two-fold repetition
    pub fn check_twofold(&self, hash: ZobristHash) -> bool {
        self.count_occurrences(hash) >= 2
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.hashes.clear();
    }

    /// Get length of history
    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }

    /// Get most recent hash
    pub fn last(&self) -> Option<&ZobristHash> {
        self.hashes.last()
    }

    /// Remove last hash (for undo)
    pub fn pop(&mut self) -> Option<ZobristHash> {
        self.hashes.pop()
    }
}

/// Incremental Zobrist hasher for efficient updates
pub struct IncrementalHasher {
    /// Zobrist keys
    keys: ZobristKeys,
    /// Current hash value
    current_hash: u64,
    /// Current player
    current_player: Player,
}

impl IncrementalHasher {
    /// Create a new incremental hasher
    pub fn new(keys: ZobristKeys, initial_board: &Board, player: Player) -> Self {
        let current_hash = keys.hash(initial_board, player);

        Self {
            keys,
            current_hash,
            current_player: player,
        }
    }

    /// Create with default keys
    pub fn with_default_keys(board: &Board, player: Player) -> Self {
        Self::new(ZobristKeys::new(), board, player)
    }

    /// Apply a move to the hash
    pub fn apply_move(&mut self, row: usize, col: usize, player: Player, cell: Cell) {
        self.current_hash = self.keys.update_hash(
            self.current_hash,
            row,
            col,
            player,
            cell,
        );
        self.current_player = player;
    }

    /// Remove a move from the hash (for undo)
    pub fn remove_move(&mut self, row: usize, col: usize, player: Player, cell: Cell) {
        self.current_hash = self.keys.remove_hash(
            self.current_hash,
            row,
            col,
            player,
            cell,
        );
        self.current_player = player;
    }

    /// Switch player
    pub fn switch_player(&mut self) {
        self.current_hash ^= self.keys.player_keys[self.current_player as usize];
        self.current_player = if self.current_player == Player::Player1 {
            Player::Player2
        } else {
            Player::Player1
        };
        self.current_hash ^= self.keys.player_keys[self.current_player as usize];
    }

    /// Get current hash
    pub fn current_hash(&self) -> u64 {
        self.current_hash
    }

    /// Get current player
    pub fn current_player(&self) -> Player {
        self.current_player
    }

    /// Reset to a specific board state
    pub fn reset(&mut self, board: &Board, player: Player) {
        self.current_hash = self.keys.hash(board, player);
        self.current_player = player;
    }

    /// Clone the hasher
    pub fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            current_hash: self.current_hash,
            current_player: self.current_player,
        }
    }
}

/// Zobrist hash verification
pub struct HashVerifier {
    keys: ZobristKeys,
}

impl HashVerifier {
    /// Create a new hash verifier
    pub fn new(keys: ZobristKeys) -> Self {
        Self { keys }
    }

    /// Verify that incremental hash matches full hash
    pub fn verify(
        &self,
        board: &Board,
        player: Player,
        incremental_hash: u64,
    ) -> bool {
        let full_hash = self.keys.hash(board, player);
        incremental_hash == full_hash
    }

    /// Verify a sequence of moves
    pub fn verify_sequence(
        &self,
        initial_board: &Board,
        moves: &[(usize, usize, Player, Cell)],
        final_hash: u64,
    ) -> bool {
        let mut hasher = IncrementalHasher::new(self.keys.clone(), initial_board, Player::Player1);

        for &(row, col, player, cell) in moves {
            hasher.apply_move(row, col, player, cell);
        }

        hasher.current_hash() == final_hash
    }
}

/// Zobrist hash collision detector
#[derive(Debug, Clone)]
pub struct CollisionDetector {
    /// Map of hash to board representation
    hash_map: std::collections::HashMap<u64, Vec<Vec<u8>>>,
    /// Number of collisions detected
    collisions: usize,
    /// Total hashes generated
    total_hashes: usize,
}

impl CollisionDetector {
    /// Create a new collision detector
    pub fn new() -> Self {
        Self {
            hash_map: std::collections::HashMap::new(),
            collisions: 0,
            total_hashes: 0,
        }
    }

    /// Check for collision
    pub fn check_collision(&mut self, hash: u64, board_repr: &[u8]) -> bool {
        self.total_hashes += 1;

        if let Some(existing) = self.hash_map.get(&hash) {
            // Check if it's a true collision (different boards with same hash)
            if existing != board_repr {
                self.collisions += 1;
                return true;
            }
        } else {
            self.hash_map.insert(hash, board_repr.to_vec());
        }

        false
    }

    /// Get collision rate
    pub fn collision_rate(&self) -> f64 {
        if self.total_hashes == 0 {
            0.0
        } else {
            self.collisions as f64 / self.total_hashes as f64
        }
    }

    /// Get number of collisions
    pub fn collisions(&self) -> usize {
        self.collisions
    }

    /// Get total hashes
    pub fn total_hashes(&self) -> usize {
        self.total_hashes
    }

    /// Clear detector
    pub fn clear(&mut self) {
        self.hash_map.clear();
        self.collisions = 0;
        self.total_hashes = 0;
    }
}

impl Default for CollisionDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Board to bytes conversion for collision detection
pub fn board_to_bytes(board: &Board) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(board.rows() * board.cols());

    for row in 0..board.rows() {
        for col in 0..board.cols() {
            if let Some(cell) = board.get_opt(row, col) {
                bytes.push(cell as u8);
            } else {
                bytes.push(0);
            }
        }
    }

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zobrist_keys_creation() {
        let keys = ZobristKeys::new();
        assert_eq!(keys.position_keys.len(), 6);
        assert_eq!(keys.position_keys[0].len(), 7);
    }

    #[test]
    fn test_zobrist_keys_with_seed() {
        let keys1 = ZobristKeys::with_seed(12345);
        let keys2 = ZobristKeys::with_seed(12345);

        assert_eq!(keys1.player_keys, keys2.player_keys);
    }

    #[test]
    fn test_zobrist_hash_deterministic() {
        let keys = ZobristKeys::with_seed(12345);
        let board = Board::new(6, 7);

        let hash1 = keys.hash(&board, Player::Player1);
        let hash2 = keys.hash(&board, Player::Player1);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_zobrist_hash_different_players() {
        let keys = ZobristKeys::new();
        let board = Board::new(6, 7);

        let hash1 = keys.hash(&board, Player::Player1);
        let hash2 = keys.hash(&board, Player::Player2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_incremental_hasher() {
        let keys = ZobristKeys::new();
        let board = Board::new(6, 7);
        let mut hasher = IncrementalHasher::with_default_keys(&board, Player::Player1);

        let initial_hash = hasher.current_hash();

        // Apply a move
        hasher.apply_move(5, 3, Player::Player1, Cell::Player1);

        assert_ne!(hasher.current_hash(), initial_hash);
    }

    #[test]
    fn test_hash_history() {
        let mut history = HashHistory::default();

        let hash = ZobristHash::new(12345, 6, 7);
        history.push(hash);

        assert_eq!(history.len(), 1);
        assert_eq!(history.count_occurrences(hash), 1);
        assert!(!history.check_repetition(hash));
    }

    #[test]
    fn test_hash_history_repetition() {
        let mut history = HashHistory::default();

        let hash = ZobristHash::new(12345, 6, 7);

        history.push(hash);
        history.push(hash);
        history.push(hash);

        assert!(history.check_repetition(hash));
        assert_eq!(history.count_occurrences(hash), 3);
    }

    #[test]
    fn test_hash_history_twofold() {
        let mut history = HashHistory::default();

        let hash = ZobristHash::new(12345, 6, 7);

        history.push(hash);
        history.push(hash);

        assert!(!history.check_repetition(hash));
        assert!(history.check_twofold(hash));
    }

    #[test]
    fn test_zobrist_hash() {
        let hash = ZobristHash::new(12345, 6, 7);

        assert_eq!(hash.hash, 12345);
        assert_eq!(hash.rows, 6);
        assert_eq!(hash.cols, 7);
    }

    #[test]
    fn test_zobrist_hash_metadata() {
        let mut hash = ZobristHash::new(12345, 6, 7);

        hash.set_metadata(67890);

        assert_eq!(hash.metadata, 67890);
    }

    #[test]
    fn test_collision_detector() {
        let mut detector = CollisionDetector::new();

        let hash = 12345;
        let bytes = vec![1, 2, 3, 4, 5];

        assert!(!detector.check_collision(hash, &bytes));
        assert_eq!(detector.collisions(), 0);

        // Same hash and bytes should not trigger collision
        assert!(!detector.check_collision(hash, &bytes));
        assert_eq!(detector.collisions(), 0);

        // Different bytes with same hash should trigger collision
        let different_bytes = vec![5, 4, 3, 2, 1];
        assert!(detector.check_collision(hash, &different_bytes));
        assert_eq!(detector.collisions(), 1);
    }

    #[test]
    fn test_hash_verifier() {
        let keys = ZobristKeys::new();
        let verifier = HashVerifier::new(keys);

        let board = Board::new(6, 7);

        let full_hash = verifier.keys.hash(&board, Player::Player1);

        assert!(verifier.verify(&board, Player::Player1, full_hash));
    }

    #[test]
    fn test_board_to_bytes() {
        let board = Board::new(6, 7);
        let bytes = board_to_bytes(&board);

        assert_eq!(bytes.len(), 42); // 6 * 7
    }
}
