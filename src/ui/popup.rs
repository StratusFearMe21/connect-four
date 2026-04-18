// Popup system for Connect Four UI
// Provides popup windows, tooltips, and overlay elements

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
    Frame,
};
use std::time::{Duration, Instant};
use crate::ui::{Theme, PaletteColor};
use crate::ui::renderer::RenderLayer;
use crate::ui::animation::FadeTransition;

// ============================================================================
// Popup Configuration
// ============================================================================

/// Configuration for popups
#[derive(Debug, Clone, Copy)]
pub struct PopupConfig {
    /// Border style
    pub has_border: bool,
    /// Show title
    pub show_title: bool,
    /// Auto-hide after duration
    pub auto_hide: bool,
    /// Auto-hide duration
    pub hide_duration: Duration,
    /// Can be dismissed with ESC
    pub dismissible: bool,
    /// Click outside to dismiss
    pub click_outside_dismiss: bool,
    /// Render layer
    pub layer: RenderLayer,
    /// Z-order within layer
    pub z_order: i32,
    /// Shadow effect
    pub shadow: bool,
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            has_border: true,
            show_title: false,
            auto_hide: false,
            hide_duration: Duration::from_secs(3),
            dismissible: true,
            click_outside_dismiss: true,
            layer: RenderLayer::Popup,
            z_order: 0,
            shadow: true,
        }
    }
}

impl PopupConfig {
    pub fn with_border(mut self, has_border: bool) -> Self {
        self.has_border = has_border;
        self
    }

    pub fn with_title(mut self, show_title: bool) -> Self {
        self.show_title = show_title;
        self
    }

    pub fn with_auto_hide(mut self, auto_hide: bool, duration: Duration) -> Self {
        self.auto_hide = auto_hide;
        self.hide_duration = duration;
        self
    }

    pub fn with_dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    pub fn with_layer(mut self, layer: RenderLayer, z_order: i32) -> Self {
        self.layer = layer;
        self.z_order = z_order;
        self
    }

    pub fn with_shadow(mut self, shadow: bool) -> Self {
        self.shadow = shadow;
        self
    }
}

// ============================================================================
// Popup Position
// ============================================================================

/// Position for a popup
#[derive(Debug, Clone, Copy)]
pub enum PopupPosition {
    /// Centered on screen
    Centered,
    /// At specific coordinates
    Absolute { x: u16, y: u16 },
    /// Relative to another element
    Relative { element_id: String, offset: (i16, i16) },
    /// Anchor to screen edge
    Anchored { anchor: PopupAnchor, margin: u16 },
}

/// Anchor position for popups
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

// ============================================================================
// Popup Size
// ============================================================================

/// Size specification for popups
#[derive(Debug, Clone, Copy)]
pub enum PopupSize {
    /// Fixed size
    Fixed { width: u16, height: u16 },
    /// Percentage of screen
    Percentage { width: u16, height: u16 },
    /// Auto-size based on content
    Auto { min_width: u16, max_width: u16, min_height: u16, max_height: u16 },
    /// Fit content exactly
    Content,
}

impl PopupSize {
    pub fn fixed(width: u16, height: u16) -> Self {
        Self::Fixed { width, height }
    }

    pub fn percentage(width: u16, height: u16) -> Self {
        Self::Percentage { width, height }
    }

    pub fn auto(min_width: u16, max_width: u16, min_height: u16, max_height: u16) -> Self {
        Self::Auto { min_width, max_width, min_height, max_height }
    }

    pub fn content() -> Self {
        Self::Content
    }
}

// ============================================================================
// Popup
// ============================================================================

/// Base popup widget
pub struct Popup {
    /// Popup ID
    id: String,
    /// Popup title
    title: String,
    /// Content
    content: Vec<Line<'static>>,
    /// Configuration
    config: PopupConfig,
    /// Position
    position: PopupPosition,
    /// Size
    size: PopupSize,
    /// Theme
    theme: Theme,
    /// Creation time
    created_at: Instant,
    /// Visible state
    visible: bool,
    /// Fade transition
    fade: FadeTransition,
}

impl Popup {
    pub fn new(id: String, theme: Theme) -> Self {
        Self {
            id,
            title: String::new(),
            content: Vec::new(),
            config: PopupConfig::default(),
            position: PopupPosition::Centered,
            size: PopupSize::Content,
            theme,
            created_at: Instant::now(),
            visible: false,
            fade: FadeTransition::fade_in(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn with_content(mut self, content: Vec<Line<'static>>) -> Self {
        self.content = content;
        self
    }

    pub fn with_config(mut self, config: PopupConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_position(mut self, position: PopupPosition) -> Self {
        self.position = position;
        self
    }

    pub fn with_size(mut self, size: PopupSize) -> Self {
        self.size = size;
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.fade.reset();
        self.fade.set_direction(true);
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn dismiss(&mut self) {
        self.hide();
    }

    pub fn update(&mut self, delta: Duration) -> bool {
        if !self.visible {
            return false;
        }

        // Update fade
        let _ = self.fade.update(delta);

        // Check auto-hide
        if self.config.auto_hide {
            if self.created_at.elapsed() >= self.config.hide_duration {
                self.hide();
                return true;
            }
        }

        false
    }

    pub fn render(&self, frame: &mut Frame, parent_area: Rect) {
        if !self.visible {
            return;
        }

        let area = self.calculate_area(parent_area);

        // Clear area behind popup
        Clear.render(area, frame.buffer_mut());

        // Draw shadow if enabled
        if self.config.shadow {
            self.draw_shadow(frame.buffer_mut(), area);
        }

        // Build widget
        let mut block = if self.config.has_border {
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.theme.get_style(PaletteColor::Border))
        } else {
            Block::default()
        };

        if self.config.show_title && !self.title.is_empty() {
            block = block.title(self.title.as_str());
        }

        let paragraph = Paragraph::new(self.content.clone())
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        paragraph.render(area, frame.buffer_mut());
    }

    fn calculate_area(&self, parent_area: Rect) -> Rect {
        // Calculate size
        let (width, height) = match self.size {
            PopupSize::Fixed { width, height } => (width, height),
            PopupSize::Percentage { width, height } => (
                parent_area.width * width / 100,
                parent_area.height * height / 100,
            ),
            PopupSize::Auto { min_width, max_width, min_height, max_height } => {
                let content_width = self.estimate_content_width();
                let content_height = self.content.len() as u16 + 2; // +2 for padding/border
                (
                    content_width.clamp(min_width, max_width.min(parent_area.width)),
                    content_height.clamp(min_height, max_height.min(parent_area.height)),
                )
            }
            PopupSize::Content => {
                let content_width = self.estimate_content_width();
                let content_height = self.content.len() as u16 + 2;
                (content_width, content_height)
            }
        };

        // Calculate position
        let (x, y) = match self.position {
            PopupPosition::Centered => {
                let x = parent_area.x + (parent_area.width.saturating_sub(width)) / 2;
                let y = parent_area.y + (parent_area.height.saturating_sub(height)) / 2;
                (x, y)
            }
            PopupPosition::Absolute { x, y } => {
                let x = x.min(parent_area.width.saturating_sub(width));
                let y = y.min(parent_area.height.saturating_sub(height));
                (x, y)
            }
            PopupPosition::Relative { element_id: _, offset } => {
                let x = (parent_area.x as i16 + offset.0).max(0) as u16;
                let y = (parent_area.y as i16 + offset.1).max(0) as u16;
                (x, y)
            }
            PopupPosition::Anchored { anchor, margin } => {
                match anchor {
                    PopupAnchor::TopLeft => (margin, margin),
                    PopupAnchor::TopCenter => {
                        let x = parent_area.x + (parent_area.width - width) / 2;
                        (x, margin)
                    }
                    PopupAnchor::TopRight => {
                        let x = parent_area.right() - width - margin;
                        (x, margin)
                    }
                    PopupAnchor::CenterLeft => {
                        let y = parent_area.y + (parent_area.height - height) / 2;
                        (margin, y)
                    }
                    PopupAnchor::Center => {
                        let x = parent_area.x + (parent_area.width - width) / 2;
                        let y = parent_area.y + (parent_area.height - height) / 2;
                        (x, y)
                    }
                    PopupAnchor::CenterRight => {
                        let y = parent_area.y + (parent_area.height - height) / 2;
                        (parent_area.right() - width - margin, y)
                    }
                    PopupAnchor::BottomLeft => {
                        let y = parent_area.bottom() - height - margin;
                        (margin, y)
                    }
                    PopupAnchor::BottomCenter => {
                        let x = parent_area.x + (parent_area.width - width) / 2;
                        let y = parent_area.bottom() - height - margin;
                        (x, y)
                    }
                    PopupAnchor::BottomRight => {
                        let x = parent_area.right() - width - margin;
                        let y = parent_area.bottom() - height - margin;
                        (x, y)
                    }
                }
            }
        };

        Rect { x, y, width, height }
    }

    fn estimate_content_width(&self) -> u16 {
        self.content
            .iter()
            .map(|line| line.width() as u16)
            .max()
            .unwrap_or(0)
            + 4 // padding
    }

    fn draw_shadow(&self, buffer: &mut Buffer, area: Rect) {
        let shadow_offset = 1;
        let shadow_style = self.theme.get_style(PaletteColor::Background);

        if area.right() + shadow_offset < buffer.area.width {
            for y in area.top()..area.bottom() {
                buffer.get_mut(area.right() + shadow_offset, y)
                    .set_style(shadow_style);
            }
        }

        if area.bottom() + shadow_offset < buffer.area.height {
            for x in area.left()..area.right() + shadow_offset + 1 {
                buffer.get_mut(x, area.bottom() + shadow_offset)
                    .set_style(shadow_style);
            }
        }
    }
}

// ============================================================================
// Tooltip
// ============================================================================

/// Tooltip popup for contextual help
pub struct Tooltip {
    /// Base popup
    popup: Popup,
    /// Hover delay before showing
    hover_delay: Duration,
    /// Hover start time
    hover_start: Option<Instant>,
}

impl Tooltip {
    pub fn new(id: String, theme: Theme) -> Self {
        Self {
            popup: Popup::new(id, theme)
                .with_config(
                    PopupConfig::default()
                        .with_border(true)
                        .with_auto_hide(false, Duration::ZERO)
                        .with_dismissible(false)
                        .with_layer(RenderLayer::Tooltip, 10),
                )
                .with_position(PopupPosition::Centered)
                .with_size(PopupSize::auto(20, 60, 1, 20)),
            hover_delay: Duration::from_millis(500),
            hover_start: None,
        }
    }

    pub fn with_text(mut self, text: String) -> Self {
        let content = vec![Line::from(text.as_str())];
        self.popup.content = content;
        self
    }

    pub fn with_hover_delay(mut self, delay: Duration) -> Self {
        self.hover_delay = delay;
        self
    }

    pub fn with_position(mut self, position: PopupPosition) -> Self {
        self.popup.position = position;
        self
    }

    pub fn start_hover(&mut self) {
        self.hover_start = Some(Instant::now());
    }

    pub fn end_hover(&mut self) {
        self.hover_start = None;
        self.popup.hide();
    }

    pub fn update(&mut self, delta: Duration) {
        if let Some(start) = self.hover_start {
            if start.elapsed() >= self.hover_delay {
                if !self.popup.is_visible() {
                    self.popup.show();
                }
            }
        }
        self.popup.update(delta);
    }

    pub fn render(&self, frame: &mut Frame, parent_area: Rect) {
        self.popup.render(frame, parent_area);
    }
}

// ============================================================================
// Toast Notification
// ============================================================================

/// Toast notification popup
pub struct Toast {
    /// Base popup
    popup: Popup,
    /// Toast type
    toast_type: ToastType,
}

/// Type of toast notification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Info,
    Success,
    Warning,
    Error,
}

impl Toast {
    pub fn new(id: String, theme: Theme, toast_type: ToastType) -> Self {
        Self {
            popup: Popup::new(id, theme)
                .with_config(
                    PopupConfig::default()
                        .with_border(true)
                        .with_auto_hide(true, Duration::from_secs(3))
                        .with_dismissible(true)
                        .with_layer(RenderLayer::Popup, 5),
                )
                .with_size(PopupSize::Content),
            toast_type,
        }
    }

    pub fn info(id: String, theme: Theme, message: String) -> Self {
        Self::new(id, theme, ToastType::Info)
            .with_title("Info".to_string())
            .with_message(message)
    }

    pub fn success(id: String, theme: Theme, message: String) -> Self {
        Self::new(id, theme, ToastType::Success)
            .with_title("Success".to_string())
            .with_message(message)
    }

    pub fn warning(id: String, theme: Theme, message: String) -> Self {
        Self::new(id, theme, ToastType::Warning)
            .with_title("Warning".to_string())
            .with_message(message)
    }

    pub fn error(id: String, theme: Theme, message: String) -> Self {
        Self::new(id, theme, ToastType::Error)
            .with_title("Error".to_string())
            .with_message(message)
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.popup = self.popup.with_title(title);
        self
    }

    pub fn with_message(mut self, message: String) -> Self {
        let content = vec![Line::from(message.as_str())];
        self.popup.content = content;
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.popup.config.auto_hide = true;
        self.popup.config.hide_duration = duration;
        self
    }

    pub fn show(&mut self) {
        self.popup.show();
    }

    pub fn update(&mut self, delta: Duration) -> bool {
        self.popup.update(delta)
    }

    pub fn render(&self, frame: &mut Frame, parent_area: Rect) {
        self.popup.render(frame, parent_area);
    }

    pub fn is_visible(&self) -> bool {
        self.popup.is_visible()
    }
}

// ============================================================================
// Popup Manager
// ============================================================================

/// Manages multiple popups
pub struct PopupManager {
    /// Active popups
    popups: Vec<Box<dyn PopupWidget>>,
    /// Theme
    theme: Theme,
}

/// Trait for popup widgets
pub trait PopupWidget: Send + Sync {
    fn id(&self) -> &str;
    fn is_visible(&self) -> bool;
    fn show(&mut self);
    fn hide(&mut self);
    fn update(&mut self, delta: Duration) -> bool;
    fn render(&self, frame: &mut Frame, parent_area: Rect);
    fn layer(&self) -> RenderLayer;
    fn z_order(&self) -> i32;
    fn dismissible(&self) -> bool;
}

impl PopupManager {
    pub fn new(theme: Theme) -> Self {
        Self {
            popups: Vec::new(),
            theme,
        }
    }

    pub fn add(&mut self, popup: Box<dyn PopupWidget>) {
        self.popups.push(popup);
    }

    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.popups.iter().position(|p| p.id() == id) {
            self.popups.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get(&self, id: &str) -> Option<&dyn PopupWidget> {
        self.popups.iter().find(|p| p.id() == id).map(|p| p.as_ref())
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut dyn PopupWidget> {
        self.popups.iter_mut().find(|p| p.id() == id).map(|p| p.as_mut())
    }

    pub fn update(&mut self, delta: Duration) -> Vec<String> {
        let mut completed = Vec::new();

        for popup in &mut self.popups {
            if popup.update(delta) {
                completed.push(popup.id().to_string());
            }
        }

        completed
    }

    pub fn render(&self, frame: &mut Frame, parent_area: Rect) {
        // Sort by layer and z-order
        let mut sorted_popups: Vec<_> = self.popups.iter().collect();
        sorted_popups.sort_by(|a, b| {
            a.layer()
                .order()
                .cmp(&b.layer().order())
                .then(a.z_order().cmp(&b.z_order()))
        });

        for popup in sorted_popups {
            if popup.is_visible() {
                popup.render(frame, parent_area);
            }
        }
    }

    pub fn dismiss(&mut self, id: &str) {
        if let Some(popup) = self.get_mut(id) {
            if popup.dismissible() {
                popup.hide();
            }
        }
    }

    pub fn dismiss_all(&mut self) {
        for popup in &mut self.popups {
            if popup.dismissible() {
                popup.hide();
            }
        }
    }

    pub fn clear(&mut self) {
        self.popups.clear();
    }

    pub fn count(&self) -> usize {
        self.popups.len()
    }
}

// ============================================================================
// Popup Widget Implementations
// ============================================================================

impl PopupWidget for Popup {
    fn id(&self) -> &str {
        self.id()
    }

    fn is_visible(&self) -> bool {
        self.is_visible()
    }

    fn show(&mut self) {
        self.show();
    }

    fn hide(&mut self) {
        self.hide();
    }

    fn update(&mut self, delta: Duration) -> bool {
        self.update(delta)
    }

    fn render(&self, frame: &mut Frame, parent_area: Rect) {
        self.render(frame, parent_area);
    }

    fn layer(&self) -> RenderLayer {
        self.config.layer
    }

    fn z_order(&self) -> i32 {
        self.config.z_order
    }

    fn dismissible(&self) -> bool {
        self.config.dismissible
    }
}

impl PopupWidget for Toast {
    fn id(&self) -> &str {
        self.popup.id()
    }

    fn is_visible(&self) -> bool {
        self.is_visible()
    }

    fn show(&mut self) {
        self.show();
    }

    fn hide(&mut self) {
        self.popup.hide();
    }

    fn update(&mut self, delta: Duration) -> bool {
        self.update(delta)
    }

    fn render(&self, frame: &mut Frame, parent_area: Rect) {
        self.render(frame, parent_area);
    }

    fn layer(&self) -> RenderLayer {
        self.popup.config.layer
    }

    fn z_order(&self) -> i32 {
        self.popup.config.z_order
    }

    fn dismissible(&self) -> bool {
        self.popup.config.dismissible
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_config_default() {
        let config = PopupConfig::default();
        assert!(config.has_border);
        assert!(!config.auto_hide);
        assert!(config.dismissible);
    }

    #[test]
    fn test_popup_config_builder() {
        let config = PopupConfig::default()
            .with_border(false)
            .with_title(true)
            .with_auto_hide(true, Duration::from_secs(5));

        assert!(!config.has_border);
        assert!(config.show_title);
        assert!(config.auto_hide);
    }

    #[test]
    fn test_popup_creation() {
        let theme = Theme::default();
        let popup = Popup::new("test".to_string(), theme.clone())
            .with_title("Test".to_string())
            .with_content(vec![Line::from("Hello")]);

        assert_eq!(popup.id(), "test");
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_popup_show_hide() {
        let theme = Theme::default();
        let mut popup = Popup::new("test".to_string(), theme.clone());

        assert!(!popup.is_visible());

        popup.show();
        assert!(popup.is_visible());

        popup.hide();
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_toast_creation() {
        let theme = Theme::default();
        let toast = Toast::info("test".to_string(), theme.clone(), "Hello".to_string());

        assert_eq!(toast.toast_type, ToastType::Info);
        assert!(!toast.is_visible());
    }

    #[test]
    fn test_popup_size_variants() {
        let fixed = PopupSize::fixed(100, 50);
        let percentage = PopupSize::percentage(50, 50);
        let auto = PopupSize::auto(20, 60, 1, 20);
        let content = PopupSize::content();

        assert!(matches!(fixed, PopupSize::Fixed { .. }));
        assert!(matches!(percentage, PopupSize::Percentage { .. }));
        assert!(matches!(auto, PopupSize::Auto { .. }));
        assert!(matches!(content, PopupSize::Content));
    }

    #[test]
    fn test_popup_manager() {
        let theme = Theme::default();
        let mut manager = PopupManager::new(theme);

        let popup = Popup::new("test".to_string(), Theme::default());
        manager.add(Box::new(popup));

        assert_eq!(manager.count(), 1);
        assert!(manager.get("test").is_some());

        manager.dismiss("test");
        assert!(manager.get("test").is_some()); // Still exists, just hidden

        manager.remove("test");
        assert!(manager.get("test").is_none());
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_tooltip() {
        let theme = Theme::default();
        let mut tooltip = Tooltip::new("test".to_string(), theme.clone())
            .with_text("Help text".to_string());

        assert!(!tooltip.popup.is_visible());

        tooltip.start_hover();
        tooltip.update(Duration::from_millis(100));
        assert!(!tooltip.popup.is_visible()); // Not yet

        tooltip.update(Duration::from_millis(500));
        assert!(tooltip.popup.is_visible());

        tooltip.end_hover();
        assert!(!tooltip.popup.is_visible());
    }
}
