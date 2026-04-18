// core/cell.rs - Cell types and related functionality
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Represents the content of a single cell on the game board
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum Cell {
    /// Empty cell with no piece
    Empty,
    /// Contains a piece from player 1 (typically red)
    Player1,
    /// Contains a piece from player 2 (typically yellow)
    Player2,
    /// Contains a blocker piece that cannot be used in a win
    Blocker,
    /// Contains an explosive piece that clears adjacent pieces
    Explosive,
    /// Contains a wall piece that cannot be passed through
    Wall,
    /// Contains a bomb piece that destroys pieces in a column
    Bomb,
    /// Contains a wild piece that can count for either player
    Wild,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Cell::Empty => write!(f, " "),
            Cell::Player1 => write!(f, "●"),
            Cell::Player2 => write!(f, "●"),
            Cell::Blocker => write!(f, "█"),
            Cell::Explosive => write!(f, "✷"),
            Cell::Wall => write!(f, "▓"),
            Cell::Bomb => write!(f, "💣"),
            Cell::Wild => write!(f, "★"),
        }
    }
}

impl FromStr for Cell {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            " " | "0" | "empty" | "e" => Ok(Cell::Empty),
            "●" | "1" | "red" | "p1" => Ok(Cell::Player1),
            "●" | "2" | "yellow" | "p2" => Ok(Cell::Player2),
            "█" | "b" | "blocker" => Ok(Cell::Blocker),
            "✷" | "x" | "explosive" => Ok(Cell::Explosive),
            "▓" | "w" | "wall" => Ok(Cell::Wall),
            "💣" | "!" | "bomb" => Ok(Cell::Bomb),
            "★" | "*" | "wild" => Ok(Cell::Wild),
            _ => Err(format!("Invalid cell value: {}", s)),
        }
    }
}

impl Cell {
    /// Returns the symbol used to display this cell
    pub fn symbol(&self) -> &'static str {
        match self {
            Cell::Empty => " ",
            Cell::Player1 => "●",
            Cell::Player2 => "●",
            Cell::Blocker => "█",
            Cell::Explosive => "✷",
            Cell::Wall => "▓",
            Cell::Bomb => "💣",
            Cell::Wild => "★",
        }
    }

    /// Returns the ASCII symbol used to display this cell
    pub fn ascii_symbol(&self) -> &'static str {
        match self {
            Cell::Empty => " ",
            Cell::Player1 => "O",
            Cell::Player2 => "X",
            Cell::Blocker => "#",
            Cell::Explosive => "*",
            Cell::Wall => "+",
            Cell::Bomb => "@",
            Cell::Wild => "?",
        }
    }

    /// Returns the Unicode symbol used to display this cell
    pub fn unicode_symbol(&self) -> &'static str {
        match self {
            Cell::Empty => " ",
            Cell::Player1 => "●",
            Cell::Player2 => "○",
            Cell::Blocker => "█",
            Cell::Explosive => "✷",
            Cell::Wall => "▓",
            Cell::Bomb => "💣",
            Cell::Wild => "★",
        }
    }

    /// Returns true if the cell is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, Cell::Empty)
    }

    /// Returns true if the cell contains a player piece
    pub fn is_player(&self) -> bool {
        matches!(self, Cell::Player1 | Cell::Player2)
    }

    /// Returns true if the cell contains a special piece
    pub fn is_special(&self) -> bool {
        matches!(self, Cell::Blocker | Cell::Explosive | Cell::Wall | Cell::Bomb | Cell::Wild)
    }

    /// Returns true if the cell can participate in a win
    pub fn can_win(&self) -> bool {
        matches!(self, Cell::Player1 | Cell::Player2 | Cell::Wild)
    }

    /// Returns the color associated with this cell for display purposes
    pub fn color_name(&self) -> &'static str {
        match self {
            Cell::Empty => "reset",
            Cell::Player1 => "red",
            Cell::Player2 => "yellow",
            Cell::Blocker => "gray",
            Cell::Explosive => "orange",
            Cell::Wall => "blue",
            Cell::Bomb => "magenta",
            Cell::Wild => "cyan",
        }
    }

    /// Converts a player number to a cell (1 -> Player1, 2 -> Player2)
    pub fn from_player_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Cell::Player1),
            2 => Some(Cell::Player2),
            _ => None,
        }
    }

    /// Returns the player number if this is a player cell
    pub fn player_number(&self) -> Option<u8> {
        match self {
            Cell::Player1 => Some(1),
            Cell::Player2 => Some(2),
            _ => None,
        }
    }

    /// Returns the opponent cell if this is a player cell
    pub fn opponent(&self) -> Option<Self> {
        match self {
            Cell::Player1 => Some(Cell::Player2),
            Cell::Player2 => Some(Cell::Player1),
            _ => None,
        }
    }

    /// Returns a descriptive string of this cell
    pub fn description(&self) -> &'static str {
        match self {
            Cell::Empty => "empty cell",
            Cell::Player1 => "player 1 piece",
            Cell::Player2 => "player 2 piece",
            Cell::Blocker => "blocker piece",
            Cell::Explosive => "explosive piece",
            Cell::Wall => "wall piece",
            Cell::Bomb => "bomb piece",
            Cell::Wild => "wild piece",
        }
    }

    /// Creates a copy of this cell as the opposite player
    pub fn as_opponent(&self) -> Option<Cell> {
        match self {
            Cell::Player1 => Some(Cell::Player2),
            Cell::Player2 => Some(Cell::Player1),
            _ => None,
        }
    }

    /// Checks if this cell matches the given player cell (including wild cards)
    pub fn matches_player(&self, player: Cell) -> bool {
        match (self, player) {
            (Cell::Wild, _) => true,
            (_, Cell::Wild) => true,
            _ => self == &player,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_default() {
        assert_eq!(Cell::default(), Cell::Empty);
    }

    #[test]
    fn test_cell_display() {
        assert_eq!(Cell::Empty.to_string(), " ");
        assert_eq!(Cell::Player1.to_string(), "●");
        assert_eq!(Cell::Player2.to_string(), "●");
    }

    #[test]
    fn test_cell_from_str() {
        assert_eq!(Cell::from_str(" ").unwrap(), Cell::Empty);
        assert_eq!(Cell::from_str("1").unwrap(), Cell::Player1);
        assert_eq!(Cell::from_str("2").unwrap(), Cell::Player2);
        assert!(Cell::from_str("invalid").is_err());
    }

    #[test]
    fn test_cell_is_empty() {
        assert!(Cell::Empty.is_empty());
        assert!(!Cell::Player1.is_empty());
    }

    #[test]
    fn test_cell_is_player() {
        assert!(Cell::Player1.is_player());
        assert!(Cell::Player2.is_player());
        assert!(!Cell::Empty.is_player());
        assert!(!Cell::Blocker.is_player());
    }

    #[test]
    fn test_cell_is_special() {
        assert!(Cell::Blocker.is_special());
        assert!(Cell::Explosive.is_special());
        assert!(!Cell::Player1.is_special());
    }

    #[test]
    fn test_cell_can_win() {
        assert!(Cell::Player1.can_win());
        assert!(Cell::Wild.can_win());
        assert!(!Cell::Blocker.can_win());
    }

    #[test]
    fn test_cell_from_player_number() {
        assert_eq!(Cell::from_player_number(1), Some(Cell::Player1));
        assert_eq!(Cell::from_player_number(2), Some(Cell::Player2));
        assert_eq!(Cell::from_player_number(3), None);
    }

    #[test]
    fn test_cell_player_number() {
        assert_eq!(Cell::Player1.player_number(), Some(1));
        assert_eq!(Cell::Player2.player_number(), Some(2));
        assert_eq!(Cell::Empty.player_number(), None);
    }

    #[test]
    fn test_cell_opponent() {
        assert_eq!(Cell::Player1.opponent(), Some(Cell::Player2));
        assert_eq!(Cell::Player2.opponent(), Some(Cell::Player1));
        assert_eq!(Cell::Empty.opponent(), None);
    }

    #[test]
    fn test_cell_matches_player() {
        assert!(Cell::Player1.matches_player(Cell::Player1));
        assert!(Cell::Wild.matches_player(Cell::Player1));
        assert!(Cell::Player1.matches_player(Cell::Wild));
        assert!(!Cell::Player1.matches_player(Cell::Player2));
    }

    #[test]
    fn test_cell_symbols() {
        assert_eq!(Cell::Empty.symbol(), " ");
        assert_eq!(Cell::Player1.symbol(), "●");
        assert_eq!(Cell::Blocker.symbol(), "█");

        assert_eq!(Cell::Empty.ascii_symbol(), " ");
        assert_eq!(Cell::Player1.ascii_symbol(), "O");
        assert_eq!(Cell::Player2.ascii_symbol(), "X");

        assert_eq!(Cell::Player1.unicode_symbol(), "●");
        assert_eq!(Cell::Player2.unicode_symbol(), "○");
    }

    #[test]
    fn test_cell_descriptions() {
        assert_eq!(Cell::Empty.description(), "empty cell");
        assert_eq!(Cell::Player1.description(), "player 1 piece");
        assert_eq!(Cell::Blocker.description(), "blocker piece");
    }
}
