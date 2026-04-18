// ui/theme.rs - Theme management for the UI
use crate::ui::colors::{PaletteColor, Theme};
use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// Comprehensive theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UITheme {
    /// Base theme type
    pub base: Theme,
    /// Primary color
    pub primary: String,
    /// Secondary color
    pub secondary: String,
    /// Success color
    pub success: String,
    /// Warning color
    pub warning: String,
    /// Error color
    pub error: String,
    /// Background color
    pub background: String,
    /// Foreground color
    pub foreground: String,
    /// Border color
    pub border: String,
    /// Muted color
    pub muted: String,
    /// Font settings
    pub font: FontSettings,
    /// Spacing settings
    pub spacing: SpacingSettings,
    /// Animation settings
    pub animation: AnimationSettings,
}

/// Font settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSettings {
    /// Bold enabled
    pub bold: bool,
    /// Italic enabled
    pub italic: bool,
    /// Underline enabled
    pub underline: bool,
    /// Dim enabled
    pub dim: bool,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            bold: true,
            italic: false,
            underline: false,
            dim: false,
        }
    }
}

/// Spacing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingSettings {
    /// Horizontal margin
    pub margin_x: u16,
    /// Vertical margin
    pub margin_y: u16,
    /// Horizontal padding
    pub padding_x: u16,
    /// Vertical padding
    pub padding_y: u16,
    /// Spacing between elements
    pub spacing: u16,
}

impl Default for SpacingSettings {
    fn default() -> Self {
        Self {
            margin_x: 2,
            margin_y: 1,
            padding_x: 1,
            padding_y: 0,
            spacing: 1,
        }
    }
}

/// Animation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSettings {
    /// Enable animations
    pub enabled: bool,
    /// Animation speed multiplier
    pub speed: f64,
    /// Enable smooth transitions
    pub smooth: bool,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            speed: 1.0,
            smooth: true,
        }
    }
}

impl Default for UITheme {
    fn default() -> Self {
        Self {
            base: Theme::Dark,
            primary: "lightblue".to_string(),
            secondary: "cyan".to_string(),
            success: "lightgreen".to_string(),
            warning: "lightyellow".to_string(),
            error: "lightred".to_string(),
            background: "black".to_string(),
            foreground: "white".to_string(),
            border: "darkgray".to_string(),
            muted: "gray".to_string(),
            font: FontSettings::default(),
            spacing: SpacingSettings::default(),
            animation: AnimationSettings::default(),
        }
    }
}

impl UITheme {
    /// Create a new theme
    pub fn new(base: Theme) -> Self {
        let colors = Self::get_colors_for_theme(base);

        Self {
            base,
            primary: colors.primary,
            secondary: colors.secondary,
            success: colors.success,
            warning: colors.warning,
            error: colors.error,
            background: colors.background,
            foreground: colors.foreground,
            border: colors.border,
            muted: colors.muted,
            ..Default::default()
        }
    }

    /// Get colors for a theme
    fn get_colors_for_theme(theme: Theme) -> ThemeColors {
        match theme {
            Theme::Light => ThemeColors {
                primary: "blue".to_string(),
                secondary: "cyan".to_string(),
                success: "green".to_string(),
                warning: "yellow".to_string(),
                error: "red".to_string(),
                background: "white".to_string(),
                foreground: "black".to_string(),
                border: "gray".to_string(),
                muted: "darkgray".to_string(),
            },
            Theme::Dark => ThemeColors {
                primary: "lightblue".to_string(),
                secondary: "cyan".to_string(),
                success: "lightgreen".to_string(),
                warning: "lightyellow".to_string(),
                error: "lightred".to_string(),
                background: "black".to_string(),
                foreground: "white".to_string(),
                border: "darkgray".to_string(),
                muted: "gray".to_string(),
            },
            Theme::Retro => ThemeColors {
                primary: "cornflowerblue".to_string(),
                secondary: "lightskyblue".to_string(),
                success: "mediumseagreen".to_string(),
                warning: "gold".to_string(),
                error: "indianred".to_string(),
                background: "aliceblue".to_string(),
                foreground: "darkslategray".to_string(),
                border: "lightsteelblue".to_string(),
                muted: "lightslategray".to_string(),
            },
            Theme::Neon => ThemeColors {
                primary: "cyan".to_string(),
                secondary: "magenta".to_string(),
                success: "lightgreen".to_string(),
                warning: "yellow".to_string(),
                error: "lightred".to_string(),
                background: "#0a0a0a".to_string(),
                foreground: "white".to_string(),
                border: "cyan".to_string(),
                muted: "gray".to_string(),
            },
        }
    }

    /// Get primary color
    pub fn primary_color(&self) -> Color {
        self.parse_color(&self.primary)
    }

    /// Get secondary color
    pub fn secondary_color(&self) -> Color {
        self.parse_color(&self.secondary)
    }

    /// Get success color
    pub fn success_color(&self) -> Color {
        self.parse_color(&self.success)
    }

    /// Get warning color
    pub fn warning_color(&self) -> Color {
        self.parse_color(&self.warning)
    }

    /// Get error color
    pub fn error_color(&self) -> Color {
        self.parse_color(&self.error)
    }

    /// Get background color
    pub fn background_color(&self) -> Color {
        self.parse_color(&self.background)
    }

    /// Get foreground color
    pub fn foreground_color(&self) -> Color {
        self.parse_color(&self.foreground)
    }

    /// Get border color
    pub fn border_color(&self) -> Color {
        self.parse_color(&self.border)
    }

    /// Get muted color
    pub fn muted_color(&self) -> Color {
        self.parse_color(&self.muted)
    }

    /// Parse a color string
    fn parse_color(&self, color_str: &str) -> Color {
        // Try hex first
        if color_str.starts_with('#') {
            if let Some(rgb) = crate::ui::colors::ColorUtils::from_hex(color_str) {
                return rgb;
            }
        }

        // Try named colors
        match color_str.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" | "pink" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            "gray" | "grey" => Color::Gray,
            "darkgray" | "darkgrey" => Color::DarkGray,
            "lightred" => Color::LightRed,
            "lightgreen" => Color::LightGreen,
            "lightyellow" => Color::LightYellow,
            "lightblue" => Color::LightBlue,
            "lightmagenta" | "lightpink" => Color::LightMagenta,
            "lightcyan" => Color::LightCyan,
            _ => Color::Reset,
        }
    }

    /// Create a style with primary color
    pub fn primary_style(&self) -> Style {
        let mut style = Style::default().fg(self.primary_color());

        if self.font.bold {
            style = style.add_modifier(Modifier::BOLD);
        }

        style
    }

    /// Create a style with secondary color
    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.secondary_color())
    }

    /// Create a style with success color
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success_color())
    }

    /// Create a style with warning color
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning_color())
    }

    /// Create a style with error color
    pub fn error_style(&self) -> Style {
        let mut style = Style::default().fg(self.error_color());

        if self.font.bold {
            style = style.add_modifier(Modifier::BOLD);
        }

        style
    }

    /// Create a style with muted color
    pub fn muted_style(&self) -> Style {
        let mut style = Style::default().fg(self.muted_color());

        if self.font.dim {
            style = style.add_modifier(Modifier::DIM);
        }

        style
    }

    /// Create a border style
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border_color())
    }

    /// Create a background style
    pub fn background_style(&self) -> Style {
        Style::default().bg(self.background_color())
    }

    /// Create a foreground style
    pub fn foreground_style(&self) -> Style {
        Style::default().fg(self.foreground_color())
    }

    /// Save theme to file
    pub fn save(&self, path: &str) -> Result<(), String> {
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize theme: {}", e))?;

        std::fs::write(path, data)
            .map_err(|e| format!("Failed to write theme file: {}", e))?;

        Ok(())
    }

    /// Load theme from file
    pub fn load(path: &str) -> Result<Self, String> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read theme file: {}", e))?;

        serde_json::from_str(&data)
            .map_err(|e| format!("Failed to deserialize theme: {}", e))
    }
}

/// Helper struct for theme colors
struct ThemeColors {
    primary: String,
    secondary: String,
    success: String,
    warning: String,
    error: String,
    background: String,
    foreground: String,
    border: String,
    muted: String,
}

/// Theme manager for managing multiple themes
pub struct ThemeManager {
    /// Current theme
    current_theme: UITheme,
    /// Available themes
    themes: Vec<UITheme>,
    /// Theme presets
    presets: Vec<UITheme>,
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new() -> Self {
        let presets = Self::create_presets();

        Self {
            current_theme: presets[0].clone(),
            themes: presets.clone(),
            presets,
        }
    }

    /// Create theme presets
    fn create_presets() -> Vec<UITheme> {
        vec![
            UITheme::new(Theme::Light),
            UITheme::new(Theme::Dark),
            UITheme::new(Theme::Retro),
            UITheme::new(Theme::Neon),
        ]
    }

    /// Get current theme
    pub fn current_theme(&self) -> &UITheme {
        &self.current_theme
    }

    /// Set theme by index
    pub fn set_theme(&mut self, index: usize) -> Result<(), String> {
        if index < self.themes.len() {
            self.current_theme = self.themes[index].clone();
            Ok(())
        } else {
            Err(format!("Theme index {} out of range", index))
        }
    }

    /// Set theme by name
    pub fn set_theme_by_name(&mut self, name: &str) -> Result<(), String> {
        let theme = Theme::from_str(name)
            .map_err(|_| format!("Unknown theme: {}", name))?;

        let theme_obj = UITheme::new(theme);
        self.current_theme = theme_obj;
        Ok(())
    }

    /// Cycle to next theme
    pub fn next_theme(&mut self) {
        let current_name = self.current_theme.base;
        let next_theme = match current_name {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Retro,
            Theme::Retro => Theme::Neon,
            Theme::Neon => Theme::Light,
        };

        self.current_theme = UITheme::new(next_theme);
    }

    /// Cycle to previous theme
    pub fn previous_theme(&mut self) {
        let current_name = self.current_theme.base;
        let prev_theme = match current_name {
            Theme::Light => Theme::Neon,
            Theme::Dark => Theme::Light,
            Theme::Retro => Theme::Dark,
            Theme::Neon => Theme::Retro,
        };

        self.current_theme = UITheme::new(prev_theme);
    }

    /// Get available themes
    pub fn available_themes(&self) -> &[UITheme] {
        &self.themes
    }

    /// Add a custom theme
    pub fn add_theme(&mut self, theme: UITheme) {
        self.themes.push(theme);
    }

    /// Get current theme name
    pub fn theme_name(&self) -> String {
        self.current_theme.base.to_string()
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom theme builder
pub struct ThemeBuilder {
    theme: UITheme,
}

impl ThemeBuilder {
    /// Create a new theme builder
    pub fn new() -> Self {
        Self {
            theme: UITheme::default(),
        }
    }

    /// Set base theme
    pub fn base(mut self, theme: Theme) -> Self {
        self.theme.base = theme;
        self
    }

    /// Set primary color
    pub fn primary(mut self, color: &str) -> Self {
        self.theme.primary = color.to_string();
        self
    }

    /// Set secondary color
    pub fn secondary(mut self, color: &str) -> Self {
        self.theme.secondary = color.to_string();
        self
    }

    /// Set success color
    pub fn success(mut self, color: &str) -> Self {
        self.theme.success = color.to_string();
        self
    }

    /// Set warning color
    pub fn warning(mut self, color: &str) -> Self {
        self.theme.warning = color.to_string();
        self
    }

    /// Set error color
    pub fn error(mut self, color: &str) -> Self {
        self.theme.error = color.to_string();
        self
    }

    /// Set background color
    pub fn background(mut self, color: &str) -> Self {
        self.theme.background = color.to_string();
        self
    }

    /// Set foreground color
    pub fn foreground(mut self, color: &str) -> Self {
        self.theme.foreground = color.to_string();
        self
    }

    /// Set border color
    pub fn border(mut self, color: &str) -> Self {
        self.theme.border = color.to_string();
        self
    }

    /// Set muted color
    pub fn muted(mut self, color: &str) -> Self {
        self.theme.muted = color.to_string();
        self
    }

    /// Enable bold
    pub fn bold(mut self, enabled: bool) -> Self {
        self.theme.font.bold = enabled;
        self
    }

    /// Enable italic
    pub fn italic(mut self, enabled: bool) -> Self {
        self.theme.font.italic = enabled;
        self
    }

    /// Set horizontal margin
    pub fn margin_x(mut self, margin: u16) -> Self {
        self.theme.spacing.margin_x = margin;
        self
    }

    /// Set vertical margin
    pub fn margin_y(mut self, margin: u16) -> Self {
        self.theme.spacing.margin_y = margin;
        self
    }

    /// Enable animations
    pub fn animations(mut self, enabled: bool) -> Self {
        self.theme.animation.enabled = enabled;
        self
    }

    /// Set animation speed
    pub fn animation_speed(mut self, speed: f64) -> Self {
        self.theme.animation.speed = speed;
        self
    }

    /// Build the theme
    pub fn build(self) -> UITheme {
        self.theme
    }
}

impl Default for ThemeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_theme_default() {
        let theme = UITheme::default();
        assert_eq!(theme.base, Theme::Dark);
    }

    #[test]
    fn test_ui_theme_new() {
        let theme = UITheme::new(Theme::Light);
        assert_eq!(theme.base, Theme::Light);
        assert_eq!(theme.background, "white");
    }

    #[test]
    fn test_theme_colors() {
        let theme = UITheme::new(Theme::Dark);

        let primary = theme.primary_color();
        let background = theme.background_color();

        assert_eq!(primary, Color::LightBlue);
        assert_eq!(background, Color::Black);
    }

    #[test]
    fn test_theme_styles() {
        let theme = UITheme::new(Theme::Dark);

        let primary = theme.primary_style();
        let error = theme.error_style();

        assert_eq!(primary.fg, Some(Color::LightBlue));
        assert_eq!(error.fg, Some(Color::LightRed));
    }

    #[test]
    fn test_theme_manager_new() {
        let manager = ThemeManager::new();
        assert_eq!(manager.available_themes().len(), 4);
    }

    #[test]
    fn test_theme_manager_next() {
        let mut manager = ThemeManager::new();
        let initial = manager.theme_name();

        manager.next_theme();
        let next = manager.theme_name();

        assert_ne!(initial, next);
    }

    #[test]
    fn test_theme_builder() {
        let theme = ThemeBuilder::new()
            .base(Theme::Light)
            .primary("blue")
            .background("white")
            .bold(true)
            .build();

        assert_eq!(theme.base, Theme::Light);
        assert_eq!(theme.primary, "blue");
        assert!(theme.font.bold);
    }
}
