// Rendering system for Connect Four UI
// Provides abstraction layer for rendering and display

use ratatui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
    Frame, Terminal,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::ui::{Theme, PaletteColor, StylePresets, colors::ColorUtils, layout::{FlexLayout, StackLayout, AnchorLayout, SpacingConfig}};

// ============================================================================
// Renderer Configuration
// ============================================================================

/// Configuration for the renderer
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Enable double buffering
    pub double_buffering: bool,
    /// Enable vsync
    pub vsync: bool,
    /// Target frame rate
    pub target_fps: u32,
    /// Enable animation interpolation
    pub interpolation: bool,
    /// Render scale (1.0 = normal, 2.0 = 2x)
    pub scale: f32,
    /// Enable alpha blending
    pub alpha_blending: bool,
    /// Enable anti-aliasing
    pub anti_aliasing: bool,
    /// Maximum render time before warning
    pub max_render_time: Duration,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            double_buffering: true,
            vsync: true,
            target_fps: 60,
            interpolation: true,
            scale: 1.0,
            alpha_blending: true,
            anti_aliasing: false, // Not applicable to terminal
            max_render_time: Duration::from_millis(16), // ~60fps
        }
    }
}

impl RendererConfig {
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps.max(1).min(120);
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.5).min(3.0);
        self
    }

    pub fn frame_duration(&self) -> Duration {
        Duration::from_secs_f32(1.0 / self.target_fps as f32)
    }
}

// ============================================================================
// Rendering Statistics
// ============================================================================

/// Statistics about rendering performance
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    /// Number of frames rendered
    pub frame_count: usize,
    /// Total rendering time
    pub total_render_time: Duration,
    /// Average frame time
    pub avg_frame_time: Duration,
    /// Minimum frame time
    pub min_frame_time: Duration,
    /// Maximum frame time
    pub max_frame_time: Duration,
    /// Current FPS
    pub current_fps: f32,
    /// Average FPS
    pub avg_fps: f32,
    /// Number of frame drops (over budget)
    pub frame_drops: usize,
    /// Last render time
    pub last_render_time: Option<Duration>,
}

impl RenderStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, render_time: Duration, target_frame_time: Duration) {
        self.frame_count += 1;
        self.total_render_time += render_time;
        self.last_render_time = Some(render_time);

        // Update min/max
        if self.frame_count == 1 {
            self.min_frame_time = render_time;
            self.max_frame_time = render_time;
        } else {
            self.min_frame_time = self.min_frame_time.min(render_time);
            self.max_frame_time = self.max_frame_time.max(render_time);
        }

        // Calculate average frame time
        self.avg_frame_time = self.total_render_time / self.frame_count as u32;

        // Calculate FPS
        self.current_fps = 1.0 / render_time.as_secs_f32();
        self.avg_fps = 1.0 / self.avg_frame_time.as_secs_f32();

        // Track frame drops
        if render_time > target_frame_time {
            self.frame_drops += 1;
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

// ============================================================================
// Render Layer
// ============================================================================

/// Render layer for compositing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderLayer {
    Background,
    Base,
    Content,
    Overlay,
    Popup,
    Tooltip,
    Top,
}

impl RenderLayer {
    pub const fn order(&self) -> usize {
        match self {
            Self::Background => 0,
            Self::Base => 1,
            Self::Content => 2,
            Self::Overlay => 3,
            Self::Popup => 4,
            Self::Tooltip => 5,
            Self::Top => 6,
        }
    }
}

// ============================================================================
// Render Command
// ============================================================================

/// Command to render a widget at a specific location
pub struct RenderCommand {
    /// Layer to render on
    layer: RenderLayer,
    /// Area to render in
    area: Rect,
    /// Widget to render
    widget: Box<dyn Widget>,
    /// Z-order within layer
    z_order: i32,
}

impl RenderCommand {
    pub fn new(layer: RenderLayer, area: Rect, widget: Box<dyn Widget>) -> Self {
        Self {
            layer,
            area,
            widget,
            z_order: 0,
        }
    }

    pub fn with_z_order(mut self, z_order: i32) -> Self {
        self.z_order = z_order;
        self
    }

    pub fn layer(&self) -> RenderLayer {
        self.layer
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn render(&self, buffer: &mut Buffer) {
        self.widget.render(self.area, buffer);
    }
}

impl std::cmp::Ord for RenderCommand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.layer
            .order()
            .cmp(&other.layer.order())
            .then(self.z_order.cmp(&other.z_order))
    }
}

impl std::cmp::PartialOrd for RenderCommand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Eq for RenderCommand {}
impl std::cmp::PartialEq for RenderCommand {
    fn eq(&self, other: &Self) -> bool {
        self.layer == other.layer && self.z_order == other.z_order
    }
}

// ============================================================================
// Renderer
// ============================================================================

/// Main renderer for the UI
pub struct Renderer {
    /// Configuration
    config: RendererConfig,
    /// Current theme
    theme: Arc<Theme>,
    /// Render statistics
    stats: RenderStats,
    /// Last frame time
    last_frame_time: Option<Instant>,
    /// Pending render commands
    pending_commands: Vec<RenderCommand>,
    /// Enabled layers
    enabled_layers: Vec<RenderLayer>,
    /// Animation time
    animation_time: f32,
}

impl Renderer {
    pub fn new(config: RendererConfig, theme: Arc<Theme>) -> Self {
        Self {
            config,
            theme,
            stats: RenderStats::new(),
            last_frame_time: None,
            pending_commands: Vec::new(),
            enabled_layers: vec![
                RenderLayer::Background,
                RenderLayer::Base,
                RenderLayer::Content,
                RenderLayer::Overlay,
                RenderLayer::Popup,
                RenderLayer::Tooltip,
                RenderLayer::Top,
            ],
            animation_time: 0.0,
        }
    }

    pub fn with_theme(theme: Theme) -> Self {
        Self::new(RendererConfig::default(), Arc::new(theme))
    }

    pub fn config(&self) -> &RendererConfig {
        &self.config
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, theme: Arc<Theme>) {
        self.theme = theme;
    }

    pub fn stats(&self) -> &RenderStats {
        &self.stats
    }

    pub fn add_command(&mut self, command: RenderCommand) {
        self.pending_commands.push(command);
    }

    pub fn clear_commands(&mut self) {
        self.pending_commands.clear();
    }

    pub fn enable_layer(&mut self, layer: RenderLayer) {
        if !self.enabled_layers.contains(&layer) {
            self.enabled_layers.push(layer);
            self.enabled_layers.sort_by_key(|l| l.order());
        }
    }

    pub fn disable_layer(&mut self, layer: RenderLayer) {
        self.enabled_layers.retain(|l| *l != layer);
    }

    pub fn is_layer_enabled(&self, layer: RenderLayer) -> bool {
        self.enabled_layers.contains(&layer)
    }

    pub fn update_animation(&mut self, delta: Duration) {
        self.animation_time += delta.as_secs_f32();
    }

    pub fn animation_time(&self) -> f32 {
        self.animation_time
    }

    /// Render a frame to the terminal
    pub fn render<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        f: impl FnOnce(&mut Frame),
    ) -> Result<(), std::io::Error> {
        let start_time = Instant::now();

        terminal.draw(f)?;

        let render_time = start_time.elapsed();
        let frame_time = self.config.frame_duration();

        self.stats.update(render_time, frame_time);
        self.last_frame_time = Some(start_time);

        Ok(())
    }

    /// Execute pending render commands
    pub fn execute_commands(&mut self, buffer: &mut Buffer) {
        // Sort commands by layer and z-order
        self.pending_commands.sort();

        // Execute commands for enabled layers only
        for command in &self.pending_commands {
            if self.is_layer_enabled(command.layer()) {
                command.render(buffer);
            }
        }

        self.pending_commands.clear();
    }

    /// Check if should render based on time
    pub fn should_render(&self) -> bool {
        if let Some(last) = self.last_frame_time {
            last.elapsed() >= self.config.frame_duration()
        } else {
            true
        }
    }

    /// Wait until next frame time (for frame rate limiting)
    pub fn wait_for_frame(&self) {
        if let Some(last) = self.last_frame_time {
            let elapsed = last.elapsed();
            let frame_time = self.config.frame_duration();

            if elapsed < frame_time {
                let wait_time = frame_time - elapsed;
                std::thread::sleep(wait_time);
            }
        }
    }

    /// Reset frame timing (call when starting new session)
    pub fn reset_frame_timing(&mut self) {
        self.last_frame_time = None;
    }
}

// ============================================================================
// Screen Buffer
// ============================================================================

/// Off-screen buffer for rendering
pub struct ScreenBuffer {
    /// Buffer content
    buffer: Buffer,
    /// Clear color
    clear_color: Color,
}

impl ScreenBuffer {
    pub fn new(area: Rect, clear_color: Color) -> Self {
        Self {
            buffer: Buffer::filled(area, Style::default().bg(clear_color)),
            clear_color,
        }
    }

    pub fn buffer(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub fn clear(&mut self) {
        let area = self.buffer.area;
        *self.buffer = Buffer::filled(area, Style::default().bg(self.clear_color));
    }

    pub fn area(&self) -> Rect {
        self.buffer.area
    }
}

// ============================================================================
// Render Context
// ============================================================================

/// Context passed during rendering
pub struct RenderContext {
    /// Renderer reference
    renderer: Renderer,
    /// Current area being rendered
    area: Rect,
    /// Clipping region
    clip_region: Option<Rect>,
    /// Local transform (offset)
    transform: (i16, i16),
    /// Opacity (0.0 to 1.0)
    opacity: f32,
}

impl RenderContext {
    pub fn new(renderer: Renderer, area: Rect) -> Self {
        Self {
            renderer,
            area,
            clip_region: None,
            transform: (0, 0),
            opacity: 1.0,
        }
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn with_clip(mut self, clip: Rect) -> Self {
        self.clip_region = Some(clip);
        self
    }

    pub fn with_transform(mut self, x: i16, y: i16) -> Self {
        self.transform = (x, y);
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn apply_transform(&self, rect: Rect) -> Rect {
        let x = (rect.x as i16 + self.transform.0).max(0) as u16;
        let y = (rect.y as i16 + self.transform.1).max(0) as u16;
        Rect { x, y, ..rect }
    }
}

// ============================================================================
// Debug Renderer
// ============================================================================

/// Renderer for debug information
pub struct DebugRenderer {
    /// Show frame time
    pub show_frame_time: bool,
    /// Show FPS
    pub show_fps: bool,
    /// Show render stats
    pub show_stats: bool,
    /// Show layer info
    pub show_layers: bool,
    /// Position of debug info
    pub position: DebugPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl DebugRenderer {
    pub fn new() -> Self {
        Self {
            show_frame_time: true,
            show_fps: true,
            show_stats: false,
            show_layers: false,
            position: DebugPosition::TopRight,
        }
    }

    pub fn with_position(mut self, position: DebugPosition) -> Self {
        self.position = position;
        self
    }

    pub fn render(&self, frame: &mut Frame, stats: &RenderStats, theme: &Theme) {
        let mut lines = Vec::new();

        if self.show_frame_time {
            if let Some(last) = stats.last_render_time {
                lines.push(Line::from(vec![
                    Span::styled("Frame: ", theme.get_style(PaletteColor::Muted)),
                    Span::styled(format!("{:.2}ms", last.as_secs_f32() * 1000.0),
                               theme.get_style(PaletteColor::Info)),
                ]));
            }
        }

        if self.show_fps {
            lines.push(Line::from(vec![
                Span::styled("FPS: ", theme.get_style(PaletteColor::Muted)),
                Span::styled(format!("{:.1}", stats.current_fps),
                           theme.get_style(if stats.current_fps >= 30.0 {
                               PaletteColor::Success
                           } else {
                               PaletteColor::Warning
                           })),
            ]));
        }

        if self.show_stats {
            lines.push(Line::from(vec![
                Span::styled("Avg: ", theme.get_style(PaletteColor::Muted)),
                Span::styled(format!("{:.1} FPS", stats.avg_fps),
                           theme.get_style(PaletteColor::Info)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Frames: ", theme.get_style(PaletteColor::Muted)),
                Span::styled(stats.frame_count.to_string(),
                           theme.get_style(PaletteColor::Text)),
            ]));
            if stats.frame_drops > 0 {
                lines.push(Line::from(vec![
                    Span::styled("Drops: ", theme.get_style(PaletteColor::Muted)),
                    Span::styled(stats.frame_drops.to_string(),
                               theme.get_style(PaletteColor::Error)),
                ]));
            }
        }

        if !lines.is_empty() {
            let area = match self.position {
                DebugPosition::TopLeft => Rect {
                    x: 1,
                    y: 1,
                    width: 25,
                    height: lines.len() as u16 + 2,
                },
                DebugPosition::TopRight => {
                    let size = frame.size();
                    Rect {
                        x: size.width.saturating_sub(26),
                        y: 1,
                        width: 25,
                        height: lines.len() as u16 + 2,
                    }
                }
                DebugPosition::BottomLeft => {
                    let size = frame.size();
                    Rect {
                        x: 1,
                        y: size.height.saturating_sub(lines.len() as u16 + 3),
                        width: 25,
                        height: lines.len() as u16 + 2,
                    }
                }
                DebugPosition::BottomRight => {
                    let size = frame.size();
                    Rect {
                        x: size.width.saturating_sub(26),
                        y: size.height.saturating_sub(lines.len() as u16 + 3),
                        width: 25,
                        height: lines.len() as u16 + 2,
                    }
                }
            };

            let paragraph = Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(theme.get_style(PaletteColor::Border)),
                )
                .alignment(Alignment::Left);

            paragraph.render(area, frame.buffer_mut());
        }
    }
}

impl Default for DebugRenderer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Fade Transition
// ============================================================================

/// Fade transition effect
pub struct FadeTransition {
    /// Progress (0.0 to 1.0)
    progress: f32,
    /// Direction (true = fade in, false = fade out)
    fade_in: bool,
}

impl FadeTransition {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            fade_in: true,
        }
    }

    pub fn fade_in() -> Self {
        Self {
            progress: 0.0,
            fade_in: true,
        }
    }

    pub fn fade_out() -> Self {
        Self {
            progress: 0.0,
            fade_in: false,
        }
    }

    pub fn update(&mut self, delta: Duration) -> bool {
        let fade_speed = 2.0; // Fade duration in seconds
        let delta_progress = delta.as_secs_f32() / fade_speed;

        if self.fade_in {
            self.progress = (self.progress + delta_progress).min(1.0);
        } else {
            self.progress = (self.progress + delta_progress).min(1.0);
        }

        self.is_complete()
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }

    pub fn alpha(&self) -> u8 {
        if self.fade_in {
            (self.progress * 255.0) as u8
        } else {
            ((1.0 - self.progress) * 255.0) as u8
        }
    }

    pub fn reset(&mut self) {
        self.progress = 0.0;
    }

    pub fn set_direction(&mut self, fade_in: bool) {
        self.fade_in = fade_in;
        self.progress = 0.0;
    }
}

// ============================================================================
// Rendering Utilities
// ============================================================================

/// Utility functions for rendering
pub struct RenderUtils;

impl RenderUtils {
    /// Create a centered rect within a parent
    pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    /// Draw a border around a rect
    pub fn draw_border(buffer: &mut Buffer, area: Rect, style: Style) {
        Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .render(area, buffer);
    }

    /// Draw a shadow effect
    pub fn draw_shadow(buffer: &mut Buffer, area: Rect, theme: &Theme) {
        let shadow_style = theme.get_style(PaletteColor::Background).bg(Color::Black);
        let shadow_offset = 1;

        if area.x + area.width + shadow_offset < buffer.area.width {
            for y in area.top()..area.bottom() {
                buffer.get_mut(area.right() + shadow_offset, y)
                    .set_style(shadow_style);
            }
        }

        if area.y + area.height + shadow_offset < buffer.area.height {
            for x in area.left()..area.right() + shadow_offset + 1 {
                buffer.get_mut(x, area.bottom() + shadow_offset)
                    .set_style(shadow_style);
            }
        }
    }

    /// Create a transparent overlay
    pub fn create_overlay(buffer: &mut Buffer, area: Rect, opacity: u8) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = buffer.get_mut(x, y);
                let style = cell.style();
                // Note: True alpha blending not fully supported in terminal
                // This is a simplified approximation
                cell.set_style(style);
            }
        }
    }

    /// Clip content to a region
    pub fn clip_area(area: Rect, clip: Rect) -> Option<Rect> {
        let x = area.x.max(clip.x);
        let y = area.y.max(clip.y);
        let right = (area.x + area.width).min(clip.x + clip.width);
        let bottom = (area.y + area.height).min(clip.y + clip.height);

        if x < right && y < bottom {
            Some(Rect {
                x,
                y,
                width: right - x,
                height: bottom - y,
            })
        } else {
            None
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_config_default() {
        let config = RendererConfig::default();
        assert_eq!(config.target_fps, 60);
        assert!(config.double_buffering);
        assert_eq!(config.scale, 1.0);
    }

    #[test]
    fn test_renderer_config_with_fps() {
        let config = RendererConfig::default().with_fps(30);
        assert_eq!(config.target_fps, 30);
    }

    #[test]
    fn test_renderer_config_frame_duration() {
        let config = RendererConfig::default().with_fps(60);
        let duration = config.frame_duration();
        assert_eq!(duration.as_millis(), 16);
    }

    #[test]
    fn test_render_stats() {
        let mut stats = RenderStats::new();
        stats.update(Duration::from_millis(16), Duration::from_millis(16));

        assert_eq!(stats.frame_count, 1);
        assert_eq!(stats.current_fps as u16, 60);
    }

    #[test]
    fn test_render_layer_order() {
        assert!(RenderLayer::Background.order() < RenderLayer::Content.order());
        assert!(RenderLayer::Content.order() < RenderLayer::Overlay.order());
        assert!(RenderLayer::Popup.order() < RenderLayer::Tooltip.order());
    }

    #[test]
    fn test_fade_transition() {
        let mut transition = FadeTransition::fade_in();
        assert!(!transition.is_complete());
        assert_eq!(transition.alpha(), 0);

        transition.update(Duration::from_secs(1));
        assert_eq!(transition.alpha(), 127);

        transition.update(Duration::from_secs(1));
        assert!(transition.is_complete());
        assert_eq!(transition.alpha(), 255);
    }

    #[test]
    fn test_render_utils_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = RenderUtils::centered_rect(50, 50, area);

        assert_eq!(centered.x, 25);
        assert_eq!(centered.y, 12);
        assert_eq!(centered.width, 50);
        assert_eq!(centered.height, 25);
    }

    #[test]
    fn test_render_utils_clip_area() {
        let area = Rect::new(10, 10, 50, 30);
        let clip = Rect::new(0, 0, 40, 40);

        let result = RenderUtils::clip_area(area, clip).unwrap();
        assert_eq!(result.x, 10);
        assert_eq!(result.y, 10);
        assert_eq!(result.width, 30);
        assert_eq!(result.height, 30);
    }

    #[test]
    fn test_render_utils_clip_area_no_overlap() {
        let area = Rect::new(50, 50, 10, 10);
        let clip = Rect::new(0, 0, 40, 40);

        let result = RenderUtils::clip_area(area, clip);
        assert!(result.is_none());
    }
}
