// core/variant.rs - Game variant implementations
use crate::core::board::Board;
use crate::core::cell::Cell;
use crate::core::coordinates::Position;
use crate::core::game_config::GameVariant;
use crate::core::player::Player;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Trait for game variants with custom rules
pub trait Variant {
    /// Returns the variant type
    fn variant_type(&self) -> GameVariant;

    /// Returns the board dimensions
    fn board_dimensions(&self) -> (usize, usize);

    /// Returns the win condition length
    fn win_length(&self) -> usize;

    /// Checks if a move is valid according to variant rules
    fn is_valid_move(&self, board: &Board, column: usize, player: Player) -> bool;

    /// Creates a board for this variant
    fn create_board(&self) -> Board;

    /// Checks if the current state is a win for the given player
    fn check_win(&self, board: &Board, player: Player) -> bool;

    /// Returns the name of this variant
    fn name(&self) -> &str;

    /// Returns a description of this variant
    fn description(&self) -> &str;
}

/// Standard Connect Four variant
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct StandardVariant;

impl Variant for StandardVariant {
    fn variant_type(&self) -> GameVariant {
        GameVariant::Standard
    }

    fn board_dimensions(&self) -> (usize, usize) {
        (6, 7)
    }

    fn win_length(&self) -> usize {
        4
    }

    fn is_valid_move(&self, board: &Board, column: usize, _player: Player) -> bool {
        column < board.cols() && !board.is_column_full(column)
    }

    fn create_board(&self) -> Board {
        Board::standard()
    }

    fn check_win(&self, board: &Board, player: Player) -> bool {
        self.check_win_standard(board, player, 4)
    }

    fn name(&self) -> &str {
        "Standard Connect Four"
    }

    fn description(&self) -> &str {
        "Classic Connect Four: 6x7 board, connect 4 pieces to win"
    }
}

impl StandardVariant {
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
}

/// Gomoku variant (5 in a row on 10x10 board)
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct GomokuVariant;

impl Variant for GomokuVariant {
    fn variant_type(&self) -> GameVariant {
        GameVariant::Gomoku
    }

    fn board_dimensions(&self) -> (usize, usize) {
        (10, 10)
    }

    fn win_length(&self) -> usize {
        5
    }

    fn is_valid_move(&self, board: &Board, column: usize, _player: Player) -> bool {
        column < board.cols() && !board.is_column_full(column)
    }

    fn create_board(&self) -> Board {
        Board::new(10, 10)
    }

    fn check_win(&self, board: &Board, player: Player) -> bool {
        self.check_win_standard(board, player, 5)
    }

    fn name(&self) -> &str {
        "Gomoku"
    }

    fn description(&self) -> &str {
        "10x10 board, connect 5 pieces to win"
    }
}

impl GomokuVariant {
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
}

/// Anti-Connect Four variant (avoid connecting 4)
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct AntiConnectFourVariant;

impl Variant for AntiConnectFourVariant {
    fn variant_type(&self) -> GameVariant {
        GameVariant::Standard // Reuses standard board
    }

    fn board_dimensions(&self) -> (usize, usize) {
        (6, 7)
    }

    fn win_length(&self) -> usize {
        4
    }

    fn is_valid_move(&self, board: &Board, column: usize, _player: Player) -> bool {
        column < board.cols() && !board.is_column_full(column)
    }

    fn create_board(&self) -> Board {
        Board::standard()
    }

    fn check_win(&self, board: &Board, player: Player) -> bool {
        // In Anti-Connect Four, the opponent wins if you connect 4
        let opponent = player.opponent();
        self.check_win_anti(board, player, 4)
    }

    fn name(&self) -> &str {
        "Anti-Connect Four"
    }

    fn description(&self) -> &str {
        "Avoid connecting 4! The first player to connect 4 loses"
    }
}

impl AntiConnectFourVariant {
    fn check_win_anti(&self, board: &Board, player: Player, win_length: usize) -> bool {
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
}

/// PopOut variant (can remove bottom pieces)
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PopOutVariant;

impl Variant for PopOutVariant {
    fn variant_type(&self) -> GameVariant {
        GameVariant::Standard
    }

    fn board_dimensions(&self) -> (usize, usize) {
        (6, 7)
    }

    fn win_length(&self) -> usize {
        4
    }

    fn is_valid_move(&self, board: &Board, column: usize, player: Player) -> bool {
        // Can either drop a piece or pop out from bottom
        let can_drop = column < board.cols() && !board.is_column_full(column);
        let can_pop = column < board.cols()
            && board.get_at(0, column) == Some(player.cell());
        can_drop || can_pop
    }

    fn create_board(&self) -> Board {
        Board::standard()
    }

    fn check_win(&self, board: &Board, player: Player) -> bool {
        use crate::core::coordinates::Direction;

        for row in 0..board.rows() {
            for col in 0..board.cols() {
                let pos = Position::new(row, col);
                if board.get(&pos) == Some(player.cell()) {
                    for direction in Direction::all().iter() {
                        if board.has_line(&pos, *direction, player.cell(), 4) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn name(&self) -> &str {
        "PopOut"
    }

    fn description(&self) -> &str {
        "Can either drop pieces or pop out from the bottom of a column"
    }
}

/// Gravity variant (pieces fall diagonally)
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct GravityVariant {
    gravity_direction: GravityDirection,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum GravityDirection {
    Down,
    Left,
    Right,
    Up,
}

impl GravityVariant {
    pub const fn new(direction: GravityDirection) -> Self {
        GravityVariant {
            gravity_direction: direction,
        }
    }

    pub const fn direction(&self) -> GravityDirection {
        self.gravity_direction
    }
}

impl Variant for GravityVariant {
    fn variant_type(&self) -> GameVariant {
        GameVariant::Standard
    }

    fn board_dimensions(&self) -> (usize, usize) {
        (6, 7)
    }

    fn win_length(&self) -> usize {
        4
    }

    fn is_valid_move(&self, board: &Board, column: usize, _player: Player) -> bool {
        match self.gravity_direction {
            GravityDirection::Down => column < board.cols() && !board.is_column_full(column),
            GravityDirection::Left => column < board.cols() && column > 0,
            GravityDirection::Right => column < board.cols() && column < board.cols() - 1,
            GravityDirection::Up => column < board.cols() && !board.is_column_full(column),
        }
    }

    fn create_board(&self) -> Board {
        Board::standard()
    }

    fn check_win(&self, board: &Board, player: Player) -> bool {
        use crate::core::coordinates::Direction;

        for row in 0..board.rows() {
            for col in 0..board.cols() {
                let pos = Position::new(row, col);
                if board.get(&pos) == Some(player.cell()) {
                    for direction in Direction::all().iter() {
                        if board.has_line(&pos, *direction, player.cell(), 4) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn name(&self) -> &str {
        match self.gravity_direction {
            GravityDirection::Down => "Gravity (Down)",
            GravityDirection::Left => "Gravity (Left)",
            GravityDirection::Right => "Gravity (Right)",
            GravityDirection::Up => "Gravity (Up)",
        }
    }

    fn description(&self) -> &str {
        "Pieces fall in the direction of gravity"
    }
}

/// Factory to create variant instances
pub struct VariantFactory;

impl VariantFactory {
    pub fn create(variant: GameVariant) -> Box<dyn Variant> {
        match variant {
            GameVariant::Standard => Box::new(StandardVariant),
            GameVariant::Gomoku => Box::new(GomokuVariant),
            _ => Box::new(StandardVariant),
        }
    }

    pub fn create_anti() -> Box<dyn Variant> {
        Box::new(AntiConnectFourVariant)
    }

    pub fn create_popout() -> Box<dyn Variant> {
        Box::new(PopOutVariant)
    }

    pub fn create_gravity(direction: GravityDirection) -> Box<dyn Variant> {
        Box::new(GravityVariant::new(direction))
    }
}

impl Display for dyn Variant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name(), self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_variant() {
        let variant = StandardVariant;
        assert_eq!(variant.board_dimensions(), (6, 7));
        assert_eq!(variant.win_length(), 4);
        assert_eq!(variant.name(), "Standard Connect Four");
    }

    #[test]
    fn test_gomoku_variant() {
        let variant = GomokuVariant;
        assert_eq!(variant.board_dimensions(), (10, 10));
        assert_eq!(variant.win_length(), 5);
        assert_eq!(variant.name(), "Gomoku");
    }

    #[test]
    fn test_create_board() {
        let board = StandardVariant.create_board();
        assert_eq!(board.rows(), 6);
        assert_eq!(board.cols(), 7);
    }

    #[test]
    fn test_variant_factory() {
        let variant = VariantFactory::create(GameVariant::Gomoku);
        assert_eq!(variant.name(), "Gomoku");
    }
}
