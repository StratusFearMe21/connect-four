// core/player.rs - Player types and management
use crate::core::cell::Cell;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Represents a player in the game
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum Player {
    /// Player 1 (first player, typically uses red pieces)
    Player1,
    /// Player 2 (second player, typically uses yellow pieces)
    Player2,
}

impl Player {
    /// Returns the opponent player
    pub const fn opponent(&self) -> Self {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }

    /// Returns the cell type associated with this player
    pub const fn cell(&self) -> Cell {
        match self {
            Player::Player1 => Cell::Player1,
            Player::Player2 => Cell::Player2,
        }
    }

    /// Returns the player number (1 or 2)
    pub const fn number(&self) -> u8 {
        match self {
            Player::Player1 => 1,
            Player::Player2 => 2,
        }
    }

    /// Creates a player from a cell type
    pub fn from_cell(cell: Cell) -> Option<Self> {
        match cell {
            Cell::Player1 => Some(Player::Player1),
            Cell::Player2 => Some(Player::Player2),
            _ => None,
        }
    }

    /// Creates a player from a player number (1 or 2)
    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Player::Player1),
            2 => Some(Player::Player2),
            _ => None,
        }
    }

    /// Returns the default player (Player 1)
    pub const fn default() -> Self {
        Player::Player1
    }

    /// Returns the color name for this player
    pub const fn color_name(&self) -> &'static str {
        match self {
            Player::Player1 => "red",
            Player::Player2 => "yellow",
        }
    }

    /// Returns the player as a string
    pub const fn as_str(&self) -> &'static str {
        match self {
            Player::Player1 => "Player 1",
            Player::Player2 => "Player 2",
        }
    }

    /// Returns a short identifier ('P1' or 'P2')
    pub const fn short_id(&self) -> &'static str {
        match self {
            Player::Player1 => "P1",
            Player::Player2 => "P2",
        }
    }

    /// Returns an icon for this player
    pub const fn icon(&self) -> char {
        match self {
            Player::Player1 => '●',
            Player::Player2 => '○',
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Player {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "1" | "p1" | "player1" | "player 1" | "red" => Ok(Player::Player1),
            "2" | "p2" | "player2" | "player 2" | "yellow" => Ok(Player::Player2),
            _ => Err(format!("Invalid player: {}", s)),
        }
    }
}

/// Player type for different game modes
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum PlayerType {
    /// Human player
    Human,
    /// AI player
    AI,
    /// Remote/network player
    Remote,
    /// CPU player (simple random moves)
    CPU,
}

impl PlayerType {
    /// Returns true if this is a human player
    pub const fn is_human(&self) -> bool {
        matches!(self, PlayerType::Human)
    }

    /// Returns true if this is an AI or CPU player
    pub const fn is_computer(&self) -> bool {
        matches!(self, PlayerType::AI | PlayerType::CPU)
    }

    /// Returns true if this is a remote player
    pub const fn is_remote(&self) -> bool {
        matches!(self, PlayerType::Remote)
    }

    /// Returns the type as a string
    pub const fn as_str(&self) -> &'static str {
        match self {
            PlayerType::Human => "Human",
            PlayerType::AI => "AI",
            PlayerType::Remote => "Remote",
            PlayerType::CPU => "CPU",
        }
    }
}

impl Display for PlayerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Player information including name and type
#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub player: Player,
    pub player_type: PlayerType,
    pub name: String,
}

impl PlayerInfo {
    /// Creates a new player info
    pub fn new(player: Player, player_type: PlayerType, name: String) -> Self {
        PlayerInfo {
            player,
            player_type,
            name,
        }
    }

    /// Creates a default human player info
    pub fn human_player1() -> Self {
        PlayerInfo {
            player: Player::Player1,
            player_type: PlayerType::Human,
            name: "Player 1".to_string(),
        }
    }

    /// Creates a default human player 2 info
    pub fn human_player2() -> Self {
        PlayerInfo {
            player: Player::Player2,
            player_type: PlayerType::Human,
            name: "Player 2".to_string(),
        }
    }

    /// Creates a default AI player info
    pub fn ai_player1() -> Self {
        PlayerInfo {
            player: Player::Player1,
            player_type: PlayerType::AI,
            name: "AI 1".to_string(),
        }
    }

    /// Creates a default AI player 2 info
    pub fn ai_player2() -> Self {
        PlayerInfo {
            player: Player::Player2,
            player_type: PlayerType::AI,
            name: "AI 2".to_string(),
        }
    }

    /// Returns the cell type for this player
    pub const fn cell(&self) -> Cell {
        self.player.cell()
    }

    /// Returns the player number
    pub const fn number(&self) -> u8 {
        self.player.number()
    }

    /// Returns a display string
    pub fn display(&self) -> String {
        format!("{} ({})", self.name, self.player_type)
    }
}

/// Turn order for games
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum TurnOrder {
    /// Player 1 goes first
    Player1First,
    /// Player 2 goes first
    Player2First,
    /// Random first player
    Random,
}

impl TurnOrder {
    /// Returns the first player
    pub fn first_player(&self) -> Player {
        match self {
            TurnOrder::Player1First => Player::Player1,
            TurnOrder::Player2First => Player::Player2,
            TurnOrder::Random => {
                if rand::random::<bool>() {
                    Player::Player1
                } else {
                    Player::Player2
                }
            }
        }
    }
}

/// Player turn tracking
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Turn {
    pub current_player: Player,
    pub turn_number: u32,
}

impl Turn {
    /// Creates a new turn
    pub const fn new(current_player: Player, turn_number: u32) -> Self {
        Turn {
            current_player,
            turn_number,
        }
    }

    /// Creates the first turn
    pub const fn first(player: Player) -> Self {
        Turn::new(player, 1)
    }

    /// Moves to the next turn
    pub fn next(&self) -> Self {
        Turn {
            current_player: self.current_player.opponent(),
            turn_number: self.turn_number + 1,
        }
    }

    /// Returns the current player
    pub const fn player(&self) -> Player {
        self.current_player
    }

    /// Returns the turn number
    pub const fn number(&self) -> u32 {
        self.turn_number
    }

    /// Returns true if it's player 1's turn
    pub const fn is_player1_turn(&self) -> bool {
        self.current_player == Player::Player1
    }

    /// Returns true if it's player 2's turn
    pub const fn is_player2_turn(&self) -> bool {
        self.current_player == Player::Player2
    }
}

impl Display for Turn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Turn {}: {}", self.turn_number, self.current_player)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_opponent() {
        assert_eq!(Player::Player1.opponent(), Player::Player2);
        assert_eq!(Player::Player2.opponent(), Player::Player1);
    }

    #[test]
    fn test_player_cell() {
        assert_eq!(Player::Player1.cell(), Cell::Player1);
        assert_eq!(Player::Player2.cell(), Cell::Player2);
    }

    #[test]
    fn test_player_number() {
        assert_eq!(Player::Player1.number(), 1);
        assert_eq!(Player::Player2.number(), 2);
    }

    #[test]
    fn test_player_from_number() {
        assert_eq!(Player::from_number(1), Some(Player::Player1));
        assert_eq!(Player::from_number(2), Some(Player::Player2));
        assert_eq!(Player::from_number(3), None);
    }

    #[test]
    fn test_player_from_cell() {
        assert_eq!(Player::from_cell(Cell::Player1), Some(Player::Player1));
        assert_eq!(Player::from_cell(Cell::Player2), Some(Player::Player2));
        assert_eq!(Player::from_cell(Cell::Empty), None);
    }

    #[test]
    fn test_player_type_is_human() {
        assert!(PlayerType::Human.is_human());
        assert!(!PlayerType::AI.is_human());
    }

    #[test]
    fn test_player_type_is_computer() {
        assert!(PlayerType::AI.is_computer());
        assert!(PlayerType::CPU.is_computer());
        assert!(!PlayerType::Human.is_computer());
    }

    #[test]
    fn test_player_info_display() {
        let info = PlayerInfo::human_player1();
        assert!(info.display().contains("Player 1"));
        assert!(info.display().contains("Human"));
    }

    #[test]
    fn test_turn_next() {
        let turn = Turn::first(Player::Player1);
        let next = turn.next();
        assert_eq!(next.player(), Player::Player2);
        assert_eq!(next.number(), 2);
    }

    #[test]
    fn test_player_from_str() {
        assert_eq!(Player::from_str("1").unwrap(), Player::Player1);
        assert_eq!(Player::from_str("p2").unwrap(), Player::Player2);
        assert!(Player::from_str("invalid").is_err());
    }
}
