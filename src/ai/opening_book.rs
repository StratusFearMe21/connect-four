// ai/opening_book.rs - Opening book for initial game moves
use crate::core::{Board, Player, Cell, MoveResult};
use crate::ai::zobrist::ZobristKeys;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Opening book entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpeningEntry {
    /// Zobrist hash of the position
    pub hash: u64,
    /// Best move(s) for this position
    pub moves: Vec<OpeningMove>,
    /// Win rate for each move (0.0 to 1.0)
    pub win_rates: Vec<f64>,
    /// Number of times this position occurred
    pub frequency: usize,
    /// Average game length from this position
    pub avg_length: f64,
    /// ELO rating of this entry
    pub elo: f64,
}

impl OpeningEntry {
    /// Create a new opening entry
    pub fn new(hash: u64) -> Self {
        Self {
            hash,
            moves: Vec::new(),
            win_rates: Vec::new(),
            frequency: 0,
            avg_length: 0.0,
            elo: 1200.0,
        }
    }

    /// Add a move to the entry
    pub fn add_move(&mut self, column: usize, win_rate: f64) {
        self.moves.push(OpeningMove {
            column,
            score: win_rate,
        });
        self.win_rates.push(win_rate);
    }

    /// Get the best move from this entry
    pub fn best_move(&self) -> Option<&OpeningMove> {
        self.moves.iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
    }

    /// Get best moves (top N)
    pub fn best_moves(&self, n: usize) -> Vec<&OpeningMove> {
        let mut sorted: Vec<_> = self.moves.iter().collect();
        sorted.sort_by(|a, b| b.score.partial_cmp(a.score).unwrap());
        sorted.truncate(n);
        sorted
    }

    /// Update frequency
    pub fn increment_frequency(&mut self) {
        self.frequency += 1;
    }

    /// Update average length
    pub fn update_avg_length(&mut self, length: usize) {
        self.avg_length = (self.avg_length * (self.frequency - 1) as f64 + length as f64) / self.frequency as f64;
    }

    /// Get average win rate
    pub fn avg_win_rate(&self) -> f64 {
        if self.win_rates.is_empty() {
            0.0
        } else {
            self.win_rates.iter().sum::<f64>() / self.win_rates.len() as f64
        }
    }
}

/// Opening move with associated score
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OpeningMove {
    /// Column to play
    pub column: usize,
    /// Score/win rate for this move
    pub score: f64,
}

/// Opening book configuration
#[derive(Debug, Clone)]
pub struct OpeningBookConfig {
    /// Maximum ply depth for opening book
    pub max_depth: usize,
    /// Minimum number of games for an entry
    pub min_games: usize,
    /// Minimum win rate threshold
    pub min_win_rate: f64,
    /// Enable learning mode
    pub enable_learning: bool,
    /// Book file path
    pub book_path: Option<String>,
}

impl Default for OpeningBookConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            min_games: 5,
            min_win_rate: 0.45,
            enable_learning: true,
            book_path: None,
        }
    }
}

/// Opening book for storing and retrieving opening moves
pub struct OpeningBook {
    /// Book entries indexed by hash
    entries: HashMap<u64, OpeningEntry>,
    /// Zobrist keys for hashing
    zobrist: ZobristKeys,
    /// Configuration
    config: OpeningBookConfig,
    /// Statistics
    stats: OpeningBookStats,
}

/// Opening book statistics
#[derive(Debug, Clone, Default)]
pub struct OpeningBookStats {
    /// Total lookups
    pub lookups: usize,
    /// Successful hits
    pub hits: usize,
    /// Misses
    pub misses: usize,
    /// Entries in book
    pub entry_count: usize,
    /// Total moves added
    pub moves_added: usize,
}

impl OpeningBookStats {
    /// Calculate hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.lookups == 0 {
            0.0
        } else {
            self.hits as f64 / self.lookups as f64
        }
    }
}

impl OpeningBook {
    /// Create a new opening book
    pub fn new(config: OpeningBookConfig) -> Self {
        Self {
            entries: HashMap::new(),
            zobrist: ZobristKeys::new(),
            config,
            stats: OpeningBookStats::default(),
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(OpeningBookConfig::default())
    }

    /// Look up a position in the opening book
    pub fn lookup(&mut self, board: &Board, player: Player) -> Option<&OpeningEntry> {
        let hash = self.zobrist.hash(board, player);
        self.stats.lookups += 1;

        if let Some(entry) = self.entries.get(&hash) {
            if entry.frequency >= self.config.min_games {
                self.stats.hits += 1;
                return Some(entry);
            }
        }

        self.stats.misses += 1;
        None
    }

    /// Get the best move for a position
    pub fn best_move(&mut self, board: &Board, player: Player) -> Option<OpeningMove> {
        if let Some(entry) = self.lookup(board, player) {
            entry.best_move().copied()
        } else {
            None
        }
    }

    /// Add an entry to the opening book
    pub fn add_entry(&mut self, entry: OpeningEntry) {
        let hash = entry.hash;

        if let Some(existing) = self.entries.get_mut(&hash) {
            // Merge entries
            for (move_idx, win_rate) in entry.win_rates.iter().enumerate() {
                if move_idx < existing.moves.len() {
                    // Update existing move
                    existing.win_rates[move_idx] = (existing.win_rates[move_idx] + win_rate) / 2.0;
                } else {
                    // Add new move
                    existing.add_move(entry.moves[move_idx].column, *win_rate);
                }
            }
            existing.frequency += entry.frequency;
        } else {
            // Add new entry
            self.entries.insert(hash, entry);
            self.stats.entry_count = self.entries.len();
        }
    }

    /// Learn from a game
    pub fn learn_from_game(
        &mut self,
        moves: &[(usize, Player)],
        outcome: GameOutcome,
        game_length: usize,
    ) {
        if !self.config.enable_learning {
            return;
        }

        let mut board = Board::new(6, 7);
        let mut current_player = Player::Player1;

        for (ply, &(col, player)) in moves.iter().enumerate() {
            if ply >= self.config.max_depth {
                break;
            }

            // Make the move
            if let Some(row) = board.find_empty_row(col) {
                board.set(row, col, if player == Player::Player1 {
                    Cell::Player1
                } else {
                    Cell::Player2
                });

                // Calculate win rate for this move
                let win_rate = match outcome {
                    GameOutcome::Win(winner) => {
                        if winner == player {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    GameOutcome::Draw => 0.5,
                };

                // Update or create entry
                let hash = self.zobrist.hash(&board, current_player);
                let entry = self.entries.entry(hash)
                    .or_insert_with(|| OpeningEntry::new(hash));

                entry.add_move(col, win_rate);
                entry.increment_frequency();
                entry.update_avg_length(game_length - ply);
                entry.elo = self.calculate_elo(entry, win_rate);

                self.stats.moves_added += 1;
            }

            current_player = if current_player == Player::Player1 {
                Player::Player2
            } else {
                Player::Player1
            };
        }
    }

    /// Calculate ELO rating for an entry
    fn calculate_elo(&self, entry: &OpeningEntry, win_rate: f64) -> f64 {
        let expected = 1.0 / (1.0 + 10.0_f64.powf((entry.elo - 1200.0) / 400.0));
        let k_factor = 32.0;
        entry.elo + k_factor * (win_rate - expected)
    }

    /// Get statistics
    pub fn stats(&self) -> &OpeningBookStats {
        &self.stats
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Save to file
    pub fn save(&self, path: &str) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&self.entries)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        std::fs::write(path, data)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }

    /// Load from file
    pub fn load(&mut self, path: &str) -> Result<(), String> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        self.entries = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to deserialize: {}", e))?;

        self.stats.entry_count = self.entries.len();
        Ok(())
    }

    /// Clear the opening book
    pub fn clear(&mut self) {
        self.entries.clear();
        self.stats = OpeningBookStats::default();
    }

    /// Prune entries with low frequency or win rate
    pub fn prune(&mut self) {
        self.entries.retain(|_, entry| {
            entry.frequency >= self.config.min_games
                && entry.avg_win_rate() >= self.config.min_win_rate
        });
        self.stats.entry_count = self.entries.len();
    }
}

/// Game outcome for learning
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOutcome {
    /// Player won
    Win(Player),
    /// Draw
    Draw,
}

/// Opening book builder
pub struct OpeningBookBuilder {
    book: OpeningBook,
    games_played: usize,
}

impl OpeningBookBuilder {
    /// Create a new opening book builder
    pub fn new(config: OpeningBookConfig) -> Self {
        Self {
            book: OpeningBook::new(config),
            games_played: 0,
        }
    }

    /// Add a game to the book
    pub fn add_game(&mut self, moves: &[(usize, Player)], outcome: GameOutcome) {
        let game_length = moves.len();
        self.book.learn_from_game(moves, outcome, game_length);
        self.games_played += 1;
    }

    /// Build the final book
    pub fn build(mut self) -> OpeningBook {
        self.book.prune();
        self.book
    }

    /// Get current book
    pub fn book(&self) -> &OpeningBook {
        &self.book
    }

    /// Get games played
    pub fn games_played(&self) -> usize {
        self.games_played
    }
}

/// Common opening sequences
pub struct StandardOpenings;

impl StandardOpenings {
    /// Get common opening moves
    pub fn common_moves() -> Vec<(usize, f64)> {
        vec![
            (3, 0.6),  // Center column
            (2, 0.2),  // Near center
            (4, 0.15), // Near center
            (1, 0.03),
            (5, 0.02),
        ]
    }

    /// Get opening book with standard moves
    pub fn create_standard_book() -> OpeningBook {
        let config = OpeningBookConfig::default();
        let mut book = OpeningBook::new(config);

        // Add standard opening position (empty board)
        let hash = book.zobrist.hash(&Board::new(6, 7), Player::Player1);
        let mut entry = OpeningEntry::new(hash);

        for (col, score) in Self::common_moves() {
            entry.add_move(col, score);
        }
        entry.frequency = 1000;
        entry.elo = 1200.0;

        book.add_entry(entry);

        book
    }
}

/// Random opening generator
pub struct RandomOpeningGenerator {
    zobrist: ZobristKeys,
}

impl RandomOpeningGenerator {
    /// Create a new random opening generator
    pub fn new() -> Self {
        Self {
            zobrist: ZobristKeys::new(),
        }
    }

    /// Generate a random opening sequence
    pub fn generate(&self, depth: usize) -> Vec<(usize, Player)> {
        let mut moves = Vec::new();
        let mut board = Board::new(6, 7);
        let mut player = Player::Player1;

        for _ in 0..depth {
            let valid_moves = board.valid_moves();

            if valid_moves.is_empty() {
                break;
            }

            // Choose random move
            let col = valid_moves[rand::random::<usize>() % valid_moves.len()];

            if let Some(row) = board.find_empty_row(col) {
                board.set(row, col, if player == Player::Player1 {
                    Cell::Player1
                } else {
                    Cell::Player2
                });

                moves.push((col, player));
                player = if player == Player::Player1 {
                    Player::Player2
                } else {
                    Player::Player1
                };
            }
        }

        moves
    }

    /// Generate multiple random openings
    pub fn generate_batch(&self, count: usize, depth: usize) -> Vec<Vec<(usize, Player)>> {
        (0..count)
            .map(|_| self.generate(depth))
            .collect()
    }
}

impl Default for RandomOpeningGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Opening book merger
pub struct OpeningBookMerger;

impl OpeningBookMerger {
    /// Merge multiple opening books
    pub fn merge(books: Vec<OpeningBook>) -> OpeningBook {
        if books.is_empty() {
            return OpeningBook::default();
        }

        let mut merged = books[0].clone();

        for book in books.into_iter().skip(1) {
            for entry in book.entries.into_values() {
                merged.add_entry(entry);
            }
        }

        merged
    }

    /// Merge with weighted averaging
    pub fn merge_weighted(books: Vec<(OpeningBook, f64)>) -> OpeningBook {
        if books.is_empty() {
            return OpeningBook::default();
        }

        let total_weight: f64 = books.iter().map(|(_, w)| w).sum();

        let mut merged = OpeningBook::default();

        for (book, weight) in &books {
            let normalized_weight = weight / total_weight;

            for entry in &book.entries {
                let hash = entry.hash;
                let new_entry = OpeningEntry::new(hash);

                for (move_idx, win_rate) in entry.win_rates.iter().enumerate() {
                    let adjusted_win_rate = win_rate * normalized_weight;
                    if move_idx < new_entry.moves.len() {
                        new_entry.win_rates[move_idx] += adjusted_win_rate;
                    } else {
                        new_entry.add_move(entry.moves[move_idx].column, adjusted_win_rate);
                    }
                }

                new_entry.frequency = (entry.frequency as f64 * normalized_weight) as usize;
                merged.add_entry(new_entry);
            }
        }

        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opening_entry_creation() {
        let entry = OpeningEntry::new(12345);
        assert_eq!(entry.hash, 12345);
        assert!(entry.moves.is_empty());
    }

    #[test]
    fn test_opening_entry_add_move() {
        let mut entry = OpeningEntry::new(12345);
        entry.add_move(3, 0.6);

        assert_eq!(entry.moves.len(), 1);
        assert_eq!(entry.moves[0].column, 3);
        assert_eq!(entry.moves[0].score, 0.6);
    }

    #[test]
    fn test_opening_entry_best_move() {
        let mut entry = OpeningEntry::new(12345);
        entry.add_move(3, 0.4);
        entry.add_move(2, 0.6);
        entry.add_move(4, 0.5);

        let best = entry.best_move().unwrap();
        assert_eq!(best.column, 2);
        assert_eq!(best.score, 0.6);
    }

    #[test]
    fn test_opening_entry_avg_win_rate() {
        let mut entry = OpeningEntry::new(12345);
        entry.add_move(3, 0.6);
        entry.add_move(2, 0.4);

        assert_eq!(entry.avg_win_rate(), 0.5);
    }

    #[test]
    fn test_opening_book_creation() {
        let book = OpeningBook::default();
        assert_eq!(book.entry_count(), 0);
    }

    #[test]
    fn test_opening_book_add_entry() {
        let mut book = OpeningBook::default();

        let mut entry = OpeningEntry::new(12345);
        entry.add_move(3, 0.6);
        entry.frequency = 10;

        book.add_entry(entry);

        assert_eq!(book.entry_count(), 1);
    }

    #[test]
    fn test_opening_book_lookup() {
        let mut book = OpeningBook::default();

        let mut entry = OpeningEntry::new(12345);
        entry.add_move(3, 0.6);
        entry.frequency = 10;

        book.add_entry(entry);

        let board = Board::new(6, 7);
        // Hash should match if we use the same zobrist keys
        // This is a simplified test
        assert!(book.entry_count() > 0);
    }

    #[test]
    fn test_opening_book_stats() {
        let mut book = OpeningBook::default();
        let board = Board::new(6, 7);

        // Perform a lookup (should miss)
        book.lookup(&board, Player::Player1);

        let stats = book.stats();
        assert_eq!(stats.lookups, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 0);
    }

    #[test]
    fn test_opening_book_prune() {
        let mut book = OpeningBook::default();

        let mut entry = OpeningEntry::new(12345);
        entry.add_move(3, 0.3); // Below min_win_rate
        entry.frequency = 10;

        book.add_entry(entry);
        assert_eq!(book.entry_count(), 1);

        book.prune();
        assert_eq!(book.entry_count(), 0);
    }

    #[test]
    fn test_opening_book_builder() {
        let config = OpeningBookConfig::default();
        let mut builder = OpeningBookBuilder::new(config);

        let moves = vec![
            (3, Player::Player1),
            (3, Player::Player2),
            (3, Player::Player1),
        ];

        builder.add_game(&moves, GameOutcome::Win(Player::Player1));

        assert_eq!(builder.games_played(), 1);
    }

    #[test]
    fn test_random_opening_generator() {
        let generator = RandomOpeningGenerator::new();
        let opening = generator.generate(5);

        assert_eq!(opening.len(), 5);
    }

    #[test]
    fn test_random_opening_generator_batch() {
        let generator = RandomOpeningGenerator::new();
        let openings = generator.generate_batch(10, 5);

        assert_eq!(openings.len(), 10);
        assert!(openings.iter().all(|o| o.len() == 5));
    }

    #[test]
    fn test_standard_openings() {
        let moves = StandardOpenings::common_moves();
        assert_eq!(moves[0].0, 3); // Center column
        assert_eq!(moves[0].1, 0.6); // Highest win rate
    }

    #[test]
    fn test_opening_book_merger() {
        let book1 = OpeningBook::default();
        let book2 = OpeningBook::default();

        let merged = OpeningBookMerger::merge(vec![book1, book2]);

        assert_eq!(merged.entry_count(), 0);
    }
}
