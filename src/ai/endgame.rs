// ai/endgame.rs - Endgame solver for perfect play in critical positions
use crate::core::{Board, Player, Cell};
use crate::ai::zobrist::ZobristKeys;
use std::collections::HashMap;
use std::sync::Arc;

/// Endgame result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndgameResult {
    /// Win for the player
    Win,
    /// Loss for the player
    Loss,
    /// Draw
    Draw,
}

impl EndgameResult {
    /// From game outcome
    pub fn from_outcome(outcome: Option<Player>, player: Player) -> Self {
        match outcome {
            Some(winner) if winner == player => Self::Win,
            Some(_) => Self::Loss,
            None => Self::Draw,
        }
    }

    /// Negate the result (for opponent)
    pub fn negate(&self) -> Self {
        match self {
            Self::Win => Self::Loss,
            Self::Loss => Self::Win,
            Self::Draw => Self::Draw,
        }
    }

    /// Get numeric value for comparison
    pub fn value(&self) -> i32 {
        match self {
            Self::Win => 1,
            Self::Draw => 0,
            Self::Loss => -1,
        }
    }
}

/// Endgame configuration
#[derive(Debug, Clone)]
pub struct EndgameConfig {
    /// Maximum depth for search
    pub max_depth: usize,
    /// Enable iterative deepening
    pub iterative_deepening: bool,
    /// Enable move ordering
    pub move_ordering: bool,
    /// Enable transposition table
    pub enable_transposition: bool,
    /// Timeout in milliseconds (None = no timeout)
    pub timeout_ms: Option<u64>,
    /// Number of pieces threshold for endgame mode
    pub piece_threshold: usize,
}

impl Default for EndgameConfig {
    fn default() -> Self {
        Self {
            max_depth: 42,
            iterative_deepening: true,
            move_ordering: true,
            enable_transposition: true,
            timeout_ms: None,
            piece_threshold: 8,
        }
    }
}

impl EndgameConfig {
    /// Quick config (low depth)
    pub fn quick() -> Self {
        Self {
            max_depth: 20,
            ..Default::default()
        }
    }

    /// Deep config (full depth)
    pub fn deep() -> Self {
        Self {
            max_depth: 42,
            ..Default::default()
        }
    }

    /// Tournament config (balanced)
    pub fn tournament() -> Self {
        Self {
            max_depth: 30,
            timeout_ms: Some(5000),
            ..Default::default()
        }
    }
}

/// Endgame solver statistics
#[derive(Debug, Clone, Default)]
pub struct EndgameStats {
    /// Positions evaluated
    pub positions_evaluated: usize,
    /// Cutoffs from alpha-beta
    pub cutoffs: usize,
    /// Transposition hits
    pub transposition_hits: usize,
    /// Total search time (milliseconds)
    pub search_time_ms: u64,
    /// Depth reached
    pub depth_reached: usize,
    /// Best move found
    pub best_move: Option<usize>,
}

/// Endgame solver for perfect play
pub struct EndgameSolver {
    /// Zobrist keys for hashing
    zobrist: ZobristKeys,
    /// Configuration
    config: EndgameConfig,
    /// Statistics
    stats: EndgameStats,
    /// Transposition table
    transposition: HashMap<u64, EndgameEntry>,
    /// Cache for solved positions
    position_cache: HashMap<u64, EndgameResult>,
    /// Timeout check
    start_time: std::time::Instant,
}

/// Endgame transposition table entry
#[derive(Debug, Clone, Copy)]
struct EndgameEntry {
    /// Result for this position
    result: EndgameResult,
    /// Depth of search
    depth: usize,
    /// Best move from this position
    best_move: Option<usize>,
}

impl EndgameSolver {
    /// Create a new endgame solver
    pub fn new(config: EndgameConfig) -> Self {
        Self {
            zobrist: ZobristKeys::new(),
            config,
            stats: EndgameStats::default(),
            transposition: HashMap::new(),
            position_cache: HashMap::new(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(EndgameConfig::default())
    }

    /// Solve a position to find the best move
    pub fn solve(&mut self, board: &Board, player: Player) -> (Option<usize>, EndgameResult) {
        self.start_time = std::time::Instant::now();
        self.stats = EndgameStats::default();

        // Check if this is an endgame position
        if !self.is_endgame_position(board) {
            return (None, EndgameResult::Draw);
        }

        let result = if self.config.iterative_deepening {
            self.solve_iterative_deepening(board, player)
        } else {
            self.solve_full(board, player)
        };

        self.stats.search_time_ms = self.start_time.elapsed().as_millis() as u64;

        (self.stats.best_move, result)
    }

    /// Check if this is an endgame position
    fn is_endgame_position(&self, board: &Board) -> bool {
        let piece_count = board.rows() * board.cols() - board.empty_count();
        piece_count >= self.config.piece_threshold
    }

    /// Solve with iterative deepening
    fn solve_iterative_deepening(&mut self, board: &Board, player: Player) -> EndgameResult {
        let mut best_result = EndgameResult::Draw;

        for depth in 1..=self.config.max_depth {
            if self.check_timeout() {
                break;
            }

            let (best_move, result) = self.solve_depth(board, player, depth);
            self.stats.depth_reached = depth;

            if best_move.is_some() {
                self.stats.best_move = best_move;
            }

            best_result = result;

            // Early termination if winning move found
            if result == EndgameResult::Win {
                break;
            }
        }

        best_result
    }

    /// Solve at a specific depth
    fn solve_depth(
        &mut self,
        board: &Board,
        player: Player,
        depth: usize,
    ) -> (Option<usize>, EndgameResult) {
        let mut best_move = None;
        let mut best_result = EndgameResult::Loss;

        // Get ordered moves
        let mut moves = self.get_ordered_moves(board);

        for &col in &moves {
            if self.check_timeout() {
                break;
            }

            // Make move
            if let Some(row) = board.find_empty_row(col) {
                let mut new_board = board.clone();
                new_board.set(row, col, if player == Player::Player1 {
                    Cell::Player1
                } else {
                    Cell::Player2
                });

                // Check for immediate win
                if new_board.check_win(player) {
                    return (Some(col), EndgameResult::Win);
                }

                // Recursive search
                let opponent = if player == Player::Player1 {
                    Player::Player2
                } else {
                    Player::Player1
                };

                let result = self.search(&new_board, opponent, depth - 1);

                // Update best move
                if result == EndgameResult::Loss {
                    // Found winning move
                    return (Some(col), EndgameResult::Win);
                } else if result == EndgameResult::Draw {
                    best_result = EndgameResult::Draw;
                    best_move = Some(col);
                }

                if best_result == EndgameResult::Loss && best_move.is_none() {
                    best_move = Some(col);
                }
            }
        }

        (best_move, best_result)
    }

    /// Search for best result
    fn search(&mut self, board: &Board, player: Player, depth: usize) -> EndgameResult {
        self.stats.positions_evaluated += 1;

        // Check timeout
        if self.check_timeout() {
            return EndgameResult::Draw;
        }

        // Check transposition table
        if self.config.enable_transposition {
            let hash = self.zobrist.hash(board, player);
            if let Some(entry) = self.transposition.get(&hash) {
                if entry.depth >= depth {
                    self.stats.transposition_hits += 1;
                    return entry.result;
                }
            }
        }

        // Terminal conditions
        if board.check_win(Player::Player1) {
            let result = if player == Player::Player1 {
                EndgameResult::Win
            } else {
                EndgameResult::Loss
            };
            return result;
        }

        if board.check_win(Player::Player2) {
            let result = if player == Player::Player2 {
                EndgameResult::Win
            } else {
                EndgameResult::Loss
            };
            return result;
        }

        if board.is_full() || depth == 0 {
            return EndgameResult::Draw;
        }

        // Recursive search
        let opponent = if player == Player::Player1 {
            Player::Player2
        } else {
            Player::Player1
        };

        let mut best_result = EndgameResult::Loss;
        let mut found_winning_move = false;

        for &col in &self.get_ordered_moves(board) {
            if let Some(row) = board.find_empty_row(col) {
                let mut new_board = board.clone();
                new_board.set(row, col, if player == Player::Player1 {
                    Cell::Player1
                } else {
                    Cell::Player2
                });

                let result = self.search(&new_board, opponent, depth - 1);

                if result == EndgameResult::Loss {
                    // Found winning move for current player
                    best_result = EndgameResult::Win;
                    found_winning_move = true;
                    self.stats.cutoffs += 1;
                    break;
                } else if result == EndgameResult::Draw && !found_winning_move {
                    best_result = EndgameResult::Draw;
                }
            }
        }

        // Store in transposition table
        if self.config.enable_transposition {
            let hash = self.zobrist.hash(board, player);
            self.transposition.insert(hash, EndgameEntry {
                result: best_result,
                depth,
                best_move: None,
            });
        }

        best_result
    }

    /// Solve without iterative deepening
    fn solve_full(&mut self, board: &Board, player: Player) -> EndgameResult {
        self.search(board, player, self.config.max_depth)
    }

    /// Get ordered moves for better pruning
    fn get_ordered_moves(&self, board: &Board) -> Vec<usize> {
        let mut moves: Vec<usize> = board.valid_moves();

        if self.config.move_ordering {
            // Order by center preference
            moves.sort_by(|&a, &b| {
                let center = board.cols() / 2;
                let dist_a = (a as i32 - center as i32).abs();
                let dist_b = (b as i32 - center as i32).abs();
                dist_a.cmp(&dist_b)
            });
        }

        moves
    }

    /// Check for timeout
    fn check_timeout(&self) -> bool {
        if let Some(timeout_ms) = self.config.timeout_ms {
            self.start_time.elapsed().as_millis() as u64 >= timeout_ms
        } else {
            false
        }
    }

    /// Get statistics
    pub fn stats(&self) -> &EndgameStats {
        &self.stats
    }

    /// Clear caches
    pub fn clear_caches(&mut self) {
        self.transposition.clear();
        self.position_cache.clear();
    }
}

/// Perfect play calculator for small positions
pub struct PerfectPlayCalculator {
    solver: EndgameSolver,
}

impl PerfectPlayCalculator {
    /// Create a new perfect play calculator
    pub fn new(config: EndgameConfig) -> Self {
        Self {
            solver: EndgameSolver::new(config),
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(EndgameConfig::default())
    }

    /// Calculate perfect play result
    pub fn calculate(&mut self, board: &Board, player: Player) -> PerfectPlayAnalysis {
        let (best_move, result) = self.solver.solve(board, player);

        PerfectPlayAnalysis {
            best_move,
            result,
            positions_evaluated: self.solver.stats().positions_evaluated,
            search_time_ms: self.solver.stats().search_time_ms,
            depth_reached: self.solver.stats().depth_reached,
        }
    }

    /// Calculate outcome for all moves
    pub fn calculate_all_moves(&mut self, board: &Board, player: Player) -> Vec<MoveOutcome> {
        let mut outcomes = Vec::new();

        for col in 0..board.cols() {
            if let Some(row) = board.find_empty_row(col) {
                let mut new_board = board.clone();
                new_board.set(row, col, if player == Player::Player1 {
                    Cell::Player1
                } else {
                    Cell::Player2
                });

                let opponent = if player == Player::Player1 {
                    Player::Player2
                } else {
                    Player::Player1
                };

                let (_, result) = self.solver.solve(&new_board, opponent);

                outcomes.push(MoveOutcome {
                    column: col,
                    result: result.negate(),
                });
            }
        }

        outcomes
    }
}

/// Perfect play analysis result
#[derive(Debug, Clone)]
pub struct PerfectPlayAnalysis {
    /// Best move to make
    pub best_move: Option<usize>,
    /// Result of perfect play
    pub result: EndgameResult,
    /// Number of positions evaluated
    pub positions_evaluated: usize,
    /// Search time in milliseconds
    pub search_time_ms: u64,
    /// Depth reached
    pub depth_reached: usize,
}

/// Outcome for a specific move
#[derive(Debug, Clone, Copy)]
pub struct MoveOutcome {
    /// Column played
    pub column: usize,
    /// Result of this move
    pub result: EndgameResult,
}

/// Endgame database for pre-computed positions
pub struct EndgameDatabase {
    /// Stored positions
    positions: HashMap<u64, EndgameResult>,
    /// Database statistics
    stats: DatabaseStats,
}

/// Database statistics
#[derive(Debug, Clone, Default)]
pub struct DatabaseStats {
    /// Total positions
    pub total_positions: usize,
    /// Winning positions
    pub winning_positions: usize,
    /// Losing positions
    pub losing_positions: usize,
    /// Drawing positions
    pub drawing_positions: usize,
}

impl EndgameDatabase {
    /// Create a new endgame database
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            stats: DatabaseStats::default(),
        }
    }

    /// Add a position to the database
    pub fn add_position(&mut self, hash: u64, result: EndgameResult) {
        self.positions.insert(hash, result);

        match result {
            EndgameResult::Win => self.stats.winning_positions += 1,
            EndgameResult::Loss => self.stats.losing_positions += 1,
            EndgameResult::Draw => self.stats.drawing_positions += 1,
        }
        self.stats.total_positions += 1;
    }

    /// Look up a position
    pub fn lookup(&self, hash: u64) -> Option<EndgameResult> {
        self.positions.get(&hash).copied()
    }

    /// Get statistics
    pub fn stats(&self) -> &DatabaseStats {
        &self.stats
    }

    /// Clear the database
    pub fn clear(&mut self) {
        self.positions.clear();
        self.stats = DatabaseStats::default();
    }

    /// Save to file
    pub fn save(&self, path: &str) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&self.positions)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        std::fs::write(path, data)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }

    /// Load from file
    pub fn load(&mut self, path: &str) -> Result<(), String> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        self.positions = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to deserialize: {}", e))?;

        self.stats.total_positions = self.positions.len();
        self.stats.winning_positions = self.positions.values().filter(|&&r| r == EndgameResult::Win).count();
        self.stats.losing_positions = self.positions.values().filter(|&&r| r == EndgameResult::Loss).count();
        self.stats.drawing_positions = self.positions.values().filter(|&&r| r == EndgameResult::Draw).count();

        Ok(())
    }
}

impl Default for EndgameDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Endgame database builder
pub struct EndgameDatabaseBuilder {
    solver: EndgameSolver,
    database: EndgameDatabase,
    zobrist: ZobristKeys,
}

impl EndgameDatabaseBuilder {
    /// Create a new endgame database builder
    pub fn new(config: EndgameConfig) -> Self {
        Self {
            solver: EndgameSolver::new(config),
            database: EndgameDatabase::new(),
            zobrist: ZobristKeys::new(),
        }
    }

    /// Build database for a specific piece count
    pub fn build(&mut self, max_pieces: usize) -> &EndgameDatabase {
        let board = Board::new(6, 7);
        self.generate_positions(&board, Player::Player1, max_pieces, 0);
        &self.database
    }

    /// Generate positions recursively
    fn generate_positions(
        &mut self,
        board: &Board,
        player: Player,
        max_pieces: usize,
        current_pieces: usize,
    ) {
        if current_pieces >= max_pieces {
            // Solve this position
            let (_, result) = self.solver.solve(board, player);
            let hash = self.zobrist.hash(board, player);
            self.database.add_position(hash, result);
            return;
        }

        // Generate all possible moves
        for col in 0..board.cols() {
            if let Some(row) = board.find_empty_row(col) {
                let mut new_board = board.clone();
                new_board.set(row, col, if player == Player::Player1 {
                    Cell::Player1
                } else {
                    Cell::Player2
                });

                let opponent = if player == Player::Player1 {
                    Player::Player2
                } else {
                    Player::Player1
                };

                self.generate_positions(&new_board, opponent, max_pieces, current_pieces + 1);
            }
        }
    }

    /// Get the database
    pub fn database(&self) -> &EndgameDatabase {
        &self.database
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endgame_result_from_outcome() {
        assert_eq!(
            EndgameResult::from_outcome(Some(Player::Player1), Player::Player1),
            EndgameResult::Win
        );
        assert_eq!(
            EndgameResult::from_outcome(Some(Player::Player2), Player::Player1),
            EndgameResult::Loss
        );
        assert_eq!(
            EndgameResult::from_outcome(None, Player::Player1),
            EndgameResult::Draw
        );
    }

    #[test]
    fn test_endgame_result_negate() {
        assert_eq!(EndgameResult::Win.negate(), EndgameResult::Loss);
        assert_eq!(EndgameResult::Loss.negate(), EndgameResult::Win);
        assert_eq!(EndgameResult::Draw.negate(), EndgameResult::Draw);
    }

    #[test]
    fn test_endgame_config() {
        let quick = EndgameConfig::quick();
        assert_eq!(quick.max_depth, 20);

        let deep = EndgameConfig::deep();
        assert_eq!(deep.max_depth, 42);

        let tournament = EndgameConfig::tournament();
        assert_eq!(tournament.max_depth, 30);
    }

    #[test]
    fn test_endgame_solver_creation() {
        let solver = EndgameSolver::default();
        assert_eq!(solver.stats.positions_evaluated, 0);
    }

    #[test]
    fn test_endgame_database() {
        let mut db = EndgameDatabase::new();

        db.add_position(12345, EndgameResult::Win);

        assert_eq!(db.stats.total_positions, 1);
        assert_eq!(db.stats.winning_positions, 1);

        let result = db.lookup(12345);
        assert_eq!(result, Some(EndgameResult::Win));
    }

    #[test]
    fn test_perfect_play_calculator() {
        let calc = PerfectPlayCalculator::default();
        let board = Board::new(6, 7);

        let analysis = calc.calculate(&board, Player::Player1);

        assert!(analysis.best_move.is_some() || analysis.result == EndgameResult::Draw);
    }

    #[test]
    fn test_endgame_result_value() {
        assert_eq!(EndgameResult::Win.value(), 1);
        assert_eq!(EndgameResult::Draw.value(), 0);
        assert_eq!(EndgameResult::Loss.value(), -1);
    }
}
