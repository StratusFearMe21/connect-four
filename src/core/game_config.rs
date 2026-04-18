// core/game_config.rs - Game configuration options
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Configuration for a game of Connect Four
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct GameConfig {
    /// Number of rows on the board
    pub rows: usize,
    /// Number of columns on the board
    pub cols: usize,
    /// Number of pieces in a row to win
    pub win_length: usize,
    /// Whether to enable special pieces
    pub enable_special_pieces: bool,
    /// Whether to enable power-ups
    pub enable_powerups: bool,
    /// Time limit per move in seconds (None for unlimited)
    pub time_limit: Option<u64>,
    /// Total time limit for each player in seconds (None for unlimited)
    pub total_time_limit: Option<u64>,
    /// Number of blockers per player
    pub blockers_per_player: u32,
    /// Number of bombs per player
    pub bombs_per_player: u32,
    /// Whether to enable undo functionality
    pub allow_undo: bool,
    /// Maximum number of undos (None for unlimited)
    pub max_undos: Option<u32>,
    /// Whether to use the traditional Connect Four rules
    pub traditional_rules: bool,
    /// Whether gravity is enabled (pieces fall to bottom)
    pub gravity_enabled: bool,
    /// Whether pieces can be stacked
    pub allow_stacking: bool,
    /// Minimum players required
    pub min_players: u32,
    /// Maximum players allowed
    pub max_players: u32,
    /// Whether to record move history
    pub record_history: bool,
    /// Whether to enable animations
    pub enable_animations: bool,
    /// Animation speed (milliseconds)
    pub animation_speed: u32,
    /// Whether sound effects are enabled
    pub enable_sound: bool,
    /// Volume level (0-100)
    pub volume: u32,
}

impl GameConfig {
    /// Creates a standard Connect Four configuration
    pub const fn standard() -> Self {
        GameConfig {
            rows: 6,
            cols: 7,
            win_length: 4,
            enable_special_pieces: false,
            enable_powerups: false,
            time_limit: None,
            total_time_limit: None,
            blockers_per_player: 0,
            bombs_per_player: 0,
            allow_undo: false,
            max_undos: None,
            traditional_rules: true,
            gravity_enabled: true,
            allow_stacking: false,
            min_players: 2,
            max_players: 2,
            record_history: true,
            enable_animations: true,
            animation_speed: 200,
            enable_sound: false,
            volume: 50,
        }
    }

    /// Creates a rapid-play configuration
    pub const fn rapid() -> Self {
        GameConfig {
            time_limit: Some(10),
            enable_animations: false,
            ..GameConfig::standard()
        }
    }

    /// Creates a blitz configuration
    pub const fn blitz() -> Self {
        GameConfig {
            time_limit: Some(3),
            enable_animations: false,
            ..GameConfig::standard()
        }
    }

    /// Creates an advanced configuration with special pieces
    pub const fn advanced() -> Self {
        GameConfig {
            enable_special_pieces: true,
            enable_powerups: true,
            blockers_per_player: 2,
            bombs_per_player: 1,
            allow_undo: true,
            max_undos: Some(3),
            ..GameConfig::standard()
        }
    }

    /// Creates a tournament configuration
    pub const fn tournament() -> Self {
        GameConfig {
            total_time_limit: Some(600), // 10 minutes per player
            allow_undo: false,
            traditional_rules: true,
            enable_animations: false,
            ..GameConfig::standard()
        }
    }

    /// Creates a custom configuration
    pub fn custom(
        rows: usize,
        cols: usize,
        win_length: usize,
    ) -> Self {
        GameConfig {
            rows,
            cols,
            win_length,
            ..GameConfig::standard()
        }
    }

    /// Returns the total number of cells on the board
    pub const fn total_cells(&self) -> usize {
        self.rows * self.cols
    }

    /// Returns true if this configuration uses special pieces
    pub const fn has_special_pieces(&self) -> bool {
        self.enable_special_pieces
            || self.blockers_per_player > 0
            || self.bombs_per_player > 0
    }

    /// Returns true if time limits are enabled
    pub const fn has_time_limits(&self) -> bool {
        self.time_limit.is_some() || self.total_time_limit.is_some()
    }

    /// Returns true if undo is enabled
    pub const fn has_undo(&self) -> bool {
        self.allow_undo
    }

    /// Returns true if animations are enabled
    pub const fn has_animations(&self) -> bool {
        self.enable_animations
    }

    /// Returns true if sound is enabled
    pub const fn has_sound(&self) -> bool {
        self.enable_sound
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.rows < 4 {
            return Err("Board must have at least 4 rows".to_string());
        }
        if self.cols < 4 {
            return Err("Board must have at least 4 columns".to_string());
        }
        if self.win_length < 3 {
            return Err("Win length must be at least 3".to_string());
        }
        if self.win_length > self.rows && self.win_length > self.cols {
            return Err("Win length cannot exceed board dimensions".to_string());
        }
        if self.min_players < 2 {
            return Err("At least 2 players are required".to_string());
        }
        if self.max_players < self.min_players {
            return Err("Max players must be at least min players".to_string());
        }
        if self.volume > 100 {
            return Err("Volume must be between 0 and 100".to_string());
        }
        Ok(())
    }

    /// Converts to a variant
    pub fn to_variant(&self) -> GameVariant {
        if self.traditional_rules
            && self.rows == 6
            && self.cols == 7
            && self.win_length == 4
        {
            GameVariant::Standard
        } else if self.rows == 8 && self.cols == 8 {
            GameVariant::EightByEight
        } else if self.rows == 10 && self.cols == 10 && self.win_length == 5 {
            GameVariant::Gomoku
        } else if self.rows == 5 && self.cols == 5 && self.win_length == 4 {
            GameVariant::ConnectFourFive
        } else {
            GameVariant::Custom
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self::standard()
    }
}

impl Display for GameConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Game Configuration:")?;
        writeln!(f, "  Board: {}x{}", self.rows, self.cols)?;
        writeln!(f, "  Win Length: {}", self.win_length)?;
        writeln!(f, "  Special Pieces: {}", self.enable_special_pieces)?;
        writeln!(f, "  Time Limit: {:?}", self.time_limit)?;
        writeln!(f, "  Total Time: {:?}", self.total_time_limit)?;
        writeln!(f, "  Blockers per Player: {}", self.blockers_per_player)?;
        writeln!(f, "  Bombs per Player: {}", self.bombs_per_player)?;
        writeln!(f, "  Allow Undo: {}", self.allow_undo)?;
        writeln!(f, "  Max Undos: {:?}", self.max_undos)?;
        writeln!(f, "  Gravity: {}", self.gravity_enabled)?;
        writeln!(f, "  Min Players: {}", self.min_players)?;
        writeln!(f, "  Max Players: {}", self.max_players)?;
        writeln!(f, "  Animations: {}", self.enable_animations)?;
        writeln!(f, "  Sound: {}", self.enable_sound)
    }
}

/// Preset game variants
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum GameVariant {
    /// Standard 6x7 Connect Four, connect 4 to win
    Standard,
    /// 8x8 board, connect 4 to win
    EightByEight,
    /// 10x10 board, connect 5 to win (Gomoku-style)
    Gomoku,
    /// 5x5 board, connect 4 to win (quick game)
    ConnectFourFive,
    /// 7x7 board, connect 4 to win
    SevenBySeven,
    /// 9x6 board (taller), connect 4 to win
    NineBySix,
    /// 8x10 board (wider), connect 5 to win
    WideBoard,
    /// Custom configuration
    Custom,
}

impl GameVariant {
    /// Returns the config for this variant
    pub fn config(&self) -> GameConfig {
        match self {
            GameVariant::Standard => GameConfig::standard(),
            GameVariant::EightByEight => GameConfig::custom(8, 8, 4),
            GameVariant::Gomoku => GameConfig::custom(10, 10, 5),
            GameVariant::ConnectFourFive => GameConfig::custom(5, 5, 4),
            GameVariant::SevenBySeven => GameConfig::custom(7, 7, 4),
            GameVariant::NineBySix => GameConfig::custom(9, 6, 4),
            GameVariant::WideBoard => GameConfig::custom(8, 10, 5),
            GameVariant::Custom => GameConfig::custom(6, 7, 4),
        }
    }

    /// Returns the name of this variant
    pub const fn name(&self) -> &'static str {
        match self {
            GameVariant::Standard => "Standard",
            GameVariant::EightByEight => "8x8",
            GameVariant::Gomoku => "Gomoku",
            GameVariant::ConnectFourFive => "5x5",
            GameVariant::SevenBySeven => "7x7",
            GameVariant::NineBySix => "9x6",
            GameVariant::WideBoard => "Wide",
            GameVariant::Custom => "Custom",
        }
    }

    /// Returns a description of this variant
    pub const fn description(&self) -> &'static str {
        match self {
            GameVariant::Standard => "Classic Connect Four: 6x7 board, connect 4",
            GameVariant::EightByEight => "8x8 board with connect 4 rules",
            GameVariant::Gomoku => "10x10 board, connect 5 to win",
            GameVariant::ConnectFourFive => "Quick game on 5x5 board, connect 4",
            GameVariant::SevenBySeven => "7x7 square board, connect 4",
            GameVariant::NineBySix => "9x6 taller board, connect 4",
            GameVariant::WideBoard => "8x10 wider board, connect 5",
            GameVariant::Custom => "Custom game configuration",
        }
    }

    /// Returns all available variants
    pub const fn all() -> [GameVariant; 8] {
        [
            GameVariant::Standard,
            GameVariant::EightByEight,
            GameVariant::Gomoku,
            GameVariant::ConnectFourFive,
            GameVariant::SevenBySeven,
            GameVariant::NineBySix,
            GameVariant::WideBoard,
            GameVariant::Custom,
        ]
    }
}

impl Display for GameVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Game mode settings
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum GameMode {
    /// Standard turn-based play
    Standard,
    /// Timed game
    Timed,
    /// Blitz (fast time limits)
    Blitz,
    /// Tournament style
    Tournament,
    /// Practice mode
    Practice,
    /// Puzzle mode
    Puzzle,
    /// Campaign mode
    Campaign,
}

impl GameMode {
    /// Returns the config for this mode
    pub fn config(&self) -> GameConfig {
        match self {
            GameMode::Standard => GameConfig::standard(),
            GameMode::Timed => GameConfig {
                time_limit: Some(30),
                ..GameConfig::standard()
            },
            GameMode::Blitz => GameConfig::blitz(),
            GameMode::Tournament => GameConfig::tournament(),
            GameMode::Practice => GameConfig {
                allow_undo: true,
                max_undos: Some(10),
                time_limit: None,
                ..GameConfig::standard()
            },
            GameMode::Puzzle => GameConfig {
                allow_undo: true,
                max_undos: None,
                ..GameConfig::standard()
            },
            GameMode::Campaign => GameConfig {
                enable_special_pieces: true,
                enable_powerups: true,
                ..GameConfig::standard()
            },
        }
    }

    /// Returns the name of this mode
    pub const fn name(&self) -> &'static str {
        match self {
            GameMode::Standard => "Standard",
            GameMode::Timed => "Timed",
            GameMode::Blitz => "Blitz",
            GameMode::Tournament => "Tournament",
            GameMode::Practice => "Practice",
            GameMode::Puzzle => "Puzzle",
            GameMode::Campaign => "Campaign",
        }
    }
}

impl Display for GameMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_standard() {
        let config = GameConfig::standard();
        assert_eq!(config.rows, 6);
        assert_eq!(config.cols, 7);
        assert_eq!(config.win_length, 4);
    }

    #[test]
    fn test_config_validate() {
        let config = GameConfig::standard();
        assert!(config.validate().is_ok());

        let bad_config = GameConfig {
            rows: 2,
            ..GameConfig::standard()
        };
        assert!(bad_config.validate().is_err());
    }

    #[test]
    fn test_variant_config() {
        let config = GameVariant::Gomoku.config();
        assert_eq!(config.rows, 10);
        assert_eq!(config.cols, 10);
        assert_eq!(config.win_length, 5);
    }

    #[test]
    fn test_mode_config() {
        let config = GameMode::Blitz.config();
        assert_eq!(config.time_limit, Some(3));
    }
}
