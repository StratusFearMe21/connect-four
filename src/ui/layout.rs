// Layout management for Connect Four UI
// Provides flexible layout managers and responsive design

use ratatui::{
    layout::{
        Alignment, Constraint, Direction, Flex, Layout, Margin, Rect,
    },
    widgets::Widget,
};
use std::collections::HashMap;
use std::time::Duration;

use crate::ui::{Theme, PaletteColor};

// ============================================================================
// Layout Configuration
// ============================================================================

/// Configuration for layout spacing
#[derive(Debug, Clone, Copy)]
pub struct SpacingConfig {
    /// Horizontal margin
    pub margin_horizontal: u16,
    /// Vertical margin
    pub margin_vertical: u16,
    /// Horizontal padding between elements
    pub padding_horizontal: u16,
    /// Vertical padding between elements
    pub padding_vertical: u16,
    /// Gap between flex items
    pub gap: u16,
}

impl Default for SpacingConfig {
    fn default() -> Self {
        Self {
            margin_horizontal: 2,
            margin_vertical: 1,
            padding_horizontal: 1,
            padding_vertical: 1,
            gap: 1,
        }
    }
}

impl SpacingConfig {
    pub fn new(margin: (u16, u16), padding: (u16, u16), gap: u16) -> Self {
        Self {
            margin_horizontal: margin.0,
            margin_vertical: margin.1,
            padding_horizontal: padding.0,
            padding_vertical: padding.1,
            gap,
        }
    }

    pub fn with_margin(mut self, horizontal: u16, vertical: u16) -> Self {
        self.margin_horizontal = horizontal;
        self.margin_vertical = vertical;
        self
    }

    pub fn with_padding(mut self, horizontal: u16, vertical: u16) -> Self {
        self.padding_horizontal = horizontal;
        self.padding_vertical = vertical;
        self
    }

    pub fn with_gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    /// Apply margin to a Rect
    pub fn apply_margin(&self, area: Rect) -> Rect {
        area.inner(&Margin {
            horizontal: self.margin_horizontal,
            vertical: self.margin_vertical,
        })
    }

    /// Apply padding to a Rect
    pub fn apply_padding(&self, area: Rect) -> Rect {
        area.inner(&Margin {
            horizontal: self.padding_horizontal,
            vertical: self.padding_vertical,
        })
    }
}

// ============================================================================
// Flex Layout
// ============================================================================

/// Flexible layout manager for dynamic distribution
pub struct FlexLayout {
    /// Direction of flex items
    direction: Direction,
    /// Flex behavior
    flex_mode: FlexMode,
    /// Alignment of items
    alignment: Alignment,
    /// Spacing configuration
    spacing: SpacingConfig,
    /// Constraints for items
    constraints: Vec<Constraint>,
}

/// Flex layout mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexMode {
    /// All items share available space equally
    Equal,
    /// Items sized according to their natural size, with remaining space distributed
    Natural,
    /// Items sized according to flex factors
    Weighted,
    /// Pack items tightly without expansion
    Tight,
}

impl FlexLayout {
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            flex_mode: FlexMode::Equal,
            alignment: Alignment::Center,
            spacing: SpacingConfig::default(),
            constraints: Vec::new(),
        }
    }

    pub fn horizontal() -> Self {
        Self::new(Direction::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new(Direction::Vertical)
    }

    pub fn with_flex_mode(mut self, mode: FlexMode) -> Self {
        self.flex_mode = mode;
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_spacing(mut self, spacing: SpacingConfig) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn with_constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn add_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn calculate_layout(&self, area: Rect) -> Vec<Rect> {
        let inner = self.spacing.apply_margin(area);
        let num_items = self.constraints.len().max(1);

        match self.flex_mode {
            FlexMode::Equal => {
                let size = match self.direction {
                    Direction::Horizontal => (inner.width / num_items as u16).max(1),
                    Direction::Vertical => (inner.height / num_items as u16).max(1),
                };
                let mut rects = Vec::with_capacity(num_items);
                for i in 0..num_items {
                    let pos = match self.direction {
                        Direction::Horizontal => inner.left() + i as u16 * size,
                        Direction::Vertical => inner.top() + i as u16 * size,
                    };
                    rects.push(Rect {
                        x: pos,
                        y: inner.y,
                        width: if self.direction == Direction::Horizontal { size } else { inner.width },
                        height: if self.direction == Direction::Vertical { size } else { inner.height },
                    });
                }
                rects
            }
            FlexMode::Natural | FlexMode::Weighted => {
                if self.constraints.is_empty() {
                    vec![inner]
                } else {
                    Layout::default()
                        .direction(self.direction)
                        .constraints(self.constraints.clone())
                        .flex(match self.flex_mode {
                            FlexMode::Weighted => Flex::SpaceBetween,
                            _ => Flex::Legacy,
                        })
                        .spacing(self.spacing.gap)
                        .horizontal_margin(0)
                        .vertical_margin(0)
                        .split(inner)
                }
            }
            FlexMode::Tight => {
                let mut rects = Vec::with_capacity(num_items);
                let total_gap = (num_items.saturating_sub(1)) as u16 * self.spacing.gap;
                let available = match self.direction {
                    Direction::Horizontal => inner.width.saturating_sub(total_gap),
                    Direction::Vertical => inner.height.saturating_sub(total_gap),
                };

                let mut pos = match self.direction {
                    Direction::Horizontal => inner.x,
                    Direction::Vertical => inner.y,
                };

                for constraint in &self.constraints {
                    let size = match *constraint {
                        Constraint::Length(n) => n.min(available),
                        Constraint::Percentage(p) => (available * p / 100).max(1),
                        Constraint::Ratio(n, d) => (available * n as u16 / d as u16).max(1),
                        Constraint::Min(n) => n.min(available),
                        Constraint::Max(n) => n.min(available),
                    };

                    rects.push(Rect {
                        x: pos,
                        y: inner.y,
                        width: if self.direction == Direction::Horizontal { size } else { inner.width },
                        height: if self.direction == Direction::Vertical { size } else { inner.height },
                    });

                    pos += size + self.spacing.gap;
                }
                rects
            }
        }
    }
}

// ============================================================================
// Grid Layout
// ============================================================================

/// Grid layout manager for 2D layouts
pub struct GridLayout {
    /// Number of columns
    columns: usize,
    /// Number of rows (0 = auto)
    rows: usize,
    /// Column constraints
    column_constraints: Vec<Constraint>,
    /// Row constraints
    row_constraints: Vec<Constraint>,
    /// Cell spacing
    spacing: SpacingConfig,
    /// Auto-flow direction
    flow: GridFlow,
}

/// Grid auto-flow direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridFlow {
    Row,
    Column,
    Dense,
}

impl GridLayout {
    pub fn new(columns: usize) -> Self {
        Self {
            columns: columns.max(1),
            rows: 0,
            column_constraints: Vec::new(),
            row_constraints: Vec::new(),
            spacing: SpacingConfig::default(),
            flow: GridFlow::Row,
        }
    }

    pub fn with_rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    pub fn with_column_constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.column_constraints = constraints;
        self
    }

    pub fn with_row_constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.row_constraints = constraints;
        self
    }

    pub fn with_spacing(mut self, spacing: SpacingConfig) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn with_flow(mut self, flow: GridFlow) -> Self {
        self.flow = flow;
        self
    }

    pub fn calculate_layout(&self, area: Rect, num_items: usize) -> Vec<Rect> {
        let inner = self.spacing.apply_margin(area);

        // Determine number of rows
        let rows = if self.rows > 0 {
            self.rows
        } else {
            (num_items + self.columns - 1) / self.columns
        };

        // Calculate cell dimensions
        let total_h_gap = (self.columns - 1) as u16 * self.spacing.gap;
        let total_v_gap = (rows - 1) as u16 * self.spacing.gap;

        let available_width = inner.width.saturating_sub(total_h_gap);
        let available_height = inner.height.saturating_sub(total_v_gap);

        let cell_width = if self.column_constraints.is_empty() {
            available_width / self.columns as u16
        } else {
            // Simple equal division for now
            available_width / self.columns as u16
        };

        let cell_height = if self.row_constraints.is_empty() {
            available_height / rows as u16
        } else {
            available_height / rows as u16
        };

        let mut rects = Vec::with_capacity(num_items);

        for item in 0..num_items {
            let (row, col) = match self.flow {
                GridFlow::Row | GridFlow::Dense => {
                    (item / self.columns, item % self.columns)
                }
                GridFlow::Column => (item % rows, item / rows),
            };

            let x = inner.left() + col as u16 * (cell_width + self.spacing.gap);
            let y = inner.top() + row as u16 * (cell_height + self.spacing.gap);

            rects.push(Rect {
                x,
                y,
                width: cell_width,
                height: cell_height,
            });
        }

        rects
    }
}

// ============================================================================
// Stack Layout
// ============================================================================

/// Stack layout for overlaying widgets
pub struct StackLayout {
    /// Layout mode
    mode: StackMode,
    /// Spacing configuration
    spacing: SpacingConfig,
    /// Child constraints
    constraints: Vec<Constraint>,
}

/// Stack layout mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackMode {
    /// All children occupy the same space
    Overlay,
    /// Children stacked with specified spacing
    Vertical,
    /// Children stacked horizontally
    Horizontal,
}

impl StackLayout {
    pub fn new(mode: StackMode) -> Self {
        Self {
            mode,
            spacing: SpacingConfig::default(),
            constraints: Vec::new(),
        }
    }

    pub fn overlay() -> Self {
        Self::new(StackMode::Overlay)
    }

    pub fn vertical() -> Self {
        Self::new(StackMode::Vertical)
    }

    pub fn horizontal() -> Self {
        Self::new(StackMode::Horizontal)
    }

    pub fn with_spacing(mut self, spacing: SpacingConfig) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn with_constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn calculate_layout(&self, area: Rect, num_children: usize) -> Vec<Rect> {
        let inner = self.spacing.apply_margin(area);

        match self.mode {
            StackMode::Overlay => {
                vec![inner; num_children]
            }
            StackMode::Vertical => {
                let total_gap = (num_children.saturating_sub(1)) as u16 * self.spacing.gap;
                let available = inner.height.saturating_sub(total_gap);
                let item_height = available / num_children as u16;

                (0..num_children)
                    .map(|i| Rect {
                        x: inner.x,
                        y: inner.top() + i as u16 * (item_height + self.spacing.gap),
                        width: inner.width,
                        height: item_height,
                    })
                    .collect()
            }
            StackMode::Horizontal => {
                let total_gap = (num_children.saturating_sub(1)) as u16 * self.spacing.gap;
                let available = inner.width.saturating_sub(total_gap);
                let item_width = available / num_children as u16;

                (0..num_children)
                    .map(|i| Rect {
                        x: inner.left() + i as u16 * (item_width + self.spacing.gap),
                        y: inner.y,
                        width: item_width,
                        height: inner.height,
                    })
                    .collect()
            }
        }
    }
}

// ============================================================================
// Anchor Layout
// ============================================================================

/// Anchor-based layout for positioning elements
pub struct AnchorLayout {
    /// Anchors for each child
    anchors: HashMap<usize, Anchor>,
    /// Default anchor for unanchored children
    default_anchor: Anchor,
    /// Spacing configuration
    spacing: SpacingConfig,
}

/// Anchor position for element placement
#[derive(Debug, Clone, Copy)]
pub struct Anchor {
    /// Horizontal anchor
    pub horizontal: HorizontalAnchor,
    /// Vertical anchor
    pub vertical: VerticalAnchor,
    /// Offset from anchor
    pub offset: (i16, i16),
}

/// Horizontal anchor position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAnchor {
    Left,
    Center,
    Right,
}

/// Vertical anchor position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAnchor {
    Top,
    Center,
    Bottom,
}

impl Anchor {
    pub const fn new(horizontal: HorizontalAnchor, vertical: VerticalAnchor) -> Self {
        Self {
            horizontal,
            vertical,
            offset: (0, 0),
        }
    }

    pub fn with_offset(mut self, x: i16, y: i16) -> Self {
        self.offset = (x, y);
        self
    }

    pub const TOP_LEFT: Self = Self::new(HorizontalAnchor::Left, VerticalAnchor::Top);
    pub const TOP_CENTER: Self = Self::new(HorizontalAnchor::Center, VerticalAnchor::Top);
    pub const TOP_RIGHT: Self = Self::new(HorizontalAnchor::Right, VerticalAnchor::Top);
    pub const CENTER_LEFT: Self = Self::new(HorizontalAnchor::Left, VerticalAnchor::Center);
    pub const CENTER: Self = Self::new(HorizontalAnchor::Center, VerticalAnchor::Center);
    pub const CENTER_RIGHT: Self = Self::new(HorizontalAnchor::Right, VerticalAnchor::Center);
    pub const BOTTOM_LEFT: Self = Self::new(HorizontalAnchor::Left, VerticalAnchor::Bottom);
    pub const BOTTOM_CENTER: Self = Self::new(HorizontalAnchor::Center, VerticalAnchor::Bottom);
    pub const BOTTOM_RIGHT: Self = Self::new(HorizontalAnchor::Right, VerticalAnchor::Bottom);
}

impl AnchorLayout {
    pub fn new() -> Self {
        Self {
            anchors: HashMap::new(),
            default_anchor: Anchor::CENTER,
            spacing: SpacingConfig::default(),
        }
    }

    pub fn with_default_anchor(mut self, anchor: Anchor) -> Self {
        self.default_anchor = anchor;
        self
    }

    pub fn with_anchor(mut self, index: usize, anchor: Anchor) -> Self {
        self.anchors.insert(index, anchor);
        self
    }

    pub fn with_spacing(mut self, spacing: SpacingConfig) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn calculate_child(
        &self,
        area: Rect,
        child_index: usize,
        child_width: u16,
        child_height: u16,
    ) -> Rect {
        let inner = self.spacing.apply_margin(area);
        let anchor = self.anchors.get(&child_index).copied().unwrap_or(self.default_anchor);

        let x = match anchor.horizontal {
            HorizontalAnchor::Left => inner.left() as i16,
            HorizontalAnchor::Center => (inner.left() as i16 + inner.right() as i16) / 2 - child_width as i16 / 2,
            HorizontalAnchor::Right => inner.right() as i16 - child_width as i16,
        };

        let y = match anchor.vertical {
            VerticalAnchor::Top => inner.top() as i16,
            VerticalAnchor::Center => (inner.top() as i16 + inner.bottom() as i16) / 2 - child_height as i16 / 2,
            VerticalAnchor::Bottom => inner.bottom() as i16 - child_height as i16,
        };

        let x = (x + anchor.offset.0).max(0) as u16;
        let y = (y + anchor.offset.1).max(0) as u16;

        Rect {
            x,
            y,
            width: child_width.min(inner.width),
            height: child_height.min(inner.height),
        }
    }
}

// ============================================================================
// Responsive Layout
// ============================================================================

/// Responsive layout that adapts to screen size
pub struct ResponsiveLayout {
    /// Small screen layout (width < 80)
    small_layout: Box<dyn Fn(Rect) -> Vec<Rect>>,
    /// Medium screen layout (80 <= width < 120)
    medium_layout: Box<dyn Fn(Rect) -> Vec<Rect>>,
    /// Large screen layout (width >= 120)
    large_layout: Box<dyn Fn(Rect) -> Vec<Rect>>,
    /// Spacing configuration
    spacing: SpacingConfig,
}

impl ResponsiveLayout {
    pub fn new<F1, F2, F3>(
        small: F1,
        medium: F2,
        large: F3,
    ) -> Self
    where
        F1: Fn(Rect) -> Vec<Rect> + 'static,
        F2: Fn(Rect) -> Vec<Rect> + 'static,
        F3: Fn(Rect) -> Vec<Rect> + 'static,
    {
        Self {
            small_layout: Box::new(small),
            medium_layout: Box::new(medium),
            large_layout: Box::new(large),
            spacing: SpacingConfig::default(),
        }
    }

    pub fn with_spacing(mut self, spacing: SpacingConfig) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn calculate_layout(&self, area: Rect) -> Vec<Rect> {
        let inner = self.spacing.apply_margin(area);

        if area.width < 80 {
            (self.small_layout)(inner)
        } else if area.width < 120 {
            (self.medium_layout)(inner)
        } else {
            (self.large_layout)(inner)
        }
    }

    /// Create a responsive layout from constraints
    pub fn from_constraints(
        small_constraints: Vec<Constraint>,
        medium_constraints: Vec<Constraint>,
        large_constraints: Vec<Constraint>,
    ) -> Self {
        Self::new(
            move |area| {
                Layout::default()
                    .constraints(small_constraints.clone())
                    .split(area)
            },
            move |area| {
                Layout::default()
                    .constraints(medium_constraints.clone())
                    .split(area)
            },
            move |area| {
                Layout::default()
                    .constraints(large_constraints.clone())
                    .split(area)
            },
        )
    }
}

// ============================================================================
// Animated Layout
// ============================================================================

/// Animated layout with transition effects
pub struct AnimatedLayout {
    /// Base layout
    base_layout: FlexLayout,
    /// Animation duration
    duration: Duration,
    /// Animation type
    animation_type: AnimationType,
    /// Current progress (0.0 to 1.0)
    progress: f32,
}

/// Animation type for layout transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    Fade,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    Scale,
}

impl AnimatedLayout {
    pub fn new(base_layout: FlexLayout, duration: Duration) -> Self {
        Self {
            base_layout,
            duration,
            animation_type: AnimationType::Fade,
            progress: 0.0,
        }
    }

    pub fn with_animation_type(mut self, animation_type: AnimationType) -> Self {
        self.animation_type = animation_type;
        self
    }

    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    pub fn update_progress(&mut self, delta: Duration) -> bool {
        let delta_progress = delta.as_secs_f32() / self.duration.as_secs_f32();
        self.progress = (self.progress + delta_progress).clamp(0.0, 1.0);
        self.progress >= 1.0
    }

    pub fn reset(&mut self) {
        self.progress = 0.0;
    }

    pub fn calculate_layout(&self, area: Rect) -> Vec<Rect> {
        let base_rects = self.base_layout.calculate_layout(area);
        let eased_progress = self.ease_in_out(self.progress);

        base_rects
            .iter()
            .map(|&rect| self.apply_animation(rect, area, eased_progress))
            .collect()
    }

    fn apply_animation(&self, rect: Rect, area: Rect, progress: f32) -> Rect {
        match self.animation_type {
            AnimationType::Fade => rect, // Fade handled by widget
            AnimationType::SlideLeft => {
                let offset = ((area.width as f32) * (1.0 - progress)) as i16;
                Rect {
                    x: (rect.x as i16 - offset).max(0) as u16,
                    ..rect
                }
            }
            AnimationType::SlideRight => {
                let offset = ((area.width as f32) * (1.0 - progress)) as i16;
                Rect {
                    x: (rect.x as i16 + offset) as u16,
                    ..rect
                }
            }
            AnimationType::SlideUp => {
                let offset = ((area.height as f32) * (1.0 - progress)) as i16;
                Rect {
                    y: (rect.y as i16 - offset).max(0) as u16,
                    ..rect
                }
            }
            AnimationType::SlideDown => {
                let offset = ((area.height as f32) * (1.0 - progress)) as i16;
                Rect {
                    y: (rect.y as i16 + offset) as u16,
                    ..rect
                }
            }
            AnimationType::Scale => {
                let scale = 0.5 + 0.5 * progress;
                let center_x = rect.x + rect.width / 2;
                let center_y = rect.y + rect.height / 2;
                let new_width = (rect.width as f32 * scale) as u16;
                let new_height = (rect.height as f32 * scale) as u16;
                Rect {
                    x: center_x.saturating_sub(new_width / 2),
                    y: center_y.saturating_sub(new_height / 2),
                    width: new_width,
                    height: new_height,
                }
            }
        }
    }

    fn ease_in_out(&self, t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }
}

// ============================================================================
// Constraint Builders
// ============================================================================

/// Helper functions for building constraints
pub struct Constraints;

impl Constraints {
    /// Create equal constraints for n items
    pub fn equal(n: usize) -> Vec<Constraint> {
        vec![Constraint::Ratio(1, n as u32); n]
    }

    /// Create percentage constraints from a slice
    pub fn from_percentages(percentages: &[u16]) -> Vec<Constraint> {
        percentages
            .iter()
            .map(|&p| Constraint::Percentage(p))
            .collect()
    }

    /// Create length constraints from a slice
    pub fn from_lengths(lengths: &[u16]) -> Vec<Constraint> {
        lengths
            .iter()
            .map(|&l| Constraint::Length(l))
            .collect()
    }

    /// Create ratio constraints from numerators (denominator calculated from sum)
    pub fn from_ratios(numerators: &[u32]) -> Vec<Constraint> {
        let denominator: u32 = numerators.iter().sum();
        numerators
            .iter()
            .map(|&n| Constraint::Ratio(n, denominator))
            .collect()
    }

    /// Mix different constraint types
    pub fn mix(constraint_spec: &[ConstraintSpec]) -> Vec<Constraint> {
        constraint_spec
            .iter()
            .map(|spec| match spec {
                ConstraintSpec::Length(l) => Constraint::Length(*l),
                ConstraintSpec::Percentage(p) => Constraint::Percentage(*p),
                ConstraintSpec::Ratio(n, d) => Constraint::Ratio(*n, *d),
                ConstraintSpec::Min(m) => Constraint::Min(*m),
                ConstraintSpec::Max(m) => Constraint::Max(*m),
            })
            .collect()
    }
}

/// Specification for constraint types
#[derive(Debug, Clone, Copy)]
pub enum ConstraintSpec {
    Length(u16),
    Percentage(u16),
    Ratio(u32, u32),
    Min(u16),
    Max(u16),
}

// ============================================================================
// Layout Utilities
// ============================================================================

/// Utility functions for layout calculations
pub struct LayoutUtils;

impl LayoutUtils {
    /// Calculate center position of an area
    pub fn center(area: Rect) -> (u16, u16) {
        (area.x + area.width / 2, area.y + area.height / 2)
    }

    /// Clamp a rect within bounds
    pub fn clamp(rect: Rect, bounds: Rect) -> Rect {
        Rect {
            x: rect.x.max(bounds.x).min(bounds.right()),
            y: rect.y.max(bounds.y).min(bounds.bottom()),
            width: rect.width.min(bounds.width),
            height: rect.height.min(bounds.height),
        }
    }

    /// Calculate distance between two rects
    pub fn distance(a: Rect, b: Rect) -> (i16, i16) {
        let a_center = LayoutUtils::center(a);
        let b_center = LayoutUtils::center(b);
        (
            b_center.0 as i16 - a_center.0 as i16,
            b_center.1 as i16 - a_center.1 as i16,
        )
    }

    /// Check if two rects overlap
    pub fn overlaps(a: Rect, b: Rect) -> bool {
        a.x < b.x + b.width
            && a.x + a.width > b.x
            && a.y < b.y + b.height
            && a.y + a.height > b.y
    }

    /// Calculate intersection of two rects
    pub fn intersection(a: Rect, b: Rect) -> Option<Rect> {
        let x = a.x.max(b.x);
        let y = a.y.max(b.y);
        let right = (a.x + a.width).min(b.x + b.width);
        let bottom = (a.y + a.height).min(b.y + b.height);

        if right > x && bottom > y {
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

    /// Inset a rect by margin
    pub fn inset(rect: Rect, horizontal: u16, vertical: u16) -> Rect {
        rect.inner(&Margin {
            horizontal,
            vertical,
        })
    }

    /// Outset a rect by margin (clamped to available space)
    pub fn outset(rect: Rect, horizontal: u16, vertical: u16, max_area: Rect) -> Rect {
        let x = rect.x.saturating_sub(horizontal);
        let y = rect.y.saturating_sub(vertical);
        let width = (rect.width + 2 * horizontal).min(max_area.width);
        let height = (rect.height + 2 * vertical).min(max_area.height);
        Rect { x, y, width, height }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacing_config_default() {
        let config = SpacingConfig::default();
        assert_eq!(config.margin_horizontal, 2);
        assert_eq!(config.margin_vertical, 1);
        assert_eq!(config.gap, 1);
    }

    #[test]
    fn test_spacing_config_builder() {
        let config = SpacingConfig::new((3, 2), (1, 1), 2);
        assert_eq!(config.margin_horizontal, 3);
        assert_eq!(config.margin_vertical, 2);
        assert_eq!(config.gap, 2);
    }

    #[test]
    fn test_flex_layout_horizontal() {
        let layout = FlexLayout::horizontal();
        let area = Rect::new(0, 0, 100, 20);
        let rects = layout.calculate_layout(area);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].width, 96); // 100 - 2*2 margin
    }

    #[test]
    fn test_flex_layout_with_constraints() {
        let layout = FlexLayout::horizontal()
            .with_constraints(vec![Constraint::Length(30), Constraint::Percentage(50)]);
        let area = Rect::new(0, 0, 100, 20);
        let rects = layout.calculate_layout(area);
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].width, 30);
    }

    #[test]
    fn test_grid_layout() {
        let layout = GridLayout::new(3);
        let area = Rect::new(0, 0, 100, 20);
        let rects = layout.calculate_layout(area, 7);
        assert_eq!(rects.len(), 7);
    }

    #[test]
    fn test_anchor_presets() {
        assert_eq!(Anchor::TOP_LEFT.horizontal, HorizontalAnchor::Left);
        assert_eq!(Anchor::TOP_LEFT.vertical, VerticalAnchor::Top);
        assert_eq!(Anchor::CENTER.horizontal, HorizontalAnchor::Center);
        assert_eq!(Anchor::CENTER.vertical, VerticalAnchor::Center);
    }

    #[test]
    fn test_constraints_equal() {
        let constraints = Constraints::equal(4);
        assert_eq!(constraints.len(), 4);
    }

    #[test]
    fn test_constraints_from_percentages() {
        let constraints = Constraints::from_percentages(&[25, 50, 25]);
        assert_eq!(constraints.len(), 3);
    }

    #[test]
    fn test_layout_utils_center() {
        let area = Rect::new(0, 0, 100, 50);
        let center = LayoutUtils::center(area);
        assert_eq!(center, (50, 25));
    }

    #[test]
    fn test_layout_utils_distance() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(20, 30, 10, 10);
        let distance = LayoutUtils::distance(a, b);
        assert_eq!(distance, (15, 25));
    }

    #[test]
    fn test_layout_utils_overlaps() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        assert!(LayoutUtils::overlaps(a, b));

        let c = Rect::new(20, 20, 10, 10);
        assert!(!LayoutUtils::overlaps(a, c));
    }

    #[test]
    fn test_responsive_layout() {
        let layout = ResponsiveLayout::from_constraints(
            vec![Constraint::Percentage(100)],
            vec![Constraint::Percentage(50), Constraint::Percentage(50)],
            vec![Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(34)],
        );

        let small_area = Rect::new(0, 0, 70, 20);
        let small_rects = layout.calculate_layout(small_area);
        assert_eq!(small_rects.len(), 1);

        let medium_area = Rect::new(0, 0, 100, 20);
        let medium_rects = layout.calculate_layout(medium_area);
        assert_eq!(medium_rects.len(), 2);

        let large_area = Rect::new(0, 0, 130, 20);
        let large_rects = layout.calculate_layout(large_area);
        assert_eq!(large_rects.len(), 3);
    }
}
