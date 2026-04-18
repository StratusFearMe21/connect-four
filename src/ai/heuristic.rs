// ai/heuristic.rs - Heuristic-based AI evaluation functions
use crate::core::{Board, Player, Cell, Direction};
use std::collections::HashMap;

/// Heuristic evaluation configuration
#[derive(Debug, Clone)]
pub struct HeuristicConfig {
    /// Weight for central column control
    pub center_weight: f64,
    /// Weight for consecutive pieces
    pub consecutive_weight: f64,
    /// Weight for blocking opponent
    pub blocking_weight: f64,
    /// Weight for potential threats
    pub threat_weight: f64,
    /// Weight for strategic positioning
    pub position_weight: f64,
    /// Weight for mobility
    pub mobility_weight: f64,
    /// Lookahead depth for threat detection
    pub threat_depth: usize,
    /// Enable advanced pattern recognition
    pub enable_patterns: bool,
    /// Pattern weights
    pub pattern_weights: HashMap<PatternType, f64>,
}

impl Default for HeuristicConfig {
    fn default() -> Self {
        let mut pattern_weights = HashMap::new();
        pattern_weights.insert(PatternType::TwoInRow, 1.0);
        pattern_weights.insert(PatternType::ThreeInRow, 5.0);
        pattern_weights.insert(PatternType::FourInRow, 100.0);
        pattern_weights.insert(PatternType::TwoGap, 2.0);
        pattern_weights.insert(PatternType::ThreeGap, 10.0);
        pattern_weights.insert(PatternType::FourGap, 1000.0);

        Self {
            center_weight: 3.0,
            consecutive_weight: 4.0,
            blocking_weight: 2.0,
            threat_weight: 6.0,
            position_weight: 1.5,
            mobility_weight: 1.0,
            threat_depth: 2,
            enable_patterns: true,
            pattern_weights,
        }
    }
}

impl HeuristicConfig {
    /// Create aggressive configuration (focus on attacks)
    pub fn aggressive() -> Self {
        let mut config = Self::default();
        config.consecutive_weight = 6.0;
        config.threat_weight = 8.0;
        config.blocking_weight = 1.0;
        config
    }

    /// Create defensive configuration (focus on blocking)
    pub fn defensive() -> Self {
        let mut config = Self::default();
        config.consecutive_weight = 2.0;
        config.threat_weight = 3.0;
        config.blocking_weight = 5.0;
        config
    }

    /// Create balanced configuration
    pub fn balanced() -> Self {
        Self::default()
    }
}

/// Pattern types for pattern recognition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternType {
    /// Two consecutive pieces
    TwoInRow,
    /// Three consecutive pieces
    ThreeInRow,
    /// Four consecutive pieces (winning)
    FourInRow,
    /// Two pieces with one gap (e.g., X_XX)
    TwoGap,
    /// Three pieces with one gap (e.g., X_XXX)
    ThreeGap,
    /// Four pieces with one gap (e.g., X_XXXX)
    FourGap,
}

impl PatternType {
    /// Get base score for pattern type
    pub fn base_score(&self) -> f64 {
        match self {
            PatternType::TwoInRow => 10.0,
            PatternType::ThreeInRow => 100.0,
            PatternType::FourInRow => 10000.0,
            PatternType::TwoGap => 15.0,
            PatternType::ThreeGap => 200.0,
            PatternType::FourGap => 50000.0,
        }
    }

    /// Get pattern length
    pub fn length(&self) -> usize {
        match self {
            PatternType::TwoInRow | PatternType::TwoGap => 2,
            PatternType::ThreeInRow | PatternType::ThreeGap => 3,
            PatternType::FourInRow | PatternType::FourGap => 4,
        }
    }
}

/// Pattern match result
#[derive(Debug, Clone)]
pub struct PatternMatch {
    /// Pattern type
    pub pattern_type: PatternType,
    /// Starting position (row, col)
    pub start: (usize, usize),
    /// Direction of the pattern
    pub direction: Direction,
    /// Player who owns the pattern
    pub player: Player,
    /// Whether the pattern can become a winning line
    pub is_winnable: bool,
    /// Score for this pattern
    pub score: f64,
}

impl PatternMatch {
    /// Create a new pattern match
    pub fn new(
        pattern_type: PatternType,
        start: (usize, usize),
        direction: Direction,
        player: Player,
        is_winnable: bool,
        score: f64,
    ) -> Self {
        Self {
            pattern_type,
            start,
            direction,
            player,
            is_winnable,
            score,
        }
    }

    /// Get end position of the pattern
    pub fn end_position(&self) -> (i32, i32) {
        let (row, col) = self.start;
        let (dr, dc) = self.direction.offset();
        let length = self.pattern_type.length() as i32;
        (
            row as i32 + dr * (length - 1),
            col as i32 + dc * (length - 1),
        )
    }
}

/// Heuristic evaluator using multiple evaluation functions
pub struct HeuristicEvaluator {
    config: HeuristicConfig,
    position_weights: Vec<Vec<f64>>,
    cache: HashMap<u64, f64>,
    stats: HeuristicStats,
}

/// Heuristic evaluation statistics
#[derive(Debug, Clone, Default)]
pub struct HeuristicStats {
    /// Total evaluations
    pub total_evaluations: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// Pattern matches found
    pub pattern_matches: usize,
    /// Average evaluation time (microseconds)
    pub avg_eval_time: f64,
}

impl HeuristicEvaluator {
    /// Create a new heuristic evaluator
    pub fn new(config: HeuristicConfig) -> Self {
        let position_weights = Self::generate_position_weights();

        Self {
            config,
            position_weights,
            cache: HashMap::new(),
            stats: HeuristicStats::default(),
        }
    }

    /// Create with default config
    pub fn default() -> Self {
        Self::new(HeuristicConfig::default())
    }

    /// Generate position weights for the board
    fn generate_position_weights() -> Vec<Vec<f64>> {
        let rows = 6;
        let cols = 7;
        let mut weights = vec![vec![0.0; cols]; rows];
        let center_col = cols / 2;

        for row in 0..rows {
            for col in 0..cols {
                // Higher weight for center columns
                let col_distance = (col as i32 - center_col as i32).abs() as f64;
                let col_weight = 1.0 / (col_distance + 1.0);

                // Higher weight for lower rows (pieces that can be built upon)
                let row_weight = (row + 1) as f64 / rows as f64;

                weights[row][col] = col_weight * row_weight * 10.0;
            }
        }

        weights
    }

    /// Evaluate a board position
    pub fn evaluate(&mut self, board: &Board, player: Player) -> f64 {
        let start = std::time::Instant::now();
        self.stats.total_evaluations += 1;

        let board_hash = self.board_hash(board, player);
        if let Some(&cached) = self.cache.get(&board_hash) {
            self.stats.cache_hits += 1;
            return cached;
        }

        let mut score = 0.0;

        // Center column control
        score += self.evaluate_center_control(board, player);

        // Consecutive pieces
        score += self.evaluate_consecutive(board, player);

        // Blocking opponent
        score += self.evaluate_blocking(board, player);

        // Threat detection
        score += self.evaluate_threats(board, player);

        // Positional advantage
        score += self.evaluate_position(board, player);

        // Mobility
        score += self.evaluate_mobility(board, player);

        // Pattern recognition
        if self.config.enable_patterns {
            score += self.evaluate_patterns(board, player);
        }

        // Cache result
        if self.cache.len() < 10000 {
            self.cache.insert(board_hash, score);
        }

        let elapsed = start.elapsed().as_micros() as f64;
        let total = self.stats.total_evaluations;
        self.stats.avg_eval_time = (self.stats.avg_eval_time * (total - 1) as f64 + elapsed) / total as f64;

        score
    }

    /// Evaluate center column control
    fn evaluate_center_control(&self, board: &Board, player: Player) -> f64 {
        let center_col = board.cols() / 2;
        let mut score = 0.0;

        for row in 0..board.rows() {
            if let Some(cell) = board.get_opt(row, center_col) {
                if cell == Cell::Player1 && player == Player::Player1 {
                    score += self.config.center_weight;
                } else if cell == Cell::Player2 && player == Player::Player2 {
                    score += self.config.center_weight;
                }
            }
        }

        score
    }

    /// Evaluate consecutive pieces in all directions
    fn evaluate_consecutive(&self, board: &Board, player: Player) -> f64 {
        let mut score = 0.0;
        let player_cell = if player == Player::Player1 {
            Cell::Player1
        } else {
            Cell::Player2
        };

        for row in 0..board.rows() {
            for col in 0..board.cols() {
                if let Some(cell) = board.get_opt(row, col) {
                    if cell == player_cell {
                        // Check all directions from this position
                        for direction in Direction::all() {
                            let consecutive = self.count_consecutive(board, row, col, direction, player);
                            score += self.consecutive_score(consecutive);
                        }
                    }
                }
            }
        }

        score * self.config.consecutive_weight
    }

    /// Count consecutive pieces in a direction
    fn count_consecutive(
        &self,
        board: &Board,
        row: usize,
        col: usize,
        direction: Direction,
        player: Player,
    ) -> usize {
        let mut count = 0;
        let player_cell = if player == Player::Player1 {
            Cell::Player1
        } else {
            Cell::Player2
        };

        let (dr, dc) = direction.offset();
        let mut current_row = row as i32;
        let mut current_col = col as i32;

        while current_row >= 0 && current_row < board.rows() as i32
            && current_col >= 0 && current_col < board.cols() as i32
        {
            if let Some(cell) = board.get_opt(current_row as usize, current_col as usize) {
                if cell == player_cell {
                    count += 1;
                } else {
                    break;
                }
            } else {
                break;
            }

            current_row += dr;
            current_col += dc;
        }

        count
    }

    /// Calculate score for consecutive pieces
    fn consecutive_score(&self, count: usize) -> f64 {
        match count {
            0 => 0.0,
            1 => 1.0,
            2 => 10.0,
            3 => 100.0,
            _ => 10000.0,
        }
    }

    /// Evaluate blocking opponent threats
    fn evaluate_blocking(&self, board: &Board, player: Player) -> f64 {
        let opponent = if player == Player::Player1 {
            Player::Player2
        } else {
            Player::Player1
        };

        let mut score = 0.0;

        // Find opponent's threats
        let threats = self.find_threats(board, opponent);

        for threat in &threats {
            if threat.count >= 3 {
                score += threat.count as f64 * self.config.blocking_weight;
            }
        }

        score
    }

    /// Evaluate potential threats
    fn evaluate_threats(&self, board: &Board, player: Player) -> f64 {
        let threats = self.find_threats(board, player);
        let mut score = 0.0;

        for threat in &threats {
            let base_score = match threat.count {
                2 => 10.0,
                3 => 100.0,
                _ => 1000.0,
            };
            score += base_score * self.config.threat_weight;
        }

        score
    }

    /// Find all threats for a player
    fn find_threats(&self, board: &Board, player: Player) -> Vec<Threat> {
        let mut threats = Vec::new();
        let player_cell = if player == Player::Player1 {
            Cell::Player1
        } else {
            Cell::Player2
        };

        for row in 0..board.rows() {
            for col in 0..board.cols() {
                if let Some(cell) = board.get_opt(row, col) {
                    if cell == player_cell {
                        for direction in Direction::all() {
                            let consecutive = self.count_consecutive(board, row, col, direction, player);
                            if consecutive >= 2 {
                                // Check if it can be extended
                                if self.can_extend(board, row, col, direction, consecutive, player) {
                                    threats.push(Threat {
                                        count: consecutive,
                                        start: (row, col),
                                        direction,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        threats
    }

    /// Check if a consecutive sequence can be extended
    fn can_extend(
        &self,
        board: &Board,
        row: usize,
        col: usize,
        direction: Direction,
        consecutive: usize,
        player: Player,
    ) -> bool {
        let (dr, dc) = direction.offset();

        // Check forward extension
        let forward_row = row as i32 + dr * consecutive as i32;
        let forward_col = col as i32 + dc * consecutive as i32;

        if forward_row >= 0 && forward_row < board.rows() as i32
            && forward_col >= 0 && forward_col < board.cols() as i32
        {
            if let Some(cell) = board.get_opt(forward_row as usize, forward_col as usize) {
                if cell == Cell::Empty {
                    return true;
                }
            }
        }

        // Check backward extension
        let backward_row = row as i32 - dr;
        let backward_col = col as i32 - dc;

        if backward_row >= 0 && backward_row < board.rows() as i32
            && backward_col >= 0 && backward_col < board.cols() as i32
        {
            if let Some(cell) = board.get_opt(backward_row as usize, backward_col as usize) {
                if cell == Cell::Empty {
                    return true;
                }
            }
        }

        false
    }

    /// Evaluate positional advantage
    fn evaluate_position(&self, board: &Board, player: Player) -> f64 {
        let mut score = 0.0;
        let player_cell = if player == Player::Player1 {
            Cell::Player1
        } else {
            Cell::Player2
        };

        for row in 0..board.rows() {
            for col in 0..board.cols() {
                if let Some(cell) = board.get_opt(row, col) {
                    if cell == player_cell {
                        score += self.position_weights[row][col];
                    }
                }
            }
        }

        score * self.config.position_weight
    }

    /// Evaluate mobility (number of valid moves)
    fn evaluate_mobility(&self, board: &Board, _player: Player) -> f64 {
        let valid_moves = board.valid_moves();
        let mobility = valid_moves.len() as f64;

        mobility * self.config.mobility_weight
    }

    /// Evaluate patterns on the board
    fn evaluate_patterns(&self, board: &Board, player: Player) -> f64 {
        let patterns = self.find_patterns(board, player);
        let mut score = 0.0;

        for pattern in &patterns {
            let weight = self.config
                .pattern_weights
                .get(&pattern.pattern_type)
                .copied()
                .unwrap_or(1.0);
            score += pattern.score * weight;
            self.stats.pattern_matches += 1;
        }

        score
    }

    /// Find all patterns on the board
    fn find_patterns(&self, board: &Board, player: Player) -> Vec<PatternMatch> {
        let mut patterns = Vec::new();
        let player_cell = if player == Player::Player1 {
            Cell::Player1
        } else {
            Cell::Player2
        };

        // Check each position for patterns
        for row in 0..board.rows() {
            for col in 0..board.cols() {
                if let Some(cell) = board.get_opt(row, col) {
                    if cell == player_cell {
                        for direction in Direction::all() {
                            // Check for consecutive patterns
                            self.check_consecutive_pattern(board, row, col, direction, player, &mut patterns);

                            // Check for gap patterns
                            self.check_gap_pattern(board, row, col, direction, player, &mut patterns);
                        }
                    }
                }
            }
        }

        patterns
    }

    /// Check for consecutive patterns
    fn check_consecutive_pattern(
        &self,
        board: &Board,
        row: usize,
        col: usize,
        direction: Direction,
        player: Player,
        patterns: &mut Vec<PatternMatch>,
    ) {
        let count = self.count_consecutive(board, row, col, direction, player);

        let pattern_type = match count {
            2 => Some(PatternType::TwoInRow),
            3 => Some(PatternType::ThreeInRow),
            4 => Some(PatternType::FourInRow),
            _ => None,
        };

        if let Some(pt) = pattern_type {
            let is_winnable = self.can_extend(board, row, col, direction, count, player);
            let score = pt.base_score() * if is_winnable { 2.0 } else { 1.0 };

            patterns.push(PatternMatch::new(
                pt,
                (row, col),
                direction,
                player,
                is_winnable,
                score,
            ));
        }
    }

    /// Check for gap patterns
    fn check_gap_pattern(
        &self,
        board: &Board,
        row: usize,
        col: usize,
        direction: Direction,
        player: Player,
        patterns: &mut Vec<PatternMatch>,
    ) {
        let (dr, dc) = direction.offset();
        let player_cell = if player == Player::Player1 {
            Cell::Player1
        } else {
            Cell::Player2
        };

        // Check for pattern with one gap: X_XX, X_XXX, etc.
        for gap_distance in 1..=2 {
            let gap_row = row as i32 + dr * gap_distance;
            let gap_col = col as i32 + dc * gap_distance;

            if gap_row >= 0 && gap_row < board.rows() as i32
                && gap_col >= 0 && gap_col < board.cols() as i32
            {
                if let Some(gap_cell) = board.get_opt(gap_row as usize, gap_col as usize) {
                    if gap_cell == Cell::Empty {
                        // Check if pieces after gap match
                        let mut pieces_after = 0;
                        let mut check_row = gap_row + dr;
                        let mut check_col = gap_col + dc;

                        while check_row >= 0 && check_row < board.rows() as i32
                            && check_col >= 0 && check_col < board.cols() as i32
                        {
                            if let Some(cell) = board.get_opt(check_row as usize, check_col as usize) {
                                if cell == player_cell {
                                    pieces_after += 1;
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }

                            check_row += dr;
                            check_col += dc;
                        }

                        if pieces_after >= 2 {
                            let pattern_type = match pieces_after {
                                2 => PatternType::TwoGap,
                                3 => PatternType::ThreeGap,
                                _ => PatternType::FourGap,
                            };

                            let is_winnable = true;
                            let score = pattern_type.base_score();

                            patterns.push(PatternMatch::new(
                                pattern_type,
                                (row, col),
                                direction,
                                player,
                                is_winnable,
                                score,
                            ));
                        }
                    }
                }
            }
        }
    }

    /// Calculate board hash for caching
    fn board_hash(&self, board: &Board, player: Player) -> u64 {
        let mut hash: u64 = player as u64;

        for row in 0..board.rows() {
            for col in 0..board.cols() {
                if let Some(cell) = board.get_opt(row, col) {
                    hash = hash.wrapping_mul(31).wrapping_add(cell as u64);
                }
            }
        }

        hash
    }

    /// Get statistics
    pub fn stats(&self) -> &HeuristicStats {
        &self.stats
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get position weights
    pub fn position_weights(&self) -> &[Vec<f64>] {
        &self.position_weights
    }
}

/// Threat representation
#[derive(Debug, Clone, Copy)]
pub struct Threat {
    /// Number of consecutive pieces
    pub count: usize,
    /// Starting position
    pub start: (usize, usize),
    /// Direction
    pub direction: Direction,
}

impl Threat {
    /// Check if this is a winning threat
    pub fn is_winning(&self) -> bool {
        self.count >= 4
    }

    /// Check if this is a critical threat (3 in a row)
    pub fn is_critical(&self) -> bool {
        self.count == 3
    }
}

/// Threat analyzer for detecting dangerous patterns
pub struct ThreatAnalyzer {
    config: HeuristicConfig,
}

impl ThreatAnalyzer {
    /// Create a new threat analyzer
    pub fn new(config: HeuristicConfig) -> Self {
        Self { config }
    }

    /// Analyze all threats on the board
    pub fn analyze(&self, board: &Board, player: Player) -> ThreatReport {
        let evaluator = HeuristicEvaluator::new(self.config.clone());
        let threats = evaluator.find_threats(board, player);

        let winning_threats = threats.iter().filter(|t| t.is_winning()).count();
        let critical_threats = threats.iter().filter(|t| t.is_critical()).count();
        let total_threats = threats.len();

        // Find most dangerous threat
        let most_dangerous = threats.iter()
            .filter(|t| t.is_critical())
            .max_by_key(|t| t.count)
            .copied();

        ThreatReport {
            total_threats,
            winning_threats,
            critical_threats,
            most_dangerous,
        }
    }

    /// Find best blocking move
    pub fn find_best_block(&self, board: &Board, opponent: Player) -> Option<usize> {
        let threats = HeuristicEvaluator::new(self.config.clone())
            .find_threats(board, opponent);

        // Find critical threats and prioritize blocking
        for threat in threats {
            if threat.is_critical() {
                let (dr, dc) = threat.direction.offset();
                // Try to block at the extension point
                let block_col = threat.start.1 as i32 + dc * threat.count as i32;
                if block_col >= 0 && block_col < board.cols() as i32 {
                    return Some(block_col as usize);
                }
            }
        }

        None
    }
}

/// Threat analysis report
#[derive(Debug, Clone)]
pub struct ThreatReport {
    /// Total number of threats
    pub total_threats: usize,
    /// Number of winning threats
    pub winning_threats: usize,
    /// Number of critical threats (3 in a row)
    pub critical_threats: usize,
    /// Most dangerous threat
    pub most_dangerous: Option<Threat>,
}

impl ThreatReport {
    /// Check if the position is critical (has winning threat)
    pub fn is_critical(&self) -> bool {
        self.winning_threats > 0
    }

    /// Get threat level
    pub fn threat_level(&self) -> ThreatLevel {
        if self.winning_threats > 0 {
            ThreatLevel::Winning
        } else if self.critical_threats > 0 {
            ThreatLevel::Critical
        } else if self.total_threats > 0 {
            ThreatLevel::Potential
        } else {
            ThreatLevel::None
        }
    }
}

/// Threat level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatLevel {
    /// No threats
    None,
    /// Potential threats (less than 3 in a row)
    Potential,
    /// Critical threats (3 in a row)
    Critical,
    /// Winning threats (4 in a row)
    Winning,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heuristic_config_default() {
        let config = HeuristicConfig::default();
        assert_eq!(config.center_weight, 3.0);
        assert_eq!(config.consecutive_weight, 4.0);
    }

    #[test]
    fn test_heuristic_config_aggressive() {
        let config = HeuristicConfig::aggressive();
        assert!(config.consecutive_weight > config.blocking_weight);
    }

    #[test]
    fn test_heuristic_config_defensive() {
        let config = HeuristicConfig::defensive();
        assert!(config.blocking_weight > config.consecutive_weight);
    }

    #[test]
    fn test_pattern_type_base_score() {
        assert_eq!(PatternType::TwoInRow.base_score(), 10.0);
        assert_eq!(PatternType::ThreeInRow.base_score(), 100.0);
        assert_eq!(PatternType::FourInRow.base_score(), 10000.0);
    }

    #[test]
    fn test_pattern_match_creation() {
        let pattern = PatternMatch::new(
            PatternType::ThreeInRow,
            (2, 3),
            Direction::Horizontal,
            Player::Player1,
            true,
            100.0,
        );

        assert_eq!(pattern.pattern_type, PatternType::ThreeInRow);
        assert_eq!(pattern.player, Player::Player1);
    }

    #[test]
    fn test_threat_properties() {
        let threat = Threat {
            count: 3,
            start: (0, 0),
            direction: Direction::Horizontal,
        };

        assert!(!threat.is_winning());
        assert!(threat.is_critical());
    }

    #[test]
    fn test_threat_report() {
        let report = ThreatReport {
            total_threats: 5,
            winning_threats: 0,
            critical_threats: 2,
            most_dangerous: None,
        };

        assert_eq!(report.total_threats, 5);
        assert_eq!(report.threat_level(), ThreatLevel::Critical);
        assert!(!report.is_critical());
    }

    #[test]
    fn test_heuristic_evaluator_creation() {
        let evaluator = HeuristicEvaluator::default();
        assert_eq!(evaluator.position_weights.len(), 6);
        assert_eq!(evaluator.position_weights[0].len(), 7);
    }
}
