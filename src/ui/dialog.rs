// Dialog system for Connect Four UI
// Provides modal dialogs, confirmation prompts, and input dialogs

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
    Frame,
};
use std::sync::Arc;
use crate::ui::{Theme, PaletteColor};
use crate::ui::popup::{Popup, PopupPosition, PopupSize};

// ============================================================================
// Dialog Result
// ============================================================================

/// Result of a dialog interaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogResult {
    /// User confirmed/accepted
    Confirm,
    /// User cancelled/rejected
    Cancel,
    /// User selected a specific button
    Button(String),
    /// User provided input text
    Input(String),
    /// User closed without action
    Closed,
}

// ============================================================================
// Dialog Type
// ============================================================================

/// Type of dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    /// Information message
    Info,
    /// Warning message
    Warning,
    /// Error message
    Error,
    /// Confirmation prompt
    Confirmation,
    /// Input prompt
    Input,
    /// Multiple choice
    MultipleChoice,
}

// ============================================================================
// Dialog Button
// ============================================================================

/// Button in a dialog
#[derive(Debug, Clone)]
pub struct DialogButton {
    /// Button label
    pub label: String,
    /// Button ID
    pub id: String,
    /// Is this the default button
    pub is_default: bool,
    /// Is this a destructive action
    pub is_destructive: bool,
    /// Is button enabled
    pub enabled: bool,
}

impl DialogButton {
    pub fn new(label: String, id: String) -> Self {
        Self {
            label,
            id,
            is_default: false,
            is_destructive: false,
            enabled: true,
        }
    }

    pub fn with_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    pub fn with_destructive(mut self) -> Self {
        self.is_destructive = true;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

// ============================================================================
// Dialog Configuration
// ============================================================================

/// Configuration for dialogs
#[derive(Debug, Clone)]
pub struct DialogConfig {
    /// Dialog type
    pub dialog_type: DialogType,
    /// Title
    pub title: String,
    /// Message
    pub message: String,
    /// Buttons
    pub buttons: Vec<DialogButton>,
    /// Input placeholder (for input dialogs)
    pub input_placeholder: String,
    /// Input default value
    pub input_default: String,
    /// Allow multiline input
    pub multiline_input: bool,
    /// Can be dismissed with ESC
    pub dismissible: bool,
    /// Can be dismissed by clicking outside
    pub click_outside_dismiss: bool,
    /// Show close button
    pub show_close: bool,
}

impl Default for DialogConfig {
    fn default() -> Self {
        Self {
            dialog_type: DialogType::Info,
            title: String::new(),
            message: String::new(),
            buttons: Vec::new(),
            input_placeholder: String::new(),
            input_default: String::new(),
            multiline_input: false,
            dismissible: true,
            click_outside_dismiss: true,
            show_close: false,
        }
    }
}

impl DialogConfig {
    pub fn new(dialog_type: DialogType) -> Self {
        Self {
            dialog_type,
            ..Default::default()
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub fn with_buttons(mut self, buttons: Vec<DialogButton>) -> Self {
        self.buttons = buttons;
        self
    }

    pub fn with_input(mut self, placeholder: String, default: String) -> Self {
        self.input_placeholder = placeholder;
        self.input_default = default;
        self
    }

    pub fn with_multiline(mut self, multiline: bool) -> Self {
        self.multiline_input = multiline;
        self
    }

    pub fn with_dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    pub fn with_close_button(mut self, show: bool) -> Self {
        self.show_close = show;
        self
    }
}

// ============================================================================
// Dialog
// ============================================================================

/// Modal dialog widget
pub struct Dialog {
    /// Dialog ID
    id: String,
    /// Configuration
    config: DialogConfig,
    /// Theme
    theme: Arc<Theme>,
    /// Current input value
    input_value: String,
    /// Selected button index
    selected_button: Option<usize>,
    /// Input cursor position
    cursor_position: usize,
    /// Result (set when dialog closes)
    result: Option<DialogResult>,
    /// Is visible
    visible: bool,
}

impl Dialog {
    pub fn new(id: String, config: DialogConfig, theme: Arc<Theme>) -> Self {
        let selected_button = config
            .buttons
            .iter()
            .position(|b| b.is_default)
            .or_else(|| if config.buttons.is_empty() { None } else { Some(0) });

        Self {
            id,
            config,
            theme,
            input_value: config.input_default.clone(),
            selected_button,
            cursor_position: config.input_default.len(),
            result: None,
            visible: false,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.result = None;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn result(&self) -> Option<DialogResult> {
        self.result.clone()
    }

    pub fn input_value(&self) -> &str {
        &self.input_value
    }

    pub fn select_next_button(&mut self) {
        if !self.config.buttons.is_empty() {
            let count = self.config.buttons.len();
            if let Some(selected) = self.selected_button {
                self.selected_button = Some((selected + 1) % count);
            }
        }
    }

    pub fn select_previous_button(&mut self) {
        if !self.config.buttons.is_empty() {
            let count = self.config.buttons.len();
            if let Some(selected) = self.selected_button {
                self.selected_button = Some(if selected == 0 {
                    count - 1
                } else {
                    selected - 1
                });
            }
        }
    }

    pub fn confirm(&mut self) {
        if let Some(index) = self.selected_button {
            if let Some(button) = self.config.buttons.get(index) {
                if button.enabled {
                    self.result = Some(DialogResult::Button(button.id.clone()));
                    self.hide();
                }
            }
        } else if self.config.dialog_type == DialogType::Input {
            self.result = Some(DialogResult::Input(self.input_value.clone()));
            self.hide();
        } else {
            self.result = Some(DialogResult::Confirm);
            self.hide();
        }
    }

    pub fn cancel(&mut self) {
        if self.config.dismissible {
            self.result = Some(DialogResult::Cancel);
            self.hide();
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.config.dialog_type == DialogType::Input {
            self.input_value.insert(self.cursor_position, c);
            self.cursor_position += 1;
        }
    }

    pub fn delete_char(&mut self) {
        if self.config.dialog_type == DialogType::Input && self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_value.remove(self.cursor_position);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_value.len() {
            self.cursor_position += 1;
        }
    }

    pub fn render(&self, frame: &mut Frame, parent_area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate dialog area (60% width, auto height)
        let dialog_width = (parent_area.width as f32 * 0.6).max(40.0) as u16;
        let dialog_height = self.calculate_height().min(parent_area.height - 4);

        let dialog_area = Rect {
            x: parent_area.x + (parent_area.width - dialog_width) / 2,
            y: parent_area.y + (parent_area.height - dialog_height) / 2,
            width: dialog_width,
            height: dialog_height,
        };

        // Clear area behind dialog
        Clear.render(dialog_area, frame.buffer_mut());

        // Split dialog into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Title
                Constraint::Min(3),     // Message
                Constraint::Length(3), // Input (if needed)
                Constraint::Length(1), // Buttons
            ])
            .split(dialog_area);

        // Title
        if !self.config.title.is_empty() {
            let title_color = match self.config.dialog_type {
                DialogType::Info => PaletteColor::Info,
                DialogType::Warning => PaletteColor::Warning,
                DialogType::Error => PaletteColor::Error,
                _ => PaletteColor::Primary,
            };

            let title = Paragraph::new(self.config.title.as_str())
                .style(self.theme.get_style(title_color).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            title.render(chunks[0], frame.buffer_mut());
        }

        // Message
        if !self.config.message.is_empty() {
            let message_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(if self.config.dialog_type == DialogType::Input { 1 } else { 0 }),
                ])
                .split(chunks[1]);

            let message = Paragraph::new(self.config.message.as_str())
                .style(self.theme.get_style(PaletteColor::Text))
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center);
            message.render(message_chunks[0], frame.buffer_mut());

            // Input field
            if self.config.dialog_type == DialogType::Input {
                let placeholder = if self.input_value.is_empty() {
                    self.config.input_placeholder.as_str()
                } else {
                    ""
                };

                let input_text = format!(
                    "{}{}{}",
                    placeholder,
                    self.input_value,
                    " ".repeat(chunks[1].width as usize - self.input_value.len() - placeholder.len())
                );

                let input = Paragraph::new(input_text)
                    .block(Block::default().borders(Borders::ALL))
                    .style(self.theme.get_style(PaletteColor::Text))
                    .alignment(Alignment::Left);
                input.render(message_chunks[1], frame.buffer_mut());
            }
        }

        // Buttons
        self.render_buttons(frame, chunks[2]);

        // Dialog border
        let border_color = match self.config.dialog_type {
            DialogType::Warning => PaletteColor::Warning,
            DialogType::Error => PaletteColor::Error,
            _ => PaletteColor::Border,
        };

        Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.get_style(border_color).add_modifier(Modifier::BOLD))
            .render(dialog_area, frame.buffer_mut());
    }

    fn calculate_height(&self) -> u16 {
        let mut height = 6u16; // border + padding

        if !self.config.title.is_empty() {
            height += 1;
        }

        // Estimate message height
        let message_lines = self
            .config
            .message
            .chars()
            .collect::<Vec<_>>()
            .chunks(60) // approximate line width
            .count() as u16;
        height += message_lines.max(1);

        if self.config.dialog_type == DialogType::Input {
            height += 2;
        }

        height += 1; // buttons
        height
    }

    fn render_buttons(&self, frame: &mut Frame, area: Rect) {
        if self.config.buttons.is_empty() {
            return;
        }

        let button_width = (area.width / self.config.buttons.len() as u16).max(12);
        let spacing = 1;

        let mut x = area.x;
        for (index, button) in self.config.buttons.iter().enumerate() {
            let is_selected = self.selected_button == Some(index);
            let button_color = if button.is_destructive {
                PaletteColor::Error
            } else if button.is_default {
                PaletteColor::Primary
            } else {
                PaletteColor::Secondary
            };

            let button_style = self.theme.get_style(button_color)
                .add_modifier(Modifier::BOLD)
                .add_modifier(if is_selected { Modifier::REVERSED } else { Modifier::empty() });

            if !button.enabled {
                button_style.remove_modifier(Modifier::BOLD);
            }

            let button_text = format!("[ {} ]", button.label);
            let button_area = Rect {
                x,
                y: area.y,
                width: button_text.len() as u16,
                height: 1,
            };

            let button = Paragraph::new(button_text)
                .style(button_style)
                .alignment(Alignment::Center);
            button.render(button_area, frame.buffer_mut());

            x += button_area.width + spacing;
        }
    }
}

// ============================================================================
// Dialog Presets
// ============================================================================

/// Pre-configured dialogs for common scenarios
pub struct DialogPresets;

impl DialogPresets {
    pub fn info(id: String, title: String, message: String, theme: Arc<Theme>) -> Dialog {
        let config = DialogConfig::new(DialogType::Info)
            .with_title(title)
            .with_message(message)
            .with_buttons(vec![DialogButton::new("OK".to_string(), "ok".to_string()).with_default()]);

        Dialog::new(id, config, theme)
    }

    pub fn warning(id: String, title: String, message: String, theme: Arc<Theme>) -> Dialog {
        let config = DialogConfig::new(DialogType::Warning)
            .with_title(title)
            .with_message(message)
            .with_buttons(vec![DialogButton::new("OK".to_string(), "ok".to_string()).with_default()]);

        Dialog::new(id, config, theme)
    }

    pub fn error(id: String, title: String, message: String, theme: Arc<Theme>) -> Dialog {
        let config = DialogConfig::new(DialogType::Error)
            .with_title(title)
            .with_message(message)
            .with_buttons(vec![DialogButton::new("OK".to_string(), "ok".to_string()).with_default()]);

        Dialog::new(id, config, theme)
    }

    pub fn confirm(id: String, title: String, message: String, theme: Arc<Theme>) -> Dialog {
        let config = DialogConfig::new(DialogType::Confirmation)
            .with_title(title)
            .with_message(message)
            .with_buttons(vec![
                DialogButton::new("Cancel".to_string(), "cancel".to_string()),
                DialogButton::new("Confirm".to_string(), "confirm".to_string()).with_default(),
            ]);

        Dialog::new(id, config, theme)
    }

    pub fn confirm_destructive(id: String, title: String, message: String, theme: Arc<Theme>) -> Dialog {
        let config = DialogConfig::new(DialogType::Confirmation)
            .with_title(title)
            .with_message(message)
            .with_buttons(vec![
                DialogButton::new("Cancel".to_string(), "cancel".to_string()),
                DialogButton::new("Delete".to_string(), "delete".to_string())
                    .with_destructive()
                    .with_default(),
            ]);

        Dialog::new(id, config, theme)
    }

    pub fn input(id: String, title: String, message: String, theme: Arc<Theme>) -> Dialog {
        let config = DialogConfig::new(DialogType::Input)
            .with_title(title)
            .with_message(message)
            .with_input("Enter text...".to_string(), String::new());

        Dialog::new(id, config, theme)
    }

    pub fn multiple_choice(id: String, title: String, message: String, choices: Vec<String>, theme: Arc<Theme>) -> Dialog {
        let buttons: Vec<DialogButton> = choices
            .iter()
            .enumerate()
            .map(|(i, choice)| {
                DialogButton::new(choice.clone(), format!("choice_{}", i))
                    .with_default(i == 0)
            })
            .collect();

        let config = DialogConfig::new(DialogType::MultipleChoice)
            .with_title(title)
            .with_message(message)
            .with_buttons(buttons);

        Dialog::new(id, config, theme)
    }
}

// ============================================================================
// Dialog Manager
// ============================================================================

/// Manages dialog lifecycle
pub struct DialogManager {
    /// Current active dialog
    active_dialog: Option<Dialog>,
    /// Dialog history
    history: Vec<Dialog>,
    /// Theme
    theme: Arc<Theme>,
}

impl DialogManager {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            active_dialog: None,
            history: Vec::new(),
            theme,
        }
    }

    pub fn show_dialog(&mut self, dialog: Dialog) {
        if let Some(current) = self.active_dialog.take() {
            self.history.push(current);
        }
        self.active_dialog = Some(dialog);
        self.active_dialog.as_mut().unwrap().show();
    }

    pub fn close_dialog(&mut self) -> Option<DialogResult> {
        self.active_dialog.take().map(|d| d.result().unwrap_or(DialogResult::Closed))
    }

    pub fn active_dialog(&self) -> Option<&Dialog> {
        self.active_dialog.as_ref()
    }

    pub fn active_dialog_mut(&mut self) -> Option<&mut Dialog> {
        self.active_dialog.as_mut()
    }

    pub fn is_dialog_active(&self) -> bool {
        self.active_dialog.is_some()
    }

    pub fn render(&self, frame: &mut Frame, parent_area: Rect) {
        if let Some(dialog) = &self.active_dialog {
            if dialog.is_visible() {
                dialog.render(frame, parent_area);
            }
        }
    }

    pub fn clear(&mut self) {
        self.active_dialog = None;
        self.history.clear();
    }
}

impl Default for DialogManager {
    fn default() -> Self {
        Self::new(Arc::new(Theme::default()))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_button() {
        let button = DialogButton::new("OK".to_string(), "ok".to_string())
            .with_default()
            .with_destructive();

        assert_eq!(button.label, "OK");
        assert!(button.is_default);
        assert!(button.is_destructive);
    }

    #[test]
    fn test_dialog_config() {
        let config = DialogConfig::new(DialogType::Info)
            .with_title("Test".to_string())
            .with_message("Message".to_string())
            .with_dismissible(false);

        assert_eq!(config.dialog_type, DialogType::Info);
        assert_eq!(config.title, "Test");
        assert!(!config.dismissible);
    }

    #[test]
    fn test_dialog_show_hide() {
        let theme = Arc::new(Theme::default());
        let config = DialogConfig::new(DialogType::Info).with_title("Test".to_string());
        let mut dialog = Dialog::new("test".to_string(), config, theme.clone());

        assert!(!dialog.is_visible());
        assert_eq!(dialog.result(), None);

        dialog.show();
        assert!(dialog.is_visible());

        dialog.confirm();
        assert!(!dialog.is_visible());
        assert_eq!(dialog.result(), Some(DialogResult::Confirm));
    }

    fn test_dialog_button_selection() {
        let theme = Arc::new(Theme::default());
        let buttons = vec![
            DialogButton::new("Cancel".to_string(), "cancel".to_string()),
            DialogButton::new("OK".to_string(), "ok".to_string()).with_default(),
        ];
        let config = DialogConfig::new(DialogType::Confirmation)
            .with_buttons(buttons.clone());
        let mut dialog = Dialog::new("test".to_string(), config, theme.clone());

        // Default button should be selected
        assert_eq!(dialog.selected_button, Some(1));

        dialog.select_previous_button();
        assert_eq!(dialog.selected_button, Some(0));

        dialog.select_next_button();
        assert_eq!(dialog.selected_button, Some(1));
    }

    fn test_dialog_input() {
        let theme = Arc::new(Theme::default());
        let config = DialogConfig::new(DialogType::Input)
            .with_title("Input".to_string())
            .with_input("Type here".to_string(), String::new());
        let mut dialog = Dialog::new("test".to_string(), config, theme.clone());

        dialog.insert_char('H');
        dialog.insert_char('i');
        assert_eq!(dialog.input_value(), "Hi");

        dialog.delete_char();
        assert_eq!(dialog.input_value(), "H");
    }

    fn test_dialog_presets() {
        let theme = Arc::new(Theme::default());
        let info_dialog = DialogPresets::info("info".to_string(), "Info".to_string(), "Message".to_string(), theme.clone());
        let confirm_dialog = DialogPresets::confirm("confirm".to_string(), "Confirm".to_string(), "Are you sure?".to_string(), theme.clone());
        let input_dialog = DialogPresets::input("input".to_string(), "Input".to_string(), "Enter text".to_string(), theme.clone());

        assert_eq!(info_dialog.config.dialog_type, DialogType::Info);
        assert_eq!(confirm_dialog.config.dialog_type, DialogType::Confirmation);
        assert_eq!(input_dialog.config.dialog_type, DialogType::Input);
    }

    fn test_dialog_manager() {
        let theme = Arc::new(Theme::default());
        let mut manager = DialogManager::new(theme.clone());

        assert!(!manager.is_dialog_active());

        let dialog = DialogPresets::info("test".to_string(), "Info".to_string(), "Message".to_string(), theme);
        manager.show_dialog(dialog);

        assert!(manager.is_dialog_active());

        let result = manager.close_dialog();
        assert!(result.is_some());
        assert!(!manager.is_dialog_active());
    }
}
