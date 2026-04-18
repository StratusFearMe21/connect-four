// Input handling for Connect Four UI
// Provides keyboard, mouse, and custom input processing

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind, MouseButton,
};
use ratatui::layout::Rect;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::ui::Theme;

// ============================================================================
// Input Event
// ============================================================================

/// Unified input event type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    /// Keyboard event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Resize event
    Resize(u16, u16),
    /// Paste event (clipboard content)
    Paste(String),
    /// Focus gained
    FocusGained,
    /// Focus lost
    FocusLost,
    /// Custom event
    Custom(String),
    /// Tick event (for animations/timers)
    Tick,
}

impl InputEvent {
    pub fn is_key(&self) -> bool {
        matches!(self, InputEvent::Key(_))
    }

    pub fn is_mouse(&self) -> bool {
        matches!(self, InputEvent::Mouse(_))
    }

    pub fn is_resize(&self) -> bool {
        matches!(self, InputEvent::Resize(_, _))
    }

    pub fn as_key(&self) -> Option<&KeyEvent> {
        match self {
            InputEvent::Key(key) => Some(key),
            _ => None,
        }
    }

    pub fn as_mouse(&self) -> Option<&MouseEvent> {
        match self {
            InputEvent::Mouse(mouse) => Some(mouse),
            _ => None,
        }
    }

    pub fn as_resize(&self) -> Option<(u16, u16)> {
        match self {
            InputEvent::Resize(w, h) => Some((*w, *h)),
            _ => None,
        }
    }
}

impl From<Event> for InputEvent {
    fn from(event: Event) -> Self {
        match event {
            Event::Key(key) => InputEvent::Key(key),
            Event::Mouse(mouse) => InputEvent::Mouse(mouse),
            Event::Resize(width, height) => InputEvent::Resize(width, height),
            Event::Paste(content) => InputEvent::Paste(content),
            Event::FocusGained => InputEvent::FocusGained,
            Event::FocusLost => InputEvent::FocusLost,
        }
    }
}

// ============================================================================
// Input Handler
// ============================================================================

/// Handles input events and dispatches to appropriate handlers
pub struct InputHandler {
    /// Key bindings map
    key_bindings: HashMap<KeyEvent, Action>,
    /// Mouse bindings map
    mouse_bindings: HashMap<MouseEventKind, Action>,
    /// Current focus state
    focused: bool,
    /// Last input time
    last_input: Option<Instant>,
    /// Input timeout for key repeats
    repeat_timeout: Duration,
    /// Theme for visual feedback
    theme: Theme,
}

/// Action to perform on input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// No action
    None,
    /// Move selection
    Move(Direction),
    /// Confirm selection
    Confirm,
    /// Cancel/Go back
    Cancel,
    /// Show help
    Help,
    /// Quit application
    Quit,
    /// Toggle state
    Toggle,
    /// Next item
    Next,
    /// Previous item
    Previous,
    /// Scroll up
    ScrollUp,
    /// Scroll down
    ScrollDown,
    /// Scroll left
    ScrollLeft,
    /// Scroll right
    ScrollRight,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Home position
    Home,
    /// End position
    End,
    /// Delete item
    Delete,
    /// Insert item
    Insert,
    /// Edit item
    Edit,
    /// Search
    Search,
    /// Refresh
    Refresh,
    /// Custom action with ID
    Custom(String),
    /// Character input
    Char(char),
}

/// Direction for movement actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Forward,
    Backward,
}

impl InputHandler {
    pub fn new(theme: Theme) -> Self {
        let mut handler = Self {
            key_bindings: HashMap::new(),
            mouse_bindings: HashMap::new(),
            focused: true,
            last_input: None,
            repeat_timeout: Duration::from_millis(200),
            theme,
        };
        handler.setup_default_bindings();
        handler
    }

    fn setup_default_bindings(&mut self) {
        // Navigation
        self.bind_key(KeyCode::Up, Action::Move(Direction::Up));
        self.bind_key(KeyCode::Down, Action::Move(Direction::Down));
        self.bind_key(KeyCode::Left, Action::Move(Direction::Left));
        self.bind_key(KeyCode::Right, Action::Move(Direction::Right));
        self.bind_key(KeyCode::PageUp, Action::PageUp);
        self.bind_key(KeyCode::PageDown, Action::PageDown);
        self.bind_key(KeyCode::Home, Action::Home);
        self.bind_key(KeyCode::End, Action::End);

        // Selection
        self.bind_key(KeyCode::Enter, Action::Confirm);
        self.bind_key(KeyCode::Char(' '), Action::Confirm);
        self.bind_key(KeyCode::Esc, Action::Cancel);

        // Application
        self.bind_key(KeyCode::Char('q'), Action::Quit);
        self.bind_key_with_mod(KeyCode::Char('c'), KeyModifiers::CONTROL, Action::Quit);
        self.bind_key(KeyCode::F(1), Action::Help);

        // Actions
        self.bind_key(KeyCode::Tab, Action::Next);
        self.bind_key_with_mod(KeyCode::BackTab, KeyModifiers::SHIFT, Action::Previous);
        self.bind_key(KeyCode::Delete, Action::Delete);
        self.bind_key(KeyCode::Insert, Action::Insert);
        self.bind_key(KeyCode::Char('e'), Action::Edit);
        self.bind_key_with_mod(KeyCode::Char('r'), KeyModifiers::CONTROL, Action::Refresh);

        // Mouse
        self.mouse_bindings.insert(MouseEventKind::Down(MouseButton::Left), Action::Confirm);
        self.mouse_bindings.insert(MouseEventKind::Down(MouseButton::Right), Action::Cancel);
        self.mouse_bindings.insert(MouseEventKind::ScrollUp, Action::ScrollUp);
        self.mouse_bindings.insert(MouseEventKind::ScrollDown, Action::ScrollDown);
    }

    pub fn bind_key(&mut self, code: KeyCode, action: Action) {
        let key_event = KeyEvent::new(code, KeyEventKind::Press);
        self.key_bindings.insert(key_event, action);
    }

    pub fn bind_key_with_mod(&mut self, code: KeyCode, modifiers: KeyModifiers, action: Action) {
        let key_event = KeyEvent::new(code, KeyEventKind::Press).into_modifier(modifiers);
        self.key_bindings.insert(key_event, action);
    }

    pub fn bind_mouse(&mut self, kind: MouseEventKind, action: Action) {
        self.mouse_bindings.insert(kind, action);
    }

    pub fn handle(&mut self, event: InputEvent) -> Action {
        self.last_input = Some(Instant::now());

        match event {
            InputEvent::Key(key_event) => {
                // Check for character input first
                if let KeyCode::Char(c) = key_event.code {
                    let modifiers = key_event.modifiers;
                    if modifiers.is_empty() && !self.key_bindings.contains_key(&key_event) {
                        return Action::Char(c);
                    }
                }

                // Look up action in bindings
                self.key_bindings
                    .get(&key_event)
                    .cloned()
                    .unwrap_or(Action::None)
            }
            InputEvent::Mouse(mouse_event) => {
                self.mouse_bindings
                    .get(&mouse_event.kind)
                    .cloned()
                    .unwrap_or(Action::None)
            }
            InputEvent::Resize(_, _) => Action::None, // Handle resize separately
            InputEvent::FocusGained => {
                self.focused = true;
                Action::None
            }
            InputEvent::FocusLost => {
                self.focused = false;
                Action::None
            }
            _ => Action::None,
        }
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn time_since_last_input(&self) -> Option<Duration> {
        self.last_input.map(|t| t.elapsed())
    }

    pub fn should_repeat(&self) -> bool {
        self.time_since_last_input()
            .map(|t| t < self.repeat_timeout)
            .unwrap_or(false)
    }

    pub fn set_repeat_timeout(&mut self, timeout: Duration) {
        self.repeat_timeout = timeout;
    }
}

// ============================================================================
// Input Context
// ============================================================================

/// Context for input handling with state tracking
#[derive(Debug, Clone)]
pub struct InputContext {
    /// Current cursor position
    cursor_pos: (u16, u16),
    /// Selection state
    selection: Option<usize>,
    /// Multi-selection state
    multi_selection: Vec<usize>,
    /// Drag state
    drag_state: Option<DragState>,
    /// Input buffer
    buffer: String,
    /// Buffer cursor position
    buffer_cursor: usize,
    /// History stack
    history: Vec<String>,
    /// History index
    history_index: usize,
    /// Context-specific data
    context_data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct DragState {
    start_pos: (u16, u16),
    current_pos: (u16, u16),
    button: MouseButton,
}

impl InputContext {
    pub fn new() -> Self {
        Self {
            cursor_pos: (0, 0),
            selection: None,
            multi_selection: Vec::new(),
            drag_state: None,
            buffer: String::new(),
            buffer_cursor: 0,
            history: Vec::new(),
            history_index: 0,
            context_data: HashMap::new(),
        }
    }

    pub fn cursor_pos(&self) -> (u16, u16) {
        self.cursor_pos
    }

    pub fn set_cursor_pos(&mut self, x: u16, y: u16) {
        self.cursor_pos = (x, y);
    }

    pub fn selection(&self) -> Option<usize> {
        self.selection
    }

    pub fn set_selection(&mut self, index: usize) {
        self.selection = Some(index);
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn toggle_selection(&mut self, index: usize) {
        if let Some(pos) = self.multi_selection.iter().position(|&i| i == index) {
            self.multi_selection.remove(pos);
        } else {
            self.multi_selection.push(index);
        }
    }

    pub fn multi_selection(&self) -> &[usize] {
        &self.multi_selection
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_state.is_some()
    }

    pub fn start_drag(&mut self, x: u16, y: u16, button: MouseButton) {
        self.drag_state = Some(DragState {
            start_pos: (x, y),
            current_pos: (x, y),
            button,
        });
    }

    pub fn update_drag(&mut self, x: u16, y: u16) {
        if let Some(state) = &mut self.drag_state {
            state.current_pos = (x, y);
        }
    }

    pub fn end_drag(&mut self) -> Option<(u16, u16, MouseButton)> {
        self.drag_state.take().map(|s| {
            (s.start_pos.0, s.start_pos.1, s.button)
        })
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn set_buffer(&mut self, buffer: String) {
        self.buffer = buffer;
        self.buffer_cursor = self.buffer.len();
    }

    pub fn buffer_cursor(&self) -> usize {
        self.buffer_cursor
    }

    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.buffer_cursor, c);
        self.buffer_cursor += 1;
    }

    pub fn delete_char(&mut self) {
        if self.buffer_cursor > 0 {
            self.buffer_cursor -= 1;
            self.buffer.remove(self.buffer_cursor);
        }
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
        self.buffer_cursor = 0;
    }

    pub fn push_history(&mut self, entry: String) {
        self.history.push(entry);
        self.history_index = self.history.len();
    }

    pub fn history_prev(&mut self) -> Option<&str> {
        if self.history_index > 0 {
            self.history_index -= 1;
            Some(&self.history[self.history_index])
        } else {
            None
        }
    }

    pub fn history_next(&mut self) -> Option<&str> {
        if self.history_index < self.history.len() {
            self.history_index += 1;
            self.history.get(self.history_index).map(|s| s.as_str())
        } else {
            None
        }
    }

    pub fn set_context_data(&mut self, key: String, value: String) {
        self.context_data.insert(key, value);
    }

    pub fn get_context_data(&self, key: &str) -> Option<&String> {
        self.context_data.get(key)
    }
}

impl Default for InputContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Input Validator
// ============================================================================

/// Validates input based on rules
pub trait InputValidator {
    /// Validate the input string
    fn validate(&self, input: &str) -> ValidationResult;
    /// Get error message for invalid input
    fn error_message(&self) -> &str;
}

/// Result of input validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
    Partial,
}

/// Numeric input validator
pub struct NumericValidator {
    min: Option<i64>,
    max: Option<i64>,
    allow_negative: bool,
}

impl NumericValidator {
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
            allow_negative: false,
        }
    }

    pub fn with_min(mut self, min: i64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn with_max(mut self, max: i64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn allow_negative(mut self, allow: bool) -> Self {
        self.allow_negative = allow;
        self
    }
}

impl InputValidator for NumericValidator {
    fn validate(&self, input: &str) -> ValidationResult {
        if input.is_empty() {
            return ValidationResult::Partial;
        }

        // Check for valid number format
        let is_negative = input.starts_with('-');
        let num_str = if is_negative {
            if !self.allow_negative {
                return ValidationResult::Invalid("Negative numbers not allowed".to_string());
            }
            &input[1..]
        } else {
            input
        };

        if num_str.is_empty() {
            return ValidationResult::Partial;
        }

        if !num_str.chars().all(|c| c.is_ascii_digit()) {
            return ValidationResult::Invalid("Must be a valid number".to_string());
        }

        if let Ok(num) = num_str.parse::<i64>() {
            let actual_num = if is_negative { -num } else { num };

            if let Some(min) = self.min {
                if actual_num < min {
                    return ValidationResult::Invalid(format!("Must be at least {}", min));
                }
            }

            if let Some(max) = self.max {
                if actual_num > max {
                    return ValidationResult::Invalid(format!("Must be at most {}", max));
                }
            }

            ValidationResult::Valid
        } else {
            ValidationResult::Partial
        }
    }

    fn error_message(&self) -> &str {
        "Invalid numeric input"
    }
}

/// String length validator
pub struct LengthValidator {
    min: usize,
    max: usize,
}

impl LengthValidator {
    pub fn new(min: usize, max: usize) -> Self {
        assert!(min <= max);
        Self { min, max }
    }
}

impl InputValidator for LengthValidator {
    fn validate(&self, input: &str) -> ValidationResult {
        if input.len() < self.min {
            if input.is_empty() {
                ValidationResult::Partial
            } else {
                ValidationResult::Invalid(format!("Must be at least {} characters", self.min))
            }
        } else if input.len() > self.max {
            ValidationResult::Invalid(format!("Must be at most {} characters", self.max))
        } else {
            ValidationResult::Valid
        }
    }

    fn error_message(&self) -> &str {
        "Invalid length"
    }
}

/// Regex pattern validator
pub struct PatternValidator {
    pattern: regex::Regex,
    error_message: String,
}

impl PatternValidator {
    pub fn new(pattern: &str, error_message: String) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: regex::Regex::new(pattern)?,
            error_message,
        })
    }
}

impl InputValidator for PatternValidator {
    fn validate(&self, input: &str) -> ValidationResult {
        if self.pattern.is_match(input) {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid(self.error_message.clone())
        }
    }

    fn error_message(&self) -> &str {
        &self.error_message
    }
}

// ============================================================================
// Input Field
// ============================================================================

/// Input field widget with validation
pub struct InputField {
    /// Field value
    value: String,
    /// Cursor position
    cursor: usize,
    /// Maximum length
    max_length: Option<usize>,
    /// Mask input (e.g., passwords)
    masked: bool,
    /// Placeholder text
    placeholder: Option<String>,
    /// Validator
    validator: Option<Box<dyn InputValidator>>,
    /// Focused state
    focused: bool,
    /// Area
    area: Rect,
}

impl InputField {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            max_length: None,
            masked: false,
            placeholder: None,
            validator: None,
            focused: false,
            area: Rect::default(),
        }
    }

    pub fn with_value(mut self, value: String) -> Self {
        self.value = value;
        self.cursor = self.value.len();
        self
    }

    pub fn with_max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);
        self
    }

    pub fn with_mask(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    pub fn with_validator(mut self, validator: Box<dyn InputValidator>) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn with_area(mut self, area: Rect) -> Self {
        self.area = area;
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: String) {
        self.value = value;
        self.cursor = self.value.len();
    }

    pub fn insert_char(&mut self, c: char) -> bool {
        if let Some(max_len) = self.max_length {
            if self.value.len() >= max_len {
                return false;
            }
        }

        self.value.insert(self.cursor, c);
        self.cursor += 1;
        true
    }

    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.value.remove(self.cursor);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.value.len() {
            self.cursor += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.value.len();
    }

    pub fn validate(&self) -> ValidationResult {
        if let Some(validator) = &self.validator {
            validator.validate(&self.value)
        } else {
            ValidationResult::Valid
        }
    }

    pub fn is_valid(&self) -> bool {
        matches!(self.validate(), ValidationResult::Valid)
    }

    pub fn display_value(&self) -> String {
        if self.masked {
            "•".repeat(self.value.len())
        } else if self.value.is_empty() {
            self.placeholder.clone().unwrap_or_default()
        } else {
            self.value.clone()
        }
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor
    }
}

impl Default for InputField {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Key Sequence Recorder
// ============================================================================

/// Records and replays key sequences
pub struct KeySequenceRecorder {
    /// Recorded sequences
    sequences: Vec<KeySequence>,
    /// Current recording
    recording: Option<KeySequence>,
    /// Recording start time
    recording_start: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct KeySequence {
    name: String,
    keys: Vec<KeyEvent>,
    duration: Duration,
}

impl KeySequenceRecorder {
    pub fn new() -> Self {
        Self {
            sequences: Vec::new(),
            recording: None,
            recording_start: None,
        }
    }

    pub fn start_recording(&mut self, name: String) {
        self.recording = Some(KeySequence {
            name,
            keys: Vec::new(),
            duration: Duration::ZERO,
        });
        self.recording_start = Some(Instant::now());
    }

    pub fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    pub fn record_key(&mut self, key: KeyEvent) {
        if let Some(recording) = &mut self.recording {
            recording.keys.push(key);
        }
    }

    pub fn stop_recording(&mut self) -> Option<KeySequence> {
        self.recording.take().map(|mut seq| {
            if let Some(start) = self.recording_start.take() {
                seq.duration = start.elapsed();
            }
            self.sequences.push(seq.clone());
            seq
        })
    }

    pub fn cancel_recording(&mut self) {
        self.recording = None;
        self.recording_start = None;
    }

    pub fn get_sequence(&self, name: &str) -> Option<&KeySequence> {
        self.sequences.iter().find(|s| s.name == name)
    }

    pub fn list_sequences(&self) -> &[KeySequence] {
        &self.sequences
    }

    pub fn remove_sequence(&mut self, name: &str) -> bool {
        if let Some(pos) = self.sequences.iter().position(|s| s.name == name) {
            self.sequences.remove(pos);
            true
        } else {
            false
        }
    }
}

impl Default for KeySequenceRecorder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_event_conversions() {
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyEventKind::Press);
        let input_event = InputEvent::Key(key_event);
        assert!(input_event.is_key());
        assert!(!input_event.is_mouse());
        assert_eq!(input_event.as_key(), Some(&key_event));
    }

    #[test]
    fn test_input_handler_default_bindings() {
        let theme = Theme::default();
        let handler = InputHandler::new(theme);

        let up_event = KeyEvent::new(KeyCode::Up, KeyEventKind::Press);
        let action = handler.handle(InputEvent::Key(up_event));
        assert_eq!(action, Action::Move(Direction::Up));

        let enter_event = KeyEvent::new(KeyCode::Enter, KeyEventKind::Press);
        let action = handler.handle(InputEvent::Key(enter_event));
        assert_eq!(action, Action::Confirm);
    }

    #[test]
    fn test_input_context() {
        let ctx = InputContext::new();
        assert_eq!(ctx.cursor_pos(), (0, 0));
        assert!(ctx.selection().is_none());
        assert!(!ctx.is_dragging());
    }

    #[test]
    fn test_input_context_selection() {
        let mut ctx = InputContext::new();
        ctx.set_selection(5);
        assert_eq!(ctx.selection(), Some(5));

        ctx.toggle_selection(5);
        assert_eq!(ctx.multi_selection().len(), 1);

        ctx.toggle_selection(5);
        assert_eq!(ctx.multi_selection().len(), 0);
    }

    #[test]
    fn test_input_field() {
        let field = InputField::new();
        assert!(field.value().is_empty());

        let mut field = field.with_value("hello".to_string());
        assert_eq!(field.value(), "hello");
        assert_eq!(field.cursor_position(), 5);

        field.move_left();
        assert_eq!(field.cursor_position(), 4);

        field.insert_char('!');
        assert_eq!(field.value(), "hell!o");
        assert_eq!(field.cursor_position(), 5);
    }

    #[test]
    fn test_numeric_validator() {
        let validator = NumericValidator::new().with_min(0).with_max(100);

        assert_eq!(validator.validate(""), ValidationResult::Partial);
        assert_eq!(validator.validate("50"), ValidationResult::Valid);
        assert_eq!(validator.validate("-1"), ValidationResult::Invalid("Negative numbers not allowed".to_string()));
        assert_eq!(validator.validate("150"), ValidationResult::Invalid("Must be at most 100".to_string()));
    }

    #[test]
    fn test_length_validator() {
        let validator = LengthValidator::new(3, 10);

        assert_eq!(validator.validate(""), ValidationResult::Partial);
        assert_eq!(validator.validate("ab"), ValidationResult::Invalid("Must be at least 3 characters".to_string()));
        assert_eq!(validator.validate("hello"), ValidationResult::Valid);
        assert_eq!(validator.validate("12345678901"), ValidationResult::Invalid("Must be at most 10 characters".to_string()));
    }

    #[test]
    fn test_input_field_masked() {
        let field = InputField::new()
            .with_value("secret".to_string())
            .with_mask(true);

        assert_eq!(field.display_value(), "••••••");
    }

    #[test]
    fn test_input_field_placeholder() {
        let field = InputField::new()
            .with_placeholder("Enter text...".to_string());

        assert_eq!(field.display_value(), "Enter text...");

        field.set_value("hello".to_string());
        assert_eq!(field.display_value(), "hello");
    }
}
