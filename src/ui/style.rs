// ui/style.rs - Style utilities and presets for UI components
use ratatui::style::{Color, Modifier, Style};
use crate::ui::theme::UITheme;
use crate::ui::colors::PaletteColor;

/// Style presets for common UI elements
pub struct StylePresets;

impl StylePresets {
    /// Default text style
    pub fn default(theme: &UITheme) -> Style {
        Style::default().fg(theme.foreground_color())
    }

    /// Primary action style
    pub fn primary(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.primary_color());
        if theme.font.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        style
    }

    /// Secondary action style
    pub fn secondary(theme: &UITheme) -> Style {
        Style::default().fg(theme.secondary_color())
    }

    /// Success text style
    pub fn success(theme: &UITheme) -> Style {
        Style::default().fg(theme.success_color())
    }

    /// Warning text style
    pub fn warning(theme: &UITheme) -> Style {
        Style::default().fg(theme.warning_color())
    }

    /// Error text style
    pub fn error(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.error_color());
        if theme.font.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        style
    }

    /// Muted text style
    pub fn muted(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.muted_color());
        if theme.font.dim {
            style = style.add_modifier(Modifier::DIM);
        }
        style
    }

    /// Header style
    pub fn header(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.primary_color());
        style = style.add_modifier(Modifier::BOLD);
        if theme.font.underline {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        style
    }

    /// Title style
    pub fn title(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.primary_color());
        style = style.add_modifier(Modifier::BOLD);
        style
    }

    /// Subtitle style
    pub fn subtitle(theme: &UITheme) -> Style {
        Style::default().fg(theme.secondary_color())
    }

    /// Selected item style
    pub fn selected(theme: &UITheme) -> Style {
        let mut style = Style::default()
            .fg(theme.background_color())
            .bg(theme.primary_color());
        if theme.font.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        style
    }

    /// Disabled item style
    pub fn disabled(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.muted_color());
        style = style.add_modifier(Modifier::DIM);
        style
    }

    /// Border style
    pub fn border(theme: &UITheme) -> Style {
        Style::default().fg(theme.border_color())
    }

    /// Focused border style
    pub fn focused_border(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.primary_color());
        style = style.add_modifier(Modifier::BOLD);
        style
    }

    /// Active item style
    pub fn active(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.primary_color());
        style = style.add_modifier(Modifier::BOLD);
        if theme.font.underline {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        style
    }

    /// Inactive item style
    pub fn inactive(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.foreground_color());
        style = style.add_modifier(Modifier::DIM);
        style
    }

    /// Highlight style
    pub fn highlight(theme: &UITheme) -> Style {
        let mut style = Style::default()
            .fg(theme.background_color())
            .bg(theme.secondary_color());
        if theme.font.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        style
    }

    /// Label style
    pub fn label(theme: &UITheme) -> Style {
        Style::default().fg(theme.muted_color())
    }

    /// Value style
    pub fn value(theme: &UITheme) -> Style {
        Style::default().fg(theme.foreground_color())
    }

    /// Code style
    pub fn code(theme: &UITheme) -> Style {
        Style::default().fg(theme.secondary_color())
    }

    /// Link style
    pub fn link(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.primary_color());
        if theme.font.underline {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        style
    }

    /// Important style
    pub fn important(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.warning_color());
        if theme.font.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        style
    }

    /// Critical style
    pub fn critical(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.error_color());
        style = style.add_modifier(Modifier::BOLD);
        style
    }
}

/// Style builder for creating custom styles
pub struct StyleBuilder {
    fg: Option<Color>,
    bg: Option<Color>,
    modifiers: Vec<Modifier>,
}

impl StyleBuilder {
    /// Create a new style builder
    pub fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            modifiers: Vec::new(),
        }
    }

    /// Set foreground color
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set background color
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Add bold modifier
    pub fn bold(mut self) -> Self {
        self.modifiers.push(Modifier::BOLD);
        self
    }

    /// Add italic modifier
    pub fn italic(mut self) -> Self {
        self.modifiers.push(Modifier::ITALIC);
        self
    }

    /// Add underline modifier
    pub fn underline(mut self) -> Self {
        self.modifiers.push(Modifier::UNDERLINED);
        self
    }

    /// Add dim modifier
    pub fn dim(mut self) -> Self {
        self.modifiers.push(Modifier::DIM);
        self
    }

    /// Add reversed modifier
    pub fn reversed(mut self) -> Self {
        self.modifiers.push(Modifier::REVERSED);
        self
    }

    /// Build the style
    pub fn build(self) -> Style {
        let mut style = Style::default();

        if let Some(fg) = self.fg {
            style = style.fg(fg);
        }

        if let Some(bg) = self.bg {
            style = style.bg(bg);
        }

        for modifier in self.modifiers {
            style = style.add_modifier(modifier);
        }

        style
    }
}

impl Default for StyleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Style utilities for common operations
pub struct StyleUtils;

impl StyleUtils {
    /// Combine two styles
    pub fn combine(style1: Style, style2: Style) -> Style {
        let mut result = Style::default();

        // Use style1's colors, fallback to style2
        result = result.fg(style1.fg.unwrap_or(style2.fg.unwrap_or(Color::Reset)));
        result = result.bg(style1.bg.unwrap_or(style2.bg.unwrap_or(Color::Reset)));

        // Combine modifiers
        for modifier in style1.add_modifier {
            result = result.add_modifier(modifier);
        }
        for modifier in style2.add_modifier {
            result = result.add_modifier(modifier);
        }

        result
    }

    /// Add modifiers to a style
    pub fn add_modifiers(style: Style, modifiers: &[Modifier]) -> Style {
        let mut result = style;
        for &modifier in modifiers {
            result = result.add_modifier(modifier);
        }
        result
    }

    /// Remove modifiers from a style
    pub fn remove_modifiers(style: Style, modifiers: &[Modifier]) -> Style {
        let mut result = style;
        for &modifier in modifiers {
            result = result.remove_modifier(modifier);
        }
        result
    }

    /// Toggle a modifier on a style
    pub fn toggle_modifier(style: Style, modifier: Modifier) -> Style {
        if style.add_modifier.contains(&modifier) {
            style.remove_modifier(modifier)
        } else {
            style.add_modifier(modifier)
        }
    }

    /// Check if a style has a modifier
    pub fn has_modifier(style: &Style, modifier: Modifier) -> bool {
        style.add_modifier.contains(&modifier)
    }

    /// Reset a style to default
    pub fn reset() -> Style {
        Style::default()
    }

    /// Invert a style's colors
    pub fn invert(style: Style) -> Style {
        let fg = style.fg.unwrap_or(Color::Reset);
        let bg = style.bg.unwrap_or(Color::Reset);

        Style::default()
            .fg(bg)
            .bg(fg)
            .add_modifier
            .iter()
            .fold(Style::default().fg(bg).bg(fg), |acc, &m| acc.add_modifier(m))
    }

    /// Apply opacity (dimming)
    pub fn apply_opacity(style: Style, opacity: f64) -> Style {
        if opacity < 1.0 {
            style.add_modifier(Modifier::DIM)
        } else {
            style
        }
    }
}

/// Color style combinations for semantic meaning
pub struct SemanticStyles;

impl SemanticStyles {
    /// Information style
    pub fn info(theme: &UITheme) -> Style {
        Style::default().fg(PaletteColor::Info.to_color(&theme.base))
    }

    /// Tip style
    pub fn tip(theme: &UITheme) -> Style {
        Style::default().fg(theme.secondary_color())
    }

    /// Note style
    pub fn note(theme: &UITheme) -> Style {
        Style::default().fg(theme.muted_color())
    }

    /// Example style
    pub fn example(theme: &UITheme) -> Style {
        Style::default().fg(theme.primary_color())
    }

    /// Quote style
    pub fn quote(theme: &UITheme) -> Style {
        let mut style = Style::default().fg(theme.muted_color());
        style = style.add_modifier(Modifier::ITALIC);
        style
    }

    /// Citation style
    pub fn citation(theme: &UITheme) -> Style {
        Style::default().fg(theme.secondary_color())
    }
}

/// Interactive element styles
pub struct InteractiveStyles;

impl InteractiveStyles {
    /// Button normal style
    pub fn button_normal(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.primary_color())
            .bg(theme.background_color())
    }

    /// Button hovered style
    pub fn button_hovered(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.background_color())
            .bg(theme.primary_color())
    }

    /// Button pressed style
    pub fn button_pressed(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.background_color())
            .bg(theme.secondary_color())
    }

    /// Button disabled style
    pub fn button_disabled(theme: &UITheme) -> Style {
        let mut style = Style::default()
            .fg(theme.muted_color());
        style = style.add_modifier(Modifier::DIM);
        style
    }

    /// Input field normal style
    pub fn input_normal(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.foreground_color())
            .bg(theme.background_color())
    }

    /// Input field focused style
    pub fn input_focused(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.foreground_color())
            .bg(theme.background_color())
    }

    /// Input field error style
    pub fn input_error(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.error_color())
            .bg(theme.background_color())
    }

    /// Checkbox checked style
    pub fn checkbox_checked(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.success_color())
    }

    /// Checkbox unchecked style
    pub fn checkbox_unchecked(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.muted_color())
    }

    /// Radio selected style
    pub fn radio_selected(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.primary_color())
    }

    /// Radio unselected style
    pub fn radio_unselected(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.muted_color())
    }

    /// Toggle on style
    pub fn toggle_on(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.success_color())
    }

    /// Toggle off style
    pub fn toggle_off(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.muted_color())
    }
}

/// Progress and status styles
pub struct ProgressStyles;

impl ProgressStyles {
    /// Progress bar background style
    pub fn progress_background(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.muted_color())
            .bg(theme.background_color())
    }

    /// Progress bar fill style
    pub fn progress_fill(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.primary_color())
    }

    /// Progress bar complete style
    pub fn progress_complete(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.success_color())
    }

    /// Spinner style
    pub fn spinner(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.primary_color())
    }

    /// Status indicator success style
    pub fn status_success(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.success_color())
    }

    /// Status indicator warning style
    pub fn status_warning(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.warning_color())
    }

    /// Status indicator error style
    pub fn status_error(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.error_color())
    }

    /// Status indicator info style
    pub fn status_info(theme: &UITheme) -> Style {
        Style::default()
            .fg(PaletteColor::Info.to_color(&theme.base))
    }

    /// Badge style
    pub fn badge(theme: &UITheme) -> Style {
        Style::default()
            .fg(theme.background_color())
            .bg(theme.primary_color())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_presets_primary() {
        let theme = UITheme::new(crate::ui::colors::Theme::Dark);
        let style = StylePresets::primary(&theme);

        assert_eq!(style.fg, Some(Color::LightBlue));
        assert!(style.add_modifier.contains(&Modifier::BOLD));
    }

    #[test]
    fn test_style_builder() {
        let style = StyleBuilder::new()
            .fg(Color::Red)
            .bg(Color::White)
            .bold()
            .underline()
            .build();

        assert_eq!(style.fg, Some(Color::Red));
        assert_eq!(style.bg, Some(Color::White));
        assert!(style.add_modifier.contains(&Modifier::BOLD));
        assert!(style.add_modifier.contains(&Modifier::UNDERLINED));
    }

    #[test]
    fn test_style_utils_toggle() {
        let style = Style::default().add_modifier(Modifier::BOLD);
        let toggled = StyleUtils::toggle_modifier(style, Modifier::BOLD);

        assert!(!toggled.add_modifier.contains(&Modifier::BOLD));
    }

    #[test]
    fn test_interactive_styles_button_hovered() {
        let theme = UITheme::new(crate::ui::colors::Theme::Dark);
        let style = InteractiveStyles::button_hovered(&theme);

        assert_eq!(style.fg, Some(Color::Black));
        assert_eq!(style.bg, Some(Color::LightBlue));
    }

    #[test]
    fn test_progress_styles_complete() {
        let theme = UITheme::new(crate::ui::colors::Theme::Dark);
        let style = ProgressStyles::progress_complete(&theme);

        assert_eq!(style.fg, Some(Color::LightGreen));
    }
}
