// ui/colors.rs - Color definitions and utilities for the UI
use ratatui::style::{Color, Modifier, Style};
use std::str::FromStr;

/// Common color palette for the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaletteColor {
    /// Primary color
    Primary,
    /// Secondary color
    Secondary,
    /// Success color
    Success,
    /// Warning color
    Warning,
    /// Error color
    Error,
    /// Info color
    Info,
    /// Background color
    Background,
    /// Foreground color
    Foreground,
    /// Border color
    Border,
    /// Muted color
    Muted,
}

impl PaletteColor {
    /// Get the color value for the given theme
    pub fn to_color(&self, theme: &Theme) -> Color {
        match theme {
            Theme::Light => self.to_light_color(),
            Theme::Dark => self.to_dark_color(),
            Theme::Retro => self.to_retro_color(),
            Theme::Neon => self.to_neon_color(),
        }
    }

    /// Get light theme color
    fn to_light_color(&self) -> Color {
        match self {
            Self::Primary => Color::Blue,
            Self::Secondary => Color::Cyan,
            Self::Success => Color::Green,
            Self::Warning => Color::Yellow,
            Self::Error => Color::Red,
            Self::Info => Color::LightBlue,
            Self::Background => Color::White,
            Self::Foreground => Color::Black,
            Self::Border => Color::Gray,
            Self::Muted => Color::DarkGray,
        }
    }

    /// Get dark theme color
    fn to_dark_color(&self) -> Color {
        match self {
            Self::Primary => Color::LightBlue,
            Self::Secondary => Color::Cyan,
            Self::Success => Color::LightGreen,
            Self::Warning => Color::LightYellow,
            Self::Error => Color::LightRed,
            Self::Info => Color::LightCyan,
            Self::Background => Color::Black,
            Self::Foreground => Color::White,
            Self::Border => Color::DarkGray,
            Self::Muted => Color::Gray,
        }
    }

    /// Get retro theme color
    fn to_retro_color(&self) -> Color {
        match self {
            Self::Primary => Color::Rgb(100, 149, 237), // Cornflower blue
            Self::Secondary => Color::Rgb(135, 206, 250), // Light sky blue
            Self::Success => Color::Rgb(60, 179, 113), // Medium sea green
            Self::Warning => Color::Rgb(255, 215, 0), // Gold
            Self::Error => Color::Rgb(205, 92, 92), // Indian red
            Self::Info => Color::Rgb(70, 130, 180), // Steel blue
            Self::Background => Color::Rgb(240, 248, 255), // Alice blue
            Self::Foreground => Color::Rgb(47, 79, 79), // Dark slate gray
            Self::Border => Color::Rgb(176, 196, 222), // Light steel blue
            Self::Muted => Color::Rgb(119, 136, 153), // Light slate gray
        }
    }

    /// Get neon theme color
    fn to_neon_color(&self) -> Color {
        match self {
            Self::Primary => Color::Rgb(0, 255, 255), // Cyan neon
            Self::Secondary => Color::Rgb(255, 0, 255), // Magenta neon
            Self::Success => Color::Rgb(57, 255, 20), // Green neon
            Self::Warning => Color::Rgb(255, 255, 0), // Yellow neon
            Self::Error => Color::Rgb(255, 0, 85), // Red neon
            Self::Info => Color::Rgb(0, 255, 255), // Cyan neon
            Self::Background => Color::Rgb(10, 10, 10), // Dark background
            Self::Foreground => Color::Rgb(255, 255, 255), // White text
            Self::Border => Color::Rgb(0, 255, 255), // Cyan border
            Self::Muted => Color::Rgb(100, 100, 100), // Dimmed
        }
    }
}

/// Game-specific colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameColor {
    /// Player 1 (Red)
    Player1,
    /// Player 2 (Yellow)
    Player2,
    /// Empty cell
    Empty,
    /// Highlighted cell
    Highlight,
    /// Selected column
    Selected,
    /// Last move
    LastMove,
    /// Winning line
    Winning,
}

impl GameColor {
    /// Convert to ratatui Color
    pub fn to_color(&self) -> Color {
        match self {
            Self::Player1 => Color::Red,
            Self::Player2 => Color::Yellow,
            Self::Empty => Color::Reset,
            Self::Highlight => Color::LightBlue,
            Self::Selected => Color::LightCyan,
            Self::LastMove => Color::LightGreen,
            Self::Winning => Color::Green,
        }
    }

    /// Convert to RGB color
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Player1 => (255, 0, 0),
            Self::Player2 => (255, 255, 0),
            Self::Empty => (240, 240, 240),
            Self::Highlight => (173, 216, 230),
            Self::Selected => (224, 255, 255),
            Self::LastMove => (144, 238, 144),
            Self::Winning => (0, 255, 0),
        }
    }
}

/// Theme selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Theme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// Retro theme
    Retro,
    /// Neon theme
    Neon,
}

impl Theme {
    /// Get all available themes
    pub fn all() -> Vec<Theme> {
        vec![Theme::Light, Theme::Dark, Theme::Retro, Theme::Neon]
    }

    /// Get theme name
    pub fn name(&self) -> &str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
            Self::Retro => "Retro",
            Self::Neon => "Neon",
        }
    }
}

impl FromStr for Theme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            "retro" => Ok(Self::Retro),
            "neon" => Ok(Self::Neon),
            _ => Err(format!("Unknown theme: {}", s)),
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Color utilities
pub struct ColorUtils;

impl ColorUtils {
    /// Interpolate between two colors
    pub fn interpolate(color1: Color, color2: Color, t: f64) -> Color {
        let (r1, g1, b1) = Self::color_to_rgb(color1);
        let (r2, g2, b2) = Self::color_to_rgb(color2);

        let r = (r1 as f64 + (r2 as f64 - r1 as f64) * t) as u8;
        let g = (g1 as f64 + (g2 as f64 - g1 as f64) * t) as u8;
        let b = (b1 as f64 + (b2 as f64 - b1 as f64) * t) as u8;

        Color::Rgb(r, g, b)
    }

    /// Darken a color
    pub fn darken(color: Color, amount: f64) -> Color {
        let (r, g, b) = Self::color_to_rgb(color);
        let factor = 1.0 - amount.clamp(0.0, 1.0);

        Color::Rgb(
            (r as f64 * factor) as u8,
            (g as f64 * factor) as u8,
            (b as f64 * factor) as u8,
        )
    }

    /// Lighten a color
    pub fn lighten(color: Color, amount: f64) -> Color {
        let (r, g, b) = Self::color_to_rgb(color);
        let factor = amount.clamp(0.0, 1.0);

        Color::Rgb(
            (r as f64 + (255.0 - r as f64) * factor) as u8,
            (g as f64 + (255.0 - g as f64) * factor) as u8,
            (b as f64 + (255.0 - b as f64) * factor) as u8,
        )
    }

    /// Convert Color to RGB
    fn color_to_rgb(color: Color) -> (u8, u8, u8) {
        match color {
            Color::Black => (0, 0, 0),
            Color::Red => (255, 0, 0),
            Color::Green => (0, 255, 0),
            Color::Yellow => (255, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::Magenta => (255, 0, 255),
            Color::Cyan => (0, 255, 255),
            Color::White => (255, 255, 255),
            Color::Gray => (128, 128, 128),
            Color::DarkGray => (64, 64, 64),
            Color::LightRed => (255, 128, 128),
            Color::LightGreen => (128, 255, 128),
            Color::LightYellow => (255, 255, 128),
            Color::LightBlue => (128, 128, 255),
            Color::LightMagenta => (255, 128, 255),
            Color::LightCyan => (128, 255, 255),
            Color::Indexed(_) => (255, 255, 255),
            Color::Rgb(r, g, b) => (r, g, b),
            Color::Reset => (255, 255, 255),
        }
    }

    /// Get hex string for a color
    pub fn to_hex(color: Color) -> String {
        let (r, g, b) = Self::color_to_rgb(color);
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    /// Parse hex string to color
    pub fn from_hex(hex: &str) -> Option<Color> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Color::Rgb(r, g, b))
    }

    /// Get contrasting text color (black or white)
    pub fn get_contrast_color(color: Color) -> Color {
        let (r, g, b) = Self::color_to_rgb(color);
        let luminance = (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64) / 255.0;

        if luminance > 0.5 {
            Color::Black
        } else {
            Color::White
        }
    }

    /// Create a gradient of colors
    pub fn create_gradient(colors: &[Color], steps: usize) -> Vec<Color> {
        if colors.is_empty() {
            return Vec::new();
        }
        if colors.len() == 1 {
            return vec![colors[0]; steps];
        }

        let mut gradient = Vec::with_capacity(steps);
        let segments = colors.len() - 1;
        let steps_per_segment = steps / segments;

        for i in 0..segments {
            let start_color = colors[i];
            let end_color = colors[i + 1];
            let segment_steps = if i == segments - 1 {
                steps - i * steps_per_segment
            } else {
                steps_per_segment
            };

            for j in 0..segment_steps {
                let t = if segment_steps > 1 {
                    j as f64 / (segment_steps - 1) as f64
                } else {
                    0.0
                };
                gradient.push(Self::interpolate(start_color, end_color, t));
            }
        }

        gradient.truncate(steps);
        gradient
    }
}

/// Pre-defined color schemes
pub struct ColorSchemes;

impl ColorSchemes {
    /// Get default color scheme
    pub fn default() -> Vec<Color> {
        vec![
            Color::Red,
            Color::Yellow,
            Color::Blue,
            Color::Green,
            Color::Cyan,
            Color::Magenta,
        ]
    }

    /// Get warm color scheme
    pub fn warm() -> Vec<Color> {
        vec![
            Color::Red,
            Color::LightRed,
            Color::Yellow,
            Color::LightYellow,
            Color::Rgb(255, 165, 0), // Orange
            Color::Rgb(255, 140, 0), // Dark orange
        ]
    }

    /// Get cool color scheme
    pub fn cool() -> Vec<Color> {
        vec![
            Color::Blue,
            Color::LightBlue,
            Color::Cyan,
            Color::LightCyan,
            Color::Rgb(0, 0, 128), // Navy
            Color::Rgb(70, 130, 180), // Steel blue
        ]
    }

    /// Get pastel color scheme
    pub fn pastel() -> Vec<Color> {
        vec![
            Color::Rgb(255, 182, 193), // Light pink
            Color::Rgb(173, 216, 230), // Light blue
            Color::Rgb(144, 238, 144), // Light green
            Color::Rgb(255, 255, 153), // Light yellow
            Color::Rgb(221, 160, 221), // Plum
            Color::Rgb(240, 128, 128), // Light coral
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_color_to_light() {
        let theme = Theme::Light;
        assert_eq!(PaletteColor::Primary.to_color(&theme), Color::Blue);
        assert_eq!(PaletteColor::Success.to_color(&theme), Color::Green);
    }

    #[test]
    fn test_palette_color_to_dark() {
        let theme = Theme::Dark;
        assert_eq!(PaletteColor::Primary.to_color(&theme), Color::LightBlue);
        assert_eq!(PaletteColor::Background.to_color(&theme), Color::Black);
    }

    #[test]
    fn test_game_color_to_color() {
        assert_eq!(GameColor::Player1.to_color(), Color::Red);
        assert_eq!(GameColor::Player2.to_color(), Color::Yellow);
        assert_eq!(GameColor::Empty.to_color(), Color::Reset);
    }

    #[test]
    fn test_theme_from_str() {
        assert_eq!(Theme::from_str("light").unwrap(), Theme::Light);
        assert_eq!(Theme::from_str("DARK").unwrap(), Theme::Dark);
        assert!(Theme::from_str("unknown").is_err());
    }

    #[test]
    fn test_color_utils_darken() {
        let darkened = ColorUtils::darken(Color::White, 0.5);
        let (r, g, b) = ColorUtils::color_to_rgb(darkened);
        assert_eq!(r, 128);
        assert_eq!(g, 128);
        assert_eq!(b, 128);
    }

    #[test]
    fn test_color_utils_lighten() {
        let lightened = ColorUtils::lighten(Color::Black, 0.5);
        let (r, g, b) = ColorUtils::color_to_rgb(lightened);
        assert_eq!(r, 128);
        assert_eq!(g, 128);
        assert_eq!(b, 128);
    }

    #[test]
    fn test_color_utils_interpolate() {
        let result = ColorUtils::interpolate(Color::Black, Color::White, 0.5);
        let (r, g, b) = ColorUtils::color_to_rgb(result);
        assert_eq!(r, 128);
        assert_eq!(g, 128);
        assert_eq!(b, 128);
    }

    #[test]
    fn test_color_utils_hex() {
        assert_eq!(ColorUtils::to_hex(Color::Red), "#ff0000");
        assert_eq!(ColorUtils::to_hex(Color::White), "#ffffff");

        assert_eq!(ColorUtils::from_hex("#ff0000"), Some(Color::Red));
        assert_eq!(ColorUtils::from_hex("#ffffff"), Some(Color::White));
    }

    #[test]
    fn test_color_utils_get_contrast() {
        assert_eq!(ColorUtils::get_contrast_color(Color::Black), Color::White);
        assert_eq!(ColorUtils::get_contrast_color(Color::White), Color::Black);
    }
}
