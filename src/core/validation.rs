// core/validation.rs - Validation utilities for game operations
use crate::core::board::Board;
use crate::core::cell::Cell;
use crate::core::coordinates::Position;
use crate::core::game_config::GameConfig;
use crate::core::game_state::GameState;
use crate::core::move_result::{InvalidMoveReason, MoveResult, ValidationStatus};
use crate::core::player::Player;
use crate::core::variant::Variant;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Validator for game operations
#[derive(Clone, Debug)]
pub struct GameValidator {
    config: GameConfig,
    variant: Option<Box<dyn Variant>>,
}

impl GameValidator {
    /// Creates a new validator with standard configuration
    pub fn new() -> Self {
        GameValidator {
            config: GameConfig::standard(),
            variant: None,
        }
    }

    /// Creates a validator with a specific configuration
    pub fn with_config(config: GameConfig) -> Self {
        GameValidator {
            config,
            variant: None,
        }
    }

    /// Creates a validator with a specific variant
    pub fn with_variant(variant: Box<dyn Variant>) -> Self {
        let config = GameConfig::standard(); // Would derive from variant in real implementation
        GameValidator {
            config,
            variant: Some(variant),
        }
    }

    /// Validates a move attempt
    pub fn validate_move(
        &self,
        state: &GameState,
        column: usize,
        player: Player,
    ) -> ValidationStatus {
        // Check if game is over
        if state.is_game_over() {
            return ValidationStatus::Invalid(InvalidMoveReason::GameOver);
        }

        // Check if it's the player's turn
        if state.current_player() != player {
            return ValidationStatus::Invalid(InvalidMoveReason::NotYourTurn);
        }

        // Check column bounds
        if column >= state.board().cols() {
            return ValidationStatus::Invalid(InvalidMoveReason::ColumnOutOfBounds);
        }

        // Check if column is full
        if state.board().is_column_full(column) {
            return ValidationStatus::Invalid(InvalidMoveReason::ColumnFull);
        }

        // Use variant-specific validation if available
        if let Some(variant) = &self.variant {
            if !variant.is_valid_move(state.board(), column, player) {
                return ValidationStatus::Invalid(InvalidMoveReason::Invalid);
            }
        }

        ValidationStatus::Valid
    }

    /// Validates a configuration
    pub fn validate_config(&self, config: &GameConfig) -> Result<ValidationReport, Vec<String>> {
        let mut report = ValidationReport::new();
        let mut errors = Vec::new();

        // Validate board dimensions
        if config.rows < 4 {
            errors.push("Board must have at least 4 rows".to_string());
            report.add_error("Board rows too small");
        }

        if config.cols < 4 {
            errors.push("Board must have at least 4 columns".to_string());
            report.add_error("Board columns too small");
        }

        // Validate win length
        if config.win_length < 3 {
            errors.push("Win length must be at least 3".to_string());
            report.add_error("Win length too small");
        }

        if config.win_length > config.rows && config.win_length > config.cols {
            errors.push("Win length cannot exceed board dimensions".to_string());
            report.add_error("Win length exceeds board");
        }

        // Validate player count
        if config.min_players < 2 {
            errors.push("At least 2 players are required".to_string());
            report.add_error("Not enough players");
        }

        if config.max_players < config.min_players {
            errors.push("Max players must be at least min players".to_string());
            report.add_error("Invalid player range");
        }

        // Validate volume
        if config.volume > 100 {
            errors.push("Volume must be between 0 and 100".to_string());
            report.add_error("Invalid volume");
        }

        if errors.is_empty() {
            Ok(report)
        } else {
            Err(errors)
        }
    }

    /// Validates the current game state
    pub fn validate_state(&self, state: &GameState) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Check board integrity
        if state.board().rows() != self.config.rows {
            report.add_error("Board rows don't match config");
        }

        if state.board().cols() != self.config.cols {
            report.add_error("Board columns don't match config");
        }

        // Check turn consistency
        if state.move_count() > 0 && state.turn_number() as usize != state.move_count() + 1 {
            report.add_warning("Turn number doesn't match move count");
        }

        // Check game over consistency
        if state.is_game_over() {
            if state.winner().is_some() && state.is_draw() {
                report.add_error("Game has both winner and draw");
            }

            if state.end_time().is_none() {
                report.add_warning("Game over but no end time");
            }
        }

        // Check board state consistency
        let player1_count = state.board().count_cells(Cell::Player1);
        let player2_count = state.board().count_cells(Cell::Player2);

        if player1_count.abs_diff(player2_count) > 1 {
            report.add_warning("Large piece count imbalance");
        }

        report
    }

    /// Validates that a win condition is correctly detected
    pub fn validate_win_detection(
        &self,
        state: &GameState,
        player: Player,
    ) -> bool {
        if let Some(variant) = &self.variant {
            return variant.check_win(state.board(), player);
        }

        // Default win checking
        self.check_win_standard(state.board(), player, self.config.win_length)
    }

    /// Standard win checking
    fn check_win_standard(&self, board: &Board, player: Player, win_length: usize) -> bool {
        use crate::core::coordinates::Direction;

        for row in 0..board.rows() {
            for col in 0..board.cols() {
                let pos = Position::new(row, col);
                if board.get(&pos) == Some(player.cell()) {
                    for direction in Direction::all().iter() {
                        if board.has_line(&pos, *direction, player.cell(), win_length) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Checks if a board state is reachable from the initial state
    pub fn validate_reachability(&self, state: &GameState) -> bool {
        // Check that pieces fall from the top (gravity)
        for col in 0..state.board().cols() {
            let mut found_empty = false;
            for row in 0..state.board().rows() {
                if state.board().is_empty_at(row, col) {
                    found_empty = true;
                } else if found_empty {
                    // Found a piece below an empty space - violates gravity
                    return false;
                }
            }
        }

        true
    }

    /// Checks if a sequence of moves is valid
    pub fn validate_move_sequence(
        &self,
        columns: &[usize],
        config: &GameConfig,
    ) -> ValidationReport {
        let mut report = ValidationReport::new();
        let mut board = Board::new(config.rows, config.cols);
        let mut player = Player::Player1;

        for (move_num, &col) in columns.iter().enumerate() {
            // Check column bounds
            if col >= board.cols() {
                report.add_error(&format!(
                    "Move {}: Column {} out of bounds",
                    move_num + 1,
                    col
                ));
                continue;
            }

            // Check if column is full
            if board.is_column_full(col) {
                report.add_error(&format!(
                    "Move {}: Column {} is full",
                    move_num + 1,
                    col
                ));
                continue;
            }

            // Place piece
            if let Some(row) = board.lowest_empty_row(col) {
                board.set_at(row, col, player.cell()).unwrap();
                player = player.opponent();
            }
        }

        report
    }
}

impl Default for GameValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Report from validation operations
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    errors: Vec<String>,
    warnings: Vec<String>,
    infos: Vec<String>,
}

impl ValidationReport {
    pub fn new() -> Self {
        ValidationReport {
            errors: Vec::new(),
            warnings: Vec::new(),
            infos: Vec::new(),
        }
    }

    pub fn add_error(&mut self, message: &str) {
        self.errors.push(message.to_string());
    }

    pub fn add_warning(&mut self, message: &str) {
        self.warnings.push(message.to_string());
    }

    pub fn add_info(&mut self, message: &str) {
        self.infos.push(message.to_string());
    }

    pub const fn errors(&self) -> &[String] {
        &self.errors
    }

    pub const fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub const fn infos(&self) -> &[String] {
        &self.infos
    }

    pub const fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub const fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    pub const fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub const fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn merge(&mut self, other: ValidationReport) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.infos.extend(other.infos);
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ValidationReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.errors.is_empty() {
            writeln!(f, "Errors:")?;
            for error in &self.errors {
                writeln!(f, "  - {}", error)?;
            }
        }

        if !self.warnings.is_empty() {
            writeln!(f, "Warnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  - {}", warning)?;
            }
        }

        if !self.infos.is_empty() {
            writeln!(f, "Info:")?;
            for info in &self.infos {
                writeln!(f, "  - {}", info)?;
            }
        }

        if self.errors.is_empty() && self.warnings.is_empty() && self.infos.is_empty() {
            write!(f, "No validation messages")
        } else {
            Ok(())
        }
    }
}

/// Board position validator
#[derive(Clone, Debug)]
pub struct PositionValidator {
    rows: usize,
    cols: usize,
}

impl PositionValidator {
    pub fn new(rows: usize, cols: usize) -> Self {
        PositionValidator { rows, cols }
    }

    pub fn is_valid(&self, pos: &Position) -> bool {
        pos.row < self.rows && pos.col < self.cols
    }

    pub fn validate(&self, pos: &Position) -> Result<(), String> {
        if pos.row >= self.rows {
            return Err(format!(
                "Row {} exceeds maximum of {}",
                pos.row, self.rows
            ));
        }
        if pos.col >= self.cols {
            return Err(format!(
                "Column {} exceeds maximum of {}",
                pos.col, self.cols
            ));
        }
        Ok(())
    }

    pub fn validate_range(&self, positions: &[Position]) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for pos in positions {
            if let Err(e) = self.validate(pos) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_new() {
        let validator = GameValidator::new();
        assert_eq!(validator.config.rows, 6);
        assert_eq!(validator.config.cols, 7);
    }

    #[test]
    fn test_validate_move_valid() {
        let validator = GameValidator::new();
        let state = GameState::standard();
        let status = validator.validate_move(&state, 3, Player::Player1);
        assert!(status.is_valid());
    }

    #[test]
    fn test_validate_move_invalid_column() {
        let validator = GameValidator::new();
        let state = GameState::standard();
        let status = validator.validate_move(&state, 10, Player::Player1);
        assert!(status.is_invalid());
    }

    #[test]
    fn test_validate_config() {
        let validator = GameValidator::new();
        let config = GameConfig::standard();
        let report = validator.validate_config(&config);
        assert!(report.is_ok());
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        report.add_error("Test error");
        report.add_warning("Test warning");

        assert_eq!(report.error_count(), 1);
        assert_eq!(report.warning_count(), 1);
        assert!(!report.is_valid());
        assert!(report.has_warnings());
    }

    #[test]
    fn test_position_validator() {
        let validator = PositionValidator::new(6, 7);
        let pos = Position::new(3, 4);
        assert!(validator.is_valid(&pos));

        let invalid_pos = Position::new(10, 4);
        assert!(!validator.is_valid(&invalid_pos));
    }
}
