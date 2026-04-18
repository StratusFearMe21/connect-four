// ai/ai_player.rs - AI player implementation combining all AI strategies
use crate::core::{Board, Player, Cell, MoveResult};
use crate::ai::minimax::MinimaxAI;
use crate::ai::alphabeta::AlphaBetaAI;
use crate::ai::mcts::MCTSAI;
use crate::ai::strategy::{AIStrategy, DifficultyLevel};
use crate::ai::evaluation::BoardEvaluator;
use crate::ai::heuristic::HeuristicEvaluator;
use crate::ai::transposition::SharedTranspositionTable;
use crate::ai::opening_book::OpeningBook;
use crate::ai::time_management::TimeManager;
use std::time::Duration;
use std::sync::Arc;

/// AI player configuration
#[derive(Debug, Clone)]
pub struct AIPlayerConfig {
    /// Player name
    pub name: String,
    /// Strategy to use
    pub strategy: AIStrategy,
    /// Difficulty level
    pub difficulty: DifficultyLevel,
    /// Search depth
    pub depth: usize,
    /// Time limit per move (None = unlimited)
    pub time_limit: Option<Duration>,
    /// Use opening book
    pub use_opening_book: bool,
    /// Use transposition table
    pub use_transposition: bool,
    /// Enable pondering (think during opponent's turn)
    pub enable_pondering: bool,
    /// Show thinking output
    pub show_thinking: bool,
}

impl Default for AIPlayerConfig {
    fn default() -> Self {
        Self {
            name: "AI Player".to_string(),
            strategy: AIStrategy::Minimax,
            difficulty: DifficultyLevel::Medium,
            depth: 6,
            time_limit: Some(Duration::from_secs(5)),
            use_opening_book: true,
            use_transposition: true,
            enable_pondering: false,
            show_thinking: false,
        }
    }
}

impl AIPlayerConfig {
    /// Create config for easy difficulty
    pub fn easy() -> Self {
        Self {
            difficulty: DifficultyLevel::Easy,
            depth: 3,
            strategy: AIStrategy::Random,
            ..Default::default()
        }
    }

    /// Create config for medium difficulty
    pub fn medium() -> Self {
        Self {
            difficulty: DifficultyLevel::Medium,
            depth: 6,
            strategy: AIStrategy::AlphaBeta,
            ..Default::default()
        }
    }

    /// Create config for hard difficulty
    pub fn hard() -> Self {
        Self {
            difficulty: DifficultyLevel::Hard,
            depth: 10,
            strategy: AIStrategy::Minimax,
            ..Default::default()
        }
    }

    /// Create config for expert difficulty
    pub fn expert() -> Self {
        Self {
            difficulty: DifficultyLevel::Expert,
            depth: 14,
            strategy: AIStrategy::MCTS,
            ..Default::default()
        }
    }

    /// Create config for master difficulty
    pub fn master() -> Self {
        Self {
            difficulty: DifficultyLevel::Master,
            depth: 18,
            strategy: AIStrategy::Hybrid,
            ..Default::default()
        }
    }
}

/// AI thinking mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThinkingMode {
    /// Normal thinking
    Normal,
    /// Pondering (thinking during opponent's turn)
    Pondering,
    /// Analysis mode (no move selection)
    Analysis,
}

/// AI move analysis
#[derive(Debug, Clone)]
pub struct AIMoveAnalysis {
    /// Column chosen
    pub column: usize,
    /// Score for this move
    pub score: f64,
    /// Depth searched
    pub depth: usize,
    /// Nodes evaluated
    pub nodes_evaluated: usize,
    /// Time taken
    pub time_ms: u64,
    /// Principal variation
    pub principal_variation: Vec<usize>,
    /// Alternative moves with scores
    pub alternative_moves: Vec<(usize, f64)>,
}

impl AIMoveAnalysis {
    /// Create a new move analysis
    pub fn new(column: usize, score: f64) -> Self {
        Self {
            column,
            score,
            depth: 0,
            nodes_evaluated: 0,
            time_ms: 0,
            principal_variation: Vec::new(),
            alternative_moves: Vec::new(),
        }
    }

    /// Set depth
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Set nodes evaluated
    pub fn with_nodes_evaluated(mut self, nodes: usize) -> Self {
        self.nodes_evaluated = nodes;
        self
    }

    /// Set time
    pub fn with_time(mut self, time_ms: u64) -> Self {
        self.time_ms = time_ms;
        self
    }

    /// Set principal variation
    pub fn with_principal_variation(mut self, pv: Vec<usize>) -> Self {
        self.principal_variation = pv;
        self
    }

    /// Set alternative moves
    pub fn with_alternatives(mut self, alts: Vec<(usize, f64)>) -> Self {
        self.alternative_moves = alts;
        self
    }
}

/// AI player implementation
pub struct AIPlayer {
    /// Configuration
    config: AIPlayerConfig,
    /// Minimax AI instance
    minimax: Option<MinimaxAI>,
    /// Alpha-beta AI instance
    alphabeta: Option<AlphaBetaAI>,
    /// MCTS AI instance
    mcts: Option<MCTSAI>,
    /// Heuristic evaluator
    heuristic: HeuristicEvaluator,
    /// Transposition table
    transposition: Option<SharedTranspositionTable>,
    /// Opening book
    opening_book: Option<OpeningBook>,
    /// Time manager
    time_manager: Option<TimeManager>,
    /// Current thinking mode
    thinking_mode: ThinkingMode,
    /// Game history for learning
    game_history: Vec<(Board, Player, usize)>,
    /// Statistics
    stats: AIPlayerStats,
}

/// AI player statistics
#[derive(Debug, Clone, Default)]
pub struct AIPlayerStats {
    /// Total moves made
    pub total_moves: usize,
    /// Total thinking time (ms)
    pub total_thinking_time_ms: u64,
    /// Average time per move (ms)
    pub avg_time_per_move_ms: f64,
    /// Best score achieved
    pub best_score: f64,
    /// Worst score achieved
    pub worst_score: f64,
    /// Opening book hits
    pub opening_book_hits: usize,
    /// Transposition table hits
    pub transposition_hits: usize,
    /// Nodes evaluated
    pub total_nodes_evaluated: usize,
}

impl AIPlayer {
    /// Create a new AI player
    pub fn new(config: AIPlayerConfig) -> Self {
        let mut player = Self {
            heuristic: HeuristicEvaluator::default(),
            transposition: if config.use_transposition {
                Some(SharedTranspositionTable::default())
            } else {
                None
            },
            opening_book: if config.use_opening_book {
                Some(OpeningBook::default())
            } else {
                None
            },
            minimax: None,
            alphabeta: None,
            mcts: None,
            time_manager: None,
            config,
            thinking_mode: ThinkingMode::Normal,
            game_history: Vec::new(),
            stats: AIPlayerStats::default(),
        };

        player.initialize_strategies();
        player
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(AIPlayerConfig::default())
    }

    /// Initialize AI strategies based on configuration
    fn initialize_strategies(&mut self) {
        match self.config.strategy {
            AIStrategy::Minimax => {
                self.minimax = Some(MinimaxAI::new(self.config.depth));
            }
            AIStrategy::AlphaBeta => {
                self.alphabeta = Some(AlphaBetaAI::new(self.config.depth));
            }
            AIStrategy::MCTS => {
                self.mcts = Some(MCTSAI::new(10000));
            }
            AIStrategy::Hybrid => {
                self.minimax = Some(MinimaxAI::new(self.config.depth));
                self.mcts = Some(MCTSAI::new(5000));
            }
            _ => {}
        }
    }

    /// Choose a move for the given board position
    pub fn choose_move(&mut self, board: &Board, player: Player) -> Option<usize> {
        let start = std::time::Instant::now();

        // Start time management
        if let Some(ref mut tm) = self.time_manager {
            tm.start_search();
        }

        // Try opening book first
        if let Some(ref book) = self.opening_book {
            if let Some(best_move) = book.best_move(board, player) {
                self.stats.opening_book_hits += 1;
                self.record_move(board, player, best_move);
                return Some(best_move);
            }
        }

        // Choose move based on strategy
        let analysis = self.analyze_position(board, player);
        let column = analysis.column;

        // Record statistics
        let elapsed = start.elapsed().as_millis() as u64;
        self.record_move(board, player, column);
        self.record_stats(elapsed, analysis.score, analysis.nodes_evaluated);

        if self.config.show_thinking {
            self.display_thinking(&analysis);
        }

        Some(column)
    }

    /// Analyze position and return detailed analysis
    pub fn analyze_position(&mut self, board: &Board, player: Player) -> AIMoveAnalysis {
        match self.config.strategy {
            AIStrategy::Random => self.analyze_random(board, player),
            AIStrategy::Minimax => self.analyze_minimax(board, player),
            AIStrategy::AlphaBeta => self.analyze_alphabeta(board, player),
            AIStrategy::MCTS => self.analyze_mcts(board, player),
            AIStrategy::Hybrid => self.analyze_hybrid(board, player),
        }
    }

    /// Analyze using random strategy
    fn analyze_random(&self, board: &Board, _player: Player) -> AIMoveAnalysis {
        let valid_moves = board.valid_moves();
        if valid_moves.is_empty() {
            return AIMoveAnalysis::new(0, 0.0);
        }

        let column = valid_moves[rand::random::<usize>() % valid_moves.len()];
        let score = self.heuristic.evaluate(board, Player::Player1);

        AIMoveAnalysis::new(column, score)
    }

    /// Analyze using minimax
    fn analyze_minimax(&mut self, board: &Board, player: Player) -> AIMoveAnalysis {
        if let Some(ref minimax) = self.minimax {
            let column = minimax.find_best_move(board, player, self.config.depth);
            let score = minimax.evaluate(board, player);

            AIMoveAnalysis::new(column.unwrap_or(0), score)
                .with_depth(self.config.depth)
        } else {
            self.analyze_random(board, player)
        }
    }

    /// Analyze using alpha-beta
    fn analyze_alphabeta(&mut self, board: &Board, player: Player) -> AIMoveAnalysis {
        if let Some(ref alphabeta) = self.alphabeta {
            let column = alphabeta.find_best_move(board, player, self.config.depth);
            let score = alphabeta.evaluate(board, player);

            AIMoveAnalysis::new(column.unwrap_or(0), score)
                .with_depth(self.config.depth)
        } else {
            self.analyze_random(board, player)
        }
    }

    /// Analyze using MCTS
    fn analyze_mcts(&mut self, board: &Board, player: Player) -> AIMoveAnalysis {
        if let Some(ref mcts) = self.mcts {
            let column = mcts.find_best_move(board, player);
            let score = mcts.evaluate(board, player);

            AIMoveAnalysis::new(column.unwrap_or(0), score)
        } else {
            self.analyze_random(board, player)
        }
    }

    /// Analyze using hybrid strategy
    fn analyze_hybrid(&mut self, board: &Board, player: Player) -> AIMoveAnalysis {
        // Use minimax for opening, MCTS for middle/endgame
        let piece_count = board.rows() * board.cols() - board.empty_count();

        if piece_count < 10 {
            self.analyze_minimax(board, player)
        } else {
            self.analyze_mcts(board, player)
        }
    }

    /// Record a move in history
    fn record_move(&mut self, board: &Board, player: Player, column: usize) {
        self.game_history.push((board.clone(), player, column));
    }

    /// Record statistics
    fn record_stats(&mut self, time_ms: u64, score: f64, nodes: usize) {
        self.stats.total_moves += 1;
        self.stats.total_thinking_time_ms += time_ms;
        self.stats.total_nodes_evaluated += nodes;

        // Update average time
        self.stats.avg_time_per_move_ms = self.stats.total_thinking_time_ms as f64
            / self.stats.total_moves as f64;

        // Update best/worst scores
        if score > self.stats.best_score {
            self.stats.best_score = score;
        }
        if score < self.stats.worst_score {
            self.stats.worst_score = score;
        }
    }

    /// Display thinking information
    fn display_thinking(&self, analysis: &AIMoveAnalysis) {
        println!("{} thinking:", self.config.name);
        println!("  Move: column {}", analysis.column);
        println!("  Score: {:.2}", analysis.score);
        println!("  Depth: {}", analysis.depth);
        println!("  Nodes: {}", analysis.nodes_evaluated);
        println!("  Time: {} ms", analysis.time_ms);

        if !analysis.principal_variation.is_empty() {
            println!("  PV: {:?}", analysis.principal_variation);
        }

        if !analysis.alternative_moves.is_empty() {
            println!("  Alternatives:");
            for (col, score) in &analysis.alternative_moves {
                println!("    Column {}: {:.2}", col, score);
            }
        }
    }

    /// Get player name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get difficulty level
    pub fn difficulty(&self) -> DifficultyLevel {
        self.config.difficulty
    }

    /// Get statistics
    pub fn stats(&self) -> &AIPlayerStats {
        &self.stats
    }

    /// Get configuration
    pub fn config(&self) -> &AIPlayerConfig {
        &self.config
    }

    /// Set thinking mode
    pub fn set_thinking_mode(&mut self, mode: ThinkingMode) {
        self.thinking_mode = mode;
    }

    /// Get thinking mode
    pub fn thinking_mode(&self) -> ThinkingMode {
        self.thinking_mode
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = AIPlayerStats::default();
    }

    /// Clear game history
    pub fn clear_history(&mut self) {
        self.game_history.clear();
    }

    /// Learn from completed game
    pub fn learn_from_game(&mut self, outcome: GameOutcome) {
        if let Some(ref mut book) = self.opening_book {
            let moves: Vec<_> = self.game_history.iter()
                .map(|(_, player, col)| (*col, *player))
                .collect();

            let ai_outcome = match outcome {
                GameOutcome::Win(Player::Player1) => crate::ai::opening_book::GameOutcome::Win(Player::Player1),
                GameOutcome::Win(Player::Player2) => crate::ai::opening_book::GameOutcome::Win(Player::Player2),
                GameOutcome::Draw => crate::ai::opening_book::GameOutcome::Draw,
            };

            book.learn_from_game(&moves, ai_outcome, self.game_history.len());
        }

        self.clear_history();
    }
}

/// Game outcome for learning
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOutcome {
    Win(Player),
    Draw,
}

impl Default for AIPlayer {
    fn default() -> Self {
        Self::new(AIPlayerConfig::default())
    }
}

/// AI player factory for creating pre-configured players
pub struct AIPlayerFactory;

impl AIPlayerFactory {
    /// Create easy AI player
    pub fn easy(name: &str) -> AIPlayer {
        let mut config = AIPlayerConfig::easy();
        config.name = name.to_string();
        AIPlayer::new(config)
    }

    /// Create medium AI player
    pub fn medium(name: &str) -> AIPlayer {
        let mut config = AIPlayerConfig::medium();
        config.name = name.to_string();
        AIPlayer::new(config)
    }

    /// Create hard AI player
    pub fn hard(name: &str) -> AIPlayer {
        let mut config = AIPlayerConfig::hard();
        config.name = name.to_string();
        AIPlayer::new(config)
    }

    /// Create expert AI player
    pub fn expert(name: &str) -> AIPlayer {
        let mut config = AIPlayerConfig::expert();
        config.name = name.to_string();
        AIPlayer::new(config)
    }

    /// Create master AI player
    pub fn master(name: &str) -> AIPlayer {
        let mut config = AIPlayerConfig::master();
        config.name = name.to_string();
        AIPlayer::new(config)
    }
}

/// Multi-strategy AI player that combines multiple strategies
pub struct MultiStrategyPlayer {
    /// Players for different phases
    opening_player: Option<AIPlayer>,
    middlegame_player: Option<AIPlayer>,
    endgame_player: Option<AIPlayer>,
    /// Piece count thresholds
    opening_threshold: usize,
    endgame_threshold: usize,
}

impl MultiStrategyPlayer {
    /// Create a new multi-strategy player
    pub fn new(
        opening: Option<AIPlayer>,
        middlegame: Option<AIPlayer>,
        endgame: Option<AIPlayer>,
    ) -> Self {
        Self {
            opening_player: opening,
            middlegame_player: middlegame,
            endgame_player: endgame,
            opening_threshold: 6,
            endgame_threshold: 36,
        }
    }

    /// Choose move using appropriate phase player
    pub fn choose_move(&mut self, board: &Board, player: Player) -> Option<usize> {
        let piece_count = board.rows() * board.cols() - board.empty_count();

        if piece_count < self.opening_threshold {
            self.opening_player.as_mut().and_then(|p| p.choose_move(board, player))
        } else if piece_count > self.endgame_threshold {
            self.endgame_player.as_mut().and_then(|p| p.choose_move(board, player))
        } else {
            self.middlegame_player.as_mut().and_then(|p| p.choose_move(board, player))
        }
    }

    /// Set thresholds
    pub fn set_thresholds(&mut self, opening: usize, endgame: usize) {
        self.opening_threshold = opening;
        self.endgame_threshold = endgame;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_player_config_default() {
        let config = AIPlayerConfig::default();
        assert_eq!(config.name, "AI Player");
        assert_eq!(config.depth, 6);
    }

    #[test]
    fn test_ai_player_config_difficulties() {
        let easy = AIPlayerConfig::easy();
        assert_eq!(easy.depth, 3);

        let medium = AIPlayerConfig::medium();
        assert_eq!(medium.depth, 6);

        let hard = AIPlayerConfig::hard();
        assert_eq!(hard.depth, 10);
    }

    #[test]
    fn test_ai_player_creation() {
        let player = AIPlayer::default();
        assert_eq!(player.name(), "AI Player");
        assert_eq!(player.difficulty(), DifficultyLevel::Medium);
    }

    #[test]
    fn test_ai_move_analysis() {
        let analysis = AIMoveAnalysis::new(3, 0.5)
            .with_depth(5)
            .with_time(100)
            .with_nodes_evaluated(1000);

        assert_eq!(analysis.column, 3);
        assert_eq!(analysis.score, 0.5);
        assert_eq!(analysis.depth, 5);
        assert_eq!(analysis.time_ms, 100);
        assert_eq!(analysis.nodes_evaluated, 1000);
    }

    #[test]
    fn test_ai_player_factory() {
        let easy_player = AIPlayerFactory::easy("Easy AI");
        assert_eq!(easy_player.name(), "Easy AI");
        assert_eq!(easy_player.difficulty(), DifficultyLevel::Easy);

        let hard_player = AIPlayerFactory::hard("Hard AI");
        assert_eq!(hard_player.difficulty(), DifficultyLevel::Hard);
    }

    #[test]
    fn test_ai_player_stats() {
        let mut stats = AIPlayerStats::default();
        stats.total_moves = 10;
        stats.total_thinking_time_ms = 5000;
        stats.total_nodes_evaluated = 10000;

        stats.avg_time_per_move_ms = 500.0;

        assert_eq!(stats.total_moves, 10);
        assert_eq!(stats.avg_time_per_move_ms, 500.0);
    }
}
