// Status bar for Connect Four UI
// Provides status information at the bottom of the screen

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
use std::time::{Duration, Instant};
use crate::ui::{Theme, PaletteColor};
use crate::core::Player;
use crate::core::GameState;

// ============================================================================
// Status Section
// ============================================================================

/// A section in the status bar
#[derive(Debug, Clone)]
pub struct StatusSection {
    /// Section ID
    pub id: String,
    /// Section label (optional)
    pub label: Option<String>,
    /// Section content
    pub content: String,
    /// Section color
    pub color: PaletteColor,
    /// Is section visible
    pub visible: bool,
    /// Section width (Constraint)
    pub width: Constraint,
    /// Alignment within section
    pub alignment: Alignment,
}

impl StatusSection {
    pub fn new(id: String, content: String, color: PaletteColor) -> Self {
        Self {
            id,
            label: None,
            content,
            color,
            visible: true,
            width: Constraint::Min(10),
            alignment: Alignment::Left,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_width(mut self, width: Constraint) -> Self {
        self.width = width;
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

// ============================================================================
// Status Bar Configuration
// ============================================================================

/// Configuration for the status bar
#[derive(Debug, Clone)]
pub struct StatusBarConfig {
    /// Show border
    pub show_border: bool,
    /// Show separator between sections
    pub show_separators: bool,
    /// Separator character
    pub separator: String,
    /// Section padding
    pub padding: u16,
    /// Height of status bar
    pub height: u16,
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            show_border: true,
            show_separators: true,
            separator: "│".to_string(),
            padding: 1,
            height: 1,
        }
    }
}

impl StatusBarConfig {
    pub fn with_border(mut self, show: bool) -> Self {
        self.show_border = show;
        self
    }

    pub fn with_separators(mut self, show: bool) -> Self {
        self.show_separators = show;
        self
    }

    pub fn with_separator(mut self, separator: String) -> Self {
        self.separator = separator;
        self
    }

    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_height(mut self, height: u16) -> Self {
        self.height = height.max(1);
        self
    }
}

// ============================================================================
// Status Bar
// ============================================================================

/// Status bar at the bottom of the screen
pub struct StatusBar {
    /// Status sections
    sections: Vec<StatusSection>,
    /// Configuration
    config: StatusBarConfig,
    /// Theme
    theme: Theme,
    /// Last update time
    last_update: Option<Instant>,
}

impl StatusBar {
    pub fn new(config: StatusBarConfig, theme: Theme) -> Self {
        Self {
            sections: Vec::new(),
            config,
            theme,
            last_update: None,
        }
    }

    pub fn with_theme(theme: Theme) -> Self {
        Self::new(StatusBarConfig::default(), theme)
    }

    pub fn add_section(&mut self, section: StatusSection) {
        self.sections.push(section);
    }

    pub fn remove_section(&mut self, id: &str) -> bool {
        if let Some(pos) = self.sections.iter().position(|s| s.id == id) {
            self.sections.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_section(&self, id: &str) -> Option<&StatusSection> {
        self.sections.iter().find(|s| s.id == id)
    }

    pub fn get_section_mut(&mut self, id: &str) -> Option<&mut StatusSection> {
        self.sections.iter_mut().find(|s| s.id == id)
    }

    pub fn set_section_content(&mut self, id: &str, content: String) {
        if let Some(section) = self.get_section_mut(id) {
            section.set_content(content);
        }
    }

    pub fn show_section(&mut self, id: &str, visible: bool) {
        if let Some(section) = self.get_section_mut(id) {
            section.set_visible(visible);
        }
    }

    pub fn config(&self) -> &StatusBarConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut StatusBarConfig {
        &mut self.config
    }

    pub fn update(&mut self, _delta: Duration) {
        self.last_update = Some(Instant::now());
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Filter visible sections
        let visible_sections: Vec<_> = self.sections.iter()
            .filter(|s| s.visible)
            .collect();

        if visible_sections.is_empty() {
            return;
        }

        let constraints: Vec<Constraint> = visible_sections.iter()
            .map(|s| s.width)
            .collect();

        // Split area into sections
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints.as_slice())
            .split(area);

        // Render each section
        for (index, section) in visible_sections.iter().enumerate() {
            if let Some(chunk) = chunks.get(index) {
                self.render_section(frame, *chunk, section);
            }
        }

        // Render border if enabled
        if self.config.show_border {
            Block::default()
                .borders(Borders::TOP)
                .border_style(self.theme.get_style(PaletteColor::Border))
                .render(area, frame.buffer_mut());
        }
    }

    fn render_section(&self, frame: &mut Frame, area: Rect, section: &StatusSection) {
        let mut text = String::new();

        // Add label if present
        if let Some(label) = &section.label {
            text.push_str(&format!("{}: ", label));
        }

        // Add content
        text.push_str(&section.content);

        let paragraph = Paragraph::new(text)
            .style(self.theme.get_style(section.color))
            .alignment(section.alignment);

        paragraph.render(area, frame.buffer_mut());

        // Draw separator
        if self.config.show_separators {
            if area.right() < frame.area().width {
                let separator_style = self.theme.get_style(PaletteColor::Muted);
                frame.buffer_mut()
                    .get_mut(area.right(), area.y)
                    .set_symbol(&self.config.separator)
                    .set_style(separator_style);
            }
        }
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new(StatusBarConfig::default(), Theme::default())
    }
}

// ============================================================================
// Game Status Bar
// ============================================================================

/// Status bar specialized for game state
pub struct GameStatusBar {
    /// Base status bar
    bar: StatusBar,
}

impl GameStatusBar {
    pub fn new(theme: Theme) -> Self {
        let mut bar = StatusBar::with_theme(theme.clone());

        // Add default game sections
        bar.add_section(
            StatusSection::new("turn".to_string(), "Player 1's turn".to_string(), PaletteColor::Primary)
                .with_label("Turn".to_string())
                .with_width(Constraint::Percentage(30))
        );

        bar.add_section(
            StatusSection::new("status".to_string(), "In Progress".to_string(), PaletteColor::Success)
                .with_label("Status".to_string())
                .with_width(Constraint::Percentage(30))
        );

        bar.add_section(
            StatusSection::new("moves".to_string(), "0 moves".to_string(), PaletteColor::Text)
                .with_label("Moves".to_string())
                .with_width(Constraint::Percentage(20))
        );

        bar.add_section(
            StatusSection::new("time".to_string(), "00:00".to_string(), PaletteColor::Muted)
                .with_label("Time".to_string())
                .with_alignment(Alignment::Right)
                .with_width(Constraint::Percentage(20))
        );

        Self { bar }
    }

    pub fn update_turn(&mut self, player: Player) {
        let text = match player {
            Player::Player1 => "Player 1's turn",
            Player::Player2 => "Player 2's turn",
        };
        self.bar.set_section_content("turn", text.to_string());
    }

    pub fn update_game_state(&mut self, state: &GameState) {
        let (text, color) = match state {
            GameState::InProgress => ("In Progress", PaletteColor::Success),
            GameState::Won(player, _) => (
                &format!("Player {} Wins!", if *player == Player::Player1 { 1 } else { 2 }),
                PaletteColor::Primary
            ),
            GameState::Draw => ("Draw!", PaletteColor::Warning),
            GameState::Abandoned => ("Abandoned", PaletteColor::Error),
        };
        self.bar.set_section_content("status", text.to_string());
        if let Some(section) = self.bar.get_section_mut("status") {
            section.color = *color;
        }
    }

    pub fn update_moves(&mut self, count: usize) {
        self.bar.set_section_content("moves", format!("{} moves", count));
    }

    pub fn update_time(&mut self, elapsed: Duration) {
        let minutes = elapsed.as_secs() / 60;
        let seconds = elapsed.as_secs() % 60;
        self.bar.set_section_content("time", format!("{:02}:{:02}", minutes, seconds));
    }

    pub fn update_all(&mut self, player: Player, state: &GameState, moves: usize, elapsed: Duration) {
        self.update_turn(player);
        self.update_game_state(state);
        self.update_moves(moves);
        self.update_time(elapsed);
    }

    pub fn bar(&self) -> &StatusBar {
        &self.bar
    }

    pub fn bar_mut(&mut self) -> &mut StatusBar {
        &mut self.bar
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.bar.render(frame, area);
    }
}

// ============================================================================
// Editor Status Bar
// ============================================================================

/// Status bar with editor-like features
pub struct EditorStatusBar {
    /// Base status bar
    bar: StatusBar,
    /// Line number
    line: usize,
    /// Column number
    column: usize,
    /// File path
    file_path: Option<String>,
    /// Mode (normal, insert, etc.)
    mode: String,
}

impl EditorStatusBar {
    pub fn new(theme: Theme) -> Self {
        let mut bar = StatusBar::with_theme(theme.clone());

        bar.add_section(
            StatusSection::new("file".to_string(), "Untitled".to_string(), PaletteColor::Text)
                .with_width(Constraint::Percentage(40))
        );

        bar.add_section(
            StatusSection::new("mode".to_string(), "NORMAL".to_string(), PaletteColor::Primary)
                .with_alignment(Alignment::Center)
                .with_width(Constraint::Percentage(20))
        );

        bar.add_section(
            StatusSection::new("position".to_string(), "Ln 1, Col 1".to_string(), PaletteColor::Muted)
                .with_alignment(Alignment::Right)
                .with_width(Constraint::Percentage(40))
        );

        Self {
            bar,
            line: 1,
            column: 1,
            file_path: None,
            mode: "NORMAL".to_string(),
        }
    }

    pub fn set_file_path(&mut self, path: Option<String>) {
        self.file_path = path.clone();
        let display = path.unwrap_or_else(|| "Untitled".to_string());
        self.bar.set_section_content("file", display);
    }

    pub fn set_mode(&mut self, mode: String) {
        self.mode = mode.clone();
        self.bar.set_section_content("mode", mode);

        let color = if mode == "INSERT" {
            PaletteColor::Warning
        } else {
            PaletteColor::Primary
        };

        if let Some(section) = self.bar.get_section_mut("mode") {
            section.color = color;
        }
    }

    pub fn set_position(&mut self, line: usize, column: usize) {
        self.line = line;
        self.column = column;
        self.bar.set_section_content("position", format!("Ln {}, Col {}", line, column));
    }

    pub fn bar(&self) -> &StatusBar {
        &self.bar
    }

    pub fn bar_mut(&mut self) -> &mut StatusBar {
        &mut self.bar
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.bar.render(frame, area);
    }
}

// ============================================================================
// Progress Status Bar
// ============================================================================

/// Status bar with progress indicator
pub struct ProgressStatusBar {
    /// Base status bar
    bar: StatusBar,
    /// Progress (0.0 to 1.0)
    progress: f32,
    /// Progress message
    message: String,
}

impl ProgressStatusBar {
    pub fn new(theme: Theme) -> Self {
        let mut bar = StatusBar::with_theme(theme.clone());

        bar.add_section(
            StatusSection::new("message".to_string(), "Loading...".to_string(), PaletteColor::Text)
                .with_width(Constraint::Percentage(70))
        );

        bar.add_section(
            StatusSection::new("progress".to_string(), "0%".to_string(), PaletteColor::Primary)
                .with_alignment(Alignment::Right)
                .with_width(Constraint::Percentage(30))
        );

        Self {
            bar,
            progress: 0.0,
            message: "Loading...".to_string(),
        }
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        self.bar.set_section_content("progress", format!("{:.0}%", self.progress * 100.0));
    }

    pub fn set_message(&mut self, message: String) {
        self.message = message;
        self.bar.set_section_content("message", message);
    }

    pub fn update(&mut self, progress: f32, message: String) {
        self.set_progress(progress);
        self.set_message(message);
    }

    pub fn bar(&self) -> &StatusBar {
        &self.bar
    }

    pub fn bar_mut(&mut self) -> &mut StatusBar {
        &mut self.bar
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.bar.render(frame, area);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_section() {
        let section = StatusSection::new("test".to_string(), "Content".to_string(), PaletteColor::Primary)
            .with_label("Label".to_string())
            .with_alignment(Alignment::Center);

        assert_eq!(section.id, "test");
        assert_eq!(section.content, "Content");
        assert_eq!(section.label, Some("Label".to_string()));
        assert_eq!(section.alignment, Alignment::Center);
    }

    #[test]
    fn test_status_bar_config() {
        let config = StatusBarConfig::default()
            .with_border(false)
            .with_separators(true)
            .with_separator("|".to_string());

        assert!(!config.show_border);
        assert!(config.show_separators);
        assert_eq!(config.separator, "|");
    }

    #[test]
    fn test_status_bar() {
        let mut bar = StatusBar::new(StatusBarConfig::default(), Theme::default());

        bar.add_section(StatusSection::new("sec1".to_string(), "Section 1".to_string(), PaletteColor::Primary));
        bar.add_section(StatusSection::new("sec2".to_string(), "Section 2".to_string(), PaletteColor::Secondary));

        assert_eq!(bar.sections.len(), 2);
        assert!(bar.get_section("sec1").is_some());

        bar.set_section_content("sec1", "Updated".to_string());
        assert_eq!(bar.get_section("sec1").unwrap().content, "Updated");

        bar.remove_section("sec1");
        assert!(bar.get_section("sec1").is_none());
    }

    #[test]
    fn test_game_status_bar() {
        let mut bar = GameStatusBar::new(Theme::default());

        assert_eq!(bar.bar.sections.len(), 4);

        bar.update_turn(Player::Player2);
        bar.update_moves(5);

        assert_eq!(bar.bar.get_section("turn").unwrap().content, "Player 2's turn");
        assert_eq!(bar.bar.get_section("moves").unwrap().content, "5 moves");
    }

    #[test]
    fn test_editor_status_bar() {
        let mut bar = EditorStatusBar::new(Theme::default());

        assert_eq!(bar.line, 1);
        assert_eq!(bar.column, 1);
        assert_eq!(bar.mode, "NORMAL");

        bar.set_file_path(Some("test.txt".to_string()));
        bar.set_mode("INSERT".to_string());
        bar.set_position(10, 5);

        assert_eq!(bar.file_path, Some("test.txt".to_string()));
        assert_eq!(bar.mode, "INSERT");
        assert_eq!(bar.line, 10);
        assert_eq!(bar.column, 5);
    }

    #[test]
    fn test_progress_status_bar() {
        let mut bar = ProgressStatusBar::new(Theme::default());

        assert_eq!(bar.progress, 0.0);

        bar.set_progress(0.5);
        bar.set_message("Halfway there...".to_string());

        assert_eq!(bar.progress, 0.5);
        assert_eq!(bar.message, "Halfway there...");
    }

    #[test]
    fn test_status_bar_visibility() {
        let mut bar = StatusBar::new(StatusBarConfig::default(), Theme::default());

        bar.add_section(StatusSection::new("sec1".to_string(), "Visible".to_string(), PaletteColor::Primary));
        bar.add_section(StatusSection::new("sec2".to_string(), "Hidden".to_string(), PaletteColor::Secondary)
            .with_visible(false));

        assert_eq!(bar.sections.len(), 2);
        assert!(bar.get_section("sec1").unwrap().visible);
        assert!(!bar.get_section("sec2").unwrap().visible);
    }
}
