// core/move_result.rs - Results of game moves
use crate::core::cell::Cell;
use crate::core::coordinates::Position;
use crate::core::player::Player;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// The result of attempting to make a move
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MoveResult {
    /// The move was successful
    Success {
        /// The position where the piece landed
        position: Position,
        /// Whether this move resulted in a win
        is_win: bool,
        /// Whether the board is now full (draw)
        is_draw: bool,
    },
    /// The move was invalid
    Invalid {
        /// The reason the move was invalid
        reason: InvalidMoveReason,
    },
    /// The game is over and no more moves can be made
    GameOver {
        /// The winner, if any
        winner: Option<Player>,
    },
}

impl MoveResult {
    /// Creates a successful move result
    pub fn success(position: Position) -> Self {
        MoveResult::Success {
            position,
            is_win: false,
            is_draw: false,
        }
    }

    /// Creates a successful move result that is a win
    pub fn winning_move(position: Position) -> Self {
        MoveResult::Success {
            position,
            is_win: true,
            is_draw: false,
        }
    }

    /// Creates a successful move result that is a draw
    pub fn draw_move(position: Position) -> Self {
        MoveResult::Success {
            position,
            is_win: false,
            is_draw: true,
        }
    }

    /// Creates an invalid move result
    pub fn invalid(reason: InvalidMoveReason) -> Self {
        MoveResult::Invalid { reason }
    }

    /// Creates a game over result
    pub fn game_over(winner: Option<Player>) -> Self {
        MoveResult::GameOver { winner }
    }

    /// Returns true if the move was successful
    pub const fn is_success(&self) -> bool {
        matches!(self, MoveResult::Success { .. })
    }

    /// Returns true if the move was invalid
    pub const fn is_invalid(&self) -> bool {
        matches!(self, MoveResult::Invalid { .. })
    }

    /// Returns true if the game is over
    pub const fn is_game_over(&self) -> bool {
        matches!(self, MoveResult::GameOver { .. })
    }

    /// Returns true if the move resulted in a win
    pub const fn is_win(&self) -> bool {
        matches!(self, MoveResult::Success { is_win: true, .. })
    }

    /// Returns true if the move resulted in a draw
    pub const fn is_draw(&self) -> bool {
        matches!(self, MoveResult::Success { is_draw: true, .. })
    }

    /// Returns the position where the piece landed
    pub fn position(&self) -> Option<Position> {
        match self {
            MoveResult::Success { position, .. } => Some(*position),
            _ => None,
        }
    }

    /// Returns the reason for an invalid move
    pub fn invalid_reason(&self) -> Option<InvalidMoveReason> {
        match self {
            MoveResult::Invalid { reason } => Some(*reason),
            _ => None,
        }
    }
}

impl Display for MoveResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MoveResult::Success {
                position,
                is_win,
                is_draw,
            } => {
                if *is_win {
                    write!(f, "Winning move at position {:?}", position)
                } else if *is_draw {
                    write!(f, "Draw at position {:?}", position)
                } else {
                    write!(f, "Move successful at position {:?}", position)
                }
            }
            MoveResult::Invalid { reason } => write!(f, "Invalid move: {}", reason),
            MoveResult::GameOver { winner } => {
                match winner {
                    Some(winner) => write!(f, "Game over. {} wins!", winner),
                    None => write!(f, "Game over. It's a draw!"),
                }
            }
        }
    }
}

/// Reasons why a move might be invalid
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum InvalidMoveReason {
    /// The column is out of bounds
    ColumnOutOfBounds,
    /// The column is full
    ColumnFull,
    /// The game is over
    GameOver,
    /// It's not this player's turn
    NotYourTurn,
    /// The move is blocked by another piece
    Blocked,
    /// The special piece is not available
    SpecialNotAvailable,
    /// Invalid move format
    InvalidFormat,
    /// Move too early (game hasn't started)
    TooEarly,
    /// General invalid move
    Invalid,
}

impl InvalidMoveReason {
    /// Returns a description of this reason
    pub const fn description(&self) -> &'static str {
        match self {
            InvalidMoveReason::ColumnOutOfBounds => "Column is out of bounds",
            InvalidMoveReason::ColumnFull => "Column is full",
            InvalidMoveReason::GameOver => "Game is already over",
            InvalidMoveReason::NotYourTurn => "It's not your turn",
            InvalidMoveReason::Blocked => "Move is blocked by another piece",
            InvalidMoveReason::SpecialNotAvailable => "Special piece not available",
            InvalidMoveReason::InvalidFormat => "Invalid move format",
            InvalidMoveReason::TooEarly => "Cannot make a move yet",
            InvalidMoveReason::Invalid => "Invalid move",
        }
    }
}

impl Display for InvalidMoveReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl std::error::Error for InvalidMoveReason {}

/// Detailed move information
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct MoveInfo {
    /// The player who made the move
    pub player: Player,
    /// The column chosen
    pub column: usize,
    /// The row where the piece landed
    pub row: usize,
    /// The position where the piece landed
    pub position: Position,
    /// The cell type placed
    pub cell: Cell,
    /// Whether this was a winning move
    pub is_winning_move: bool,
    /// Whether this completed the board (draw)
    pub is_final_move: bool,
    /// The resulting board state hash
    pub board_hash: u64,
}

impl MoveInfo {
    /// Creates new move info
    pub fn new(
        player: Player,
        column: usize,
        row: usize,
        cell: Cell,
        board_hash: u64,
    ) -> Self {
        MoveInfo {
            player,
            column,
            row,
            position: Position::new(row, column),
            cell,
            is_winning_move: false,
            is_final_move: false,
            board_hash,
        }
    }

    /// Marks this move as a winning move
    pub fn set_winning(&mut self) {
        self.is_winning_move = true;
    }

    /// Marks this move as the final move
    pub fn set_final(&mut self) {
        self.is_final_move = true;
    }
}

/// Statistics about a move
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct MoveStats {
    /// Time taken to make the move (milliseconds)
    pub time_ms: u64,
    /// Number of positions considered
    pub positions_considered: u32,
    /// Depth of search (for AI moves)
    pub search_depth: u32,
    /// Number of alpha-beta cutoffs
    pub cutoffs: u32,
    /// Number of transposition table hits
    pub transposition_hits: u32,
    /// Memory used (bytes)
    pub memory_used: u64,
}

impl MoveStats {
    /// Creates new move stats
    pub const fn new() -> Self {
        MoveStats {
            time_ms: 0,
            positions_considered: 0,
            search_depth: 0,
            cutoffs: 0,
            transposition_hits: 0,
            memory_used: 0,
        }
    }

    /// Creates move stats with timing information
    pub fn with_timing(time_ms: u64) -> Self {
        MoveStats {
            time_ms,
            ..MoveStats::new()
        }
    }

    /// Updates the time taken
    pub fn set_time(&mut self, time_ms: u64) {
        self.time_ms = time_ms;
    }

    /// Increments positions considered
    pub fn add_positions(&mut self, count: u32) {
        self.positions_considered = self.positions_considered.saturating_add(count);
    }

    /// Increments cutoffs
    pub fn add_cutoff(&mut self) {
        self.cutoffs = self.cutoffs.saturating_add(1);
    }

    /// Increments transposition hits
    pub fn add_transposition_hit(&mut self) {
        self.transposition_hits = self.transposition_hits.saturating_add(1);
    }
}

/// Move validation result
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ValidationStatus {
    /// The move is valid and can be made
    Valid,
    /// The move is invalid
    Invalid(InvalidMoveReason),
    /// The move is valid but requires confirmation
    RequiresConfirmation,
}

impl ValidationStatus {
    /// Returns true if the move is valid
    pub const fn is_valid(&self) -> bool {
        matches!(self, ValidationStatus::Valid)
    }

    /// Returns true if the move is invalid
    pub const fn is_invalid(&self) -> bool {
        matches!(self, ValidationStatus::Invalid(_))
    }

    /// Returns true if confirmation is required
    pub const fn requires_confirmation(&self) -> bool {
        matches!(self, ValidationStatus::RequiresConfirmation)
    }

    /// Returns the invalid reason if any
    pub fn invalid_reason(&self) -> Option<InvalidMoveReason> {
        match self {
            ValidationStatus::Invalid(reason) => Some(*reason),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_result_success() {
        let pos = Position::new(5, 3);
        let result = MoveResult::success(pos);
        assert!(result.is_success());
        assert_eq!(result.position(), Some(pos));
        assert!(!result.is_win());
    }

    #[test]
    fn test_move_result_winning() {
        let pos = Position::new(5, 3);
        let result = MoveResult::winning_move(pos);
        assert!(result.is_success());
        assert!(result.is_win());
    }

    #[test]
    fn test_move_result_invalid() {
        let result = MoveResult::invalid(InvalidMoveReason::ColumnFull);
        assert!(result.is_invalid());
        assert_eq!(result.invalid_reason(), Some(InvalidMoveReason::ColumnFull));
    }

    #[test]
    fn test_invalid_move_reason_description() {
        assert_eq!(
            InvalidMoveReason::ColumnOutOfBounds.description(),
            "Column is out of bounds"
        );
    }

    #[test]
    fn test_move_info() {
        let info = MoveInfo::new(Player::Player1, 3, 5, Cell::Player1, 12345);
        assert_eq!(info.player, Player::Player1);
        assert_eq!(info.column, 3);
        assert_eq!(info.row, 5);
    }

    #[test]
    fn test_move_stats() {
        let mut stats = MoveStats::new();
        stats.set_time(1000);
        stats.add_positions(100);
        stats.add_cutoff();
        assert_eq!(stats.time_ms, 1000);
        assert_eq!(stats.positions_considered, 100);
        assert_eq!(stats.cutoffs, 1);
    }

    #[test]
    fn test_validation_status() {
        let valid = ValidationStatus::Valid;
        assert!(valid.is_valid());
        assert!(!valid.is_invalid());

        let invalid = ValidationStatus::Invalid(InvalidMoveReason::Invalid);
        assert!(invalid.is_invalid());
    }
}
