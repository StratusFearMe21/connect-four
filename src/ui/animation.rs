// Animation system for Connect Four UI
// Provides smooth animations and transitions for UI elements

use ratatui::layout::Rect;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::ui::{Theme, PaletteColor};

// ============================================================================
// Animation Configuration
// ============================================================================

/// Configuration for animations
#[derive(Debug, Clone, Copy)]
pub struct AnimationConfig {
    /// Default animation duration
    pub default_duration: Duration,
    /// Animation time scale (speed multiplier)
    pub time_scale: f32,
    /// Enable easing functions
    pub use_easing: bool,
    /// Maximum number of active animations
    pub max_animations: usize,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            default_duration: Duration::from_millis(300),
            time_scale: 1.0,
            use_easing: true,
            max_animations: 100,
        }
    }
}

impl AnimationConfig {
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.default_duration = duration;
        self
    }

    pub fn with_time_scale(mut self, scale: f32) -> Self {
        self.time_scale = scale.max(0.1).min(5.0);
        self
    }
}

// ============================================================================
// Easing Functions
// ============================================================================

/// Easing function for animation interpolation
pub type EasingFunction = fn(f32) -> f32;

/// Built-in easing functions
pub mod easing {
    /// Linear easing (no easing)
    pub fn linear(t: f32) -> f32 {
        t
    }

    /// Quadratic ease-in
    pub fn ease_in_quad(t: f32) -> f32 {
        t * t
    }

    /// Quadratic ease-out
    pub fn ease_out_quad(t: f32) -> f32 {
        t * (2.0 - t)
    }

    /// Quadratic ease-in-out
    pub fn ease_in_out_quad(t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }

    /// Cubic ease-in
    pub fn ease_in_cubic(t: f32) -> f32 {
        t * t * t
    }

    /// Cubic ease-out
    pub fn ease_out_cubic(t: f32) -> f32 {
        (t - 1.0).powi(3) + 1.0
    }

    /// Cubic ease-in-out
    pub fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            (t - 1.0).powi(3) * 4.0 + 1.0
        }
    }

    /// Exponential ease-in
    pub fn ease_in_exp(t: f32) -> f32 {
        if t == 0.0 {
            0.0
        } else {
            2f32.powf(10.0 * (t - 1.0))
        }
    }

    /// Exponential ease-out
    pub fn ease_out_exp(t: f32) -> f32 {
        if t == 1.0 {
            1.0
        } else {
            1.0 - 2f32.powf(-10.0 * t)
        }
    }

    /// Exponential ease-in-out
    pub fn ease_in_out_exp(t: f32) -> f32 {
        if t == 0.0 {
            0.0
        } else if t == 1.0 {
            1.0
        } else if t < 0.5 {
            2f32.powf(20.0 * t - 10.0) / 2.0
        } else {
            (2.0 - 2f32.powf(-20.0 * t + 10.0)) / 2.0
        }
    }

    /// Elastic ease-out
    pub fn elastic_out(t: f32) -> f32 {
        if t == 0.0 || t == 1.0 {
            t
        } else {
            2f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * (2.0 * std::f32::consts::PI).sin()) + 1.0
        }
    }

    /// Bounce ease-out
    pub fn bounce_out(t: f32) -> f32 {
        if t < 1.0 / 2.75 {
            7.5625 * t * t
        } else if t < 2.0 / 2.75 {
            7.5625 * (t - 1.5 / 2.75).powi(2) + 0.75
        } else if t < 2.5 / 2.75 {
            7.5625 * (t - 2.25 / 2.75).powi(2) + 0.9375
        } else {
            7.5625 * (t - 2.625 / 2.75).powi(2) + 0.984375
        }
    }
}

// ============================================================================
// Animation Type
// ============================================================================

/// Type of animation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    /// Fade in/out
    Fade,
    /// Slide (direction specified separately)
    Slide,
    /// Scale
    Scale,
    /// Rotate
    Rotate,
    /// Pulse
    Pulse,
    /// Shake
    Shake,
    /// Bounce
    Bounce,
    /// Custom
    Custom,
}

// ============================================================================
// Animation State
// ============================================================================

/// State of an animation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    /// Not started
    Idle,
    /// Playing
    Playing,
    /// Paused
    Paused,
    /// Completed
    Completed,
    /// Cancelled
    Cancelled,
}

// ============================================================================
// Animation Direction
// ============================================================================

/// Direction for slide animations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideDirection {
    Up,
    Down,
    Left,
    Right,
}

// ============================================================================
// Animation Properties
// ============================================================================

/// Properties for an animation
#[derive(Debug, Clone)]
pub struct AnimationProperties {
    /// Duration of the animation
    pub duration: Duration,
    /// Easing function
    pub easing: EasingFunction,
    /// Delay before starting
    pub delay: Duration,
    /// Number of repetitions (0 = infinite, 1 = once)
    pub repetitions: u32,
    /// Whether to alternate direction on repetition
    pub alternate: bool,
    /// Whether to keep final state after completion
    pub retain: bool,
}

impl Default for AnimationProperties {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(300),
            easing: easing::ease_in_out_quad,
            delay: Duration::ZERO,
            repetitions: 1,
            alternate: false,
            retain: true,
        }
    }
}

impl AnimationProperties {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            ..Default::default()
        }
    }

    pub fn with_easing(mut self, easing: EasingFunction) -> Self {
        self.easing = easing;
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub fn with_repetitions(mut self, reps: u32) -> Self {
        self.repetitions = reps;
        self
    }

    pub fn with_alternate(mut self, alt: bool) -> Self {
        self.alternate = alt;
        self
    }

    pub fn with_retain(mut self, retain: bool) -> Self {
        self.retain = retain;
        self
    }
}

// ============================================================================
// Animation
// ============================================================================

/// Single animation instance
pub struct Animation {
    /// Unique ID
    id: String,
    /// Animation type
    anim_type: AnimationType,
    /// Animation properties
    properties: AnimationProperties,
    /// Current state
    state: AnimationState,
    /// Progress (0.0 to 1.0)
    progress: f32,
    /// Start time
    start_time: Option<Instant>,
    /// Current repetition
    current_repetition: u32,
    /// Direction for alternating animations
    forward: bool,
}

impl Animation {
    pub fn new(id: String, anim_type: AnimationType, properties: AnimationProperties) -> Self {
        Self {
            id,
            anim_type,
            properties,
            state: AnimationState::Idle,
            progress: 0.0,
            start_time: None,
            current_repetition: 0,
            forward: true,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn anim_type(&self) -> AnimationType {
        self.anim_type
    }

    pub fn state(&self) -> AnimationState {
        self.state
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn start(&mut self) {
        self.state = AnimationState::Playing;
        self.start_time = Some(Instant::now());
    }

    pub fn pause(&mut self) {
        if self.state == AnimationState::Playing {
            self.state = AnimationState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Playing;
        }
    }

    pub fn cancel(&mut self) {
        self.state = AnimationState::Cancelled;
    }

    pub fn reset(&mut self) {
        self.state = AnimationState::Idle;
        self.progress = 0.0;
        self.start_time = None;
        self.current_repetition = 0;
        self.forward = true;
    }

    pub fn update(&mut self, delta: Duration, time_scale: f32) -> bool {
        if self.state != AnimationState::Playing {
            return false;
        }

        let start_time = match self.start_time {
            Some(t) => t,
            None => {
                self.start_time = Some(Instant::now());
                return false;
            }
        };

        let elapsed = start_time.elapsed();

        // Check for delay
        if elapsed < self.properties.delay {
            return false;
        }

        let anim_elapsed = elapsed - self.properties.delay;
        let scaled_elapsed = anim_elapsed.mul_f32(time_scale);
        let total_duration = self.properties.duration;

        // Calculate progress
        let raw_progress = (scaled_elapsed.as_secs_f32() / total_duration.as_secs_f32()).clamp(0.0, 1.0);

        // Apply easing
        self.progress = (self.properties.easing)(raw_progress);

        // If alternating and not on first repetition
        if !self.forward {
            self.progress = 1.0 - self.progress;
        }

        // Check if animation is complete
        if raw_progress >= 1.0 {
            self.current_repetition += 1;

            if self.properties.repetitions == 0 || self.current_repetition < self.properties.repetitions {
                // Reset for next repetition
                self.start_time = Some(Instant::now());
                if self.properties.alternate {
                    self.forward = !self.forward;
                }
                return false;
            } else {
                self.state = AnimationState::Completed;
                self.progress = if self.properties.retain { 1.0 } else { 0.0 };
                return true;
            }
        }

        false
    }

    pub fn is_complete(&self) -> bool {
        self.state == AnimationState::Completed
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, AnimationState::Playing | AnimationState::Paused)
    }
}

// ============================================================================
// Animation Manager
// ============================================================================

/// Manages multiple animations
pub struct AnimationManager {
    /// Configuration
    config: AnimationConfig,
    /// Active animations
    animations: HashMap<String, Animation>,
    /// Animation queue (ordered by priority)
    queue: Vec<String>,
}

impl AnimationManager {
    pub fn new(config: AnimationConfig) -> Self {
        Self {
            config,
            animations: HashMap::new(),
            queue: Vec::new(),
        }
    }

    pub fn add(&mut self, animation: Animation) {
        let id = animation.id().to_string();
        self.animations.insert(id.clone(), animation);
        self.queue.push(id);
    }

    pub fn remove(&mut self, id: &str) -> Option<Animation> {
        self.queue.retain(|x| x != id);
        self.animations.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<&Animation> {
        self.animations.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Animation> {
        self.animations.get_mut(id)
    }

    pub fn start(&mut self, id: &str) {
        if let Some(anim) = self.animations.get_mut(id) {
            anim.start();
        }
    }

    pub fn pause(&mut self, id: &str) {
        if let Some(anim) = self.animations.get_mut(id) {
            anim.pause();
        }
    }

    pub fn resume(&mut self, id: &str) {
        if let Some(anim) = self.animations.get_mut(id) {
            anim.resume();
        }
    }

    pub fn cancel(&mut self, id: &str) {
        if let Some(anim) = self.animations.get_mut(id) {
            anim.cancel();
        }
    }

    pub fn update(&mut self, delta: Duration) -> Vec<String> {
        let mut completed = Vec::new();

        for id in self.queue.clone() {
            if let Some(anim) = self.animations.get_mut(&id) {
                let was_complete = anim.is_complete();
                anim.update(delta, self.config.time_scale);
                if !was_complete && anim.is_complete() {
                    completed.push(id);
                }
            }
        }

        // Remove completed animations
        for id in &completed {
            self.remove(id);
        }

        completed
    }

    pub fn cancel_all(&mut self) {
        for anim in self.animations.values_mut() {
            anim.cancel();
        }
    }

    pub fn reset_all(&mut self) {
        for anim in self.animations.values_mut() {
            anim.reset();
        }
    }

    pub fn active_count(&self) -> usize {
        self.animations.values().filter(|a| a.is_active()).count()
    }

    pub fn clear(&mut self) {
        self.animations.clear();
        self.queue.clear();
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new(AnimationConfig::default())
    }
}

// ============================================================================
// Common Animations
// ============================================================================

/// Pre-built animations for common UI effects
pub struct CommonAnimations;

impl CommonAnimations {
    /// Create a fade-in animation
    pub fn fade_in(id: String, duration: Duration) -> Animation {
        let props = AnimationProperties::new(duration)
            .with_easing(easing::ease_in_out_quad);
        Animation::new(id, AnimationType::Fade, props)
    }

    /// Create a fade-out animation
    pub fn fade_out(id: String, duration: Duration) -> Animation {
        let props = AnimationProperties::new(duration)
            .with_easing(easing::ease_in_out_quad);
        Animation::new(id, AnimationType::Fade, props)
    }

    /// Create a slide animation
    pub fn slide(id: String, duration: Duration, direction: SlideDirection) -> Animation {
        let props = AnimationProperties::new(duration)
            .with_easing(easing::ease_out_cubic);
        Animation::new(id, AnimationType::Slide, props)
    }

    /// Create a scale animation
    pub fn scale(id: String, duration: Duration) -> Animation {
        let props = AnimationProperties::new(duration)
            .with_easing(easing::elastic_out);
        Animation::new(id, AnimationType::Scale, props)
    }

    /// Create a pulse animation
    pub fn pulse(id: String, duration: Duration) -> Animation {
        let props = AnimationProperties::new(duration)
            .with_easing(easing::ease_in_out_sine)
            .with_repetitions(0); // Infinite
        Animation::new(id, AnimationType::Pulse, props)
    }

    /// Create a bounce animation
    pub fn bounce(id: String, duration: Duration) -> Animation {
        let props = AnimationProperties::new(duration)
            .with_easing(easing::bounce_out);
        Animation::new(id, AnimationType::Bounce, props)
    }

    /// Create a shake animation
    pub fn shake(id: String, duration: Duration) -> Animation {
        let props = AnimationProperties::new(duration)
            .with_easing(easing::linear)
            .with_repetitions(3)
            .with_alternate(true);
        Animation::new(id, AnimationType::Shake, props)
    }
}

// ============================================================================
// Animation Interpolator
// ============================================================================

/// Interpolates values based on animation progress
pub struct Interpolator;

impl Interpolator {
    /// Interpolate between two f32 values
    pub fn f32(progress: f32, start: f32, end: f32) -> f32 {
        start + (end - start) * progress
    }

    /// Interpolate between two u16 values (for positions/sizes)
    pub fn u16(progress: f32, start: u16, end: u16) -> u16 {
        Interpolator::f32(progress, start as f32, end as f32) as u16
    }

    /// Interpolate between two i16 values
    pub fn i16(progress: f32, start: i16, end: i16) -> i16 {
        Interpolator::f32(progress, start as f32, end as f32) as i16
    }

    /// Interpolate color
    pub fn color(progress: f32, start: ratatui::style::Color, end: ratatui::style::Color) -> ratatui::style::Color {
        match (start, end) {
            (ratatui::style::Color::Rgb(sr, sg, sb), ratatui::style::Color::Rgb(er, eg, eb)) => {
                let r = Interpolator::f32(progress, sr as f32, er as f32) as u8;
                let g = Interpolator::f32(progress, sg as f32, eg as f32) as u8;
                let b = Interpolator::f32(progress, sb as f32, eb as f32) as u8;
                ratatui::style::Color::Rgb(r, g, b)
            }
            _ => end, // Can't interpolate other color types
        }
    }

    /// Interpolate Rect
    pub fn rect(progress: f32, start: Rect, end: Rect) -> Rect {
        Rect {
            x: Interpolator::u16(progress, start.x, end.x),
            y: Interpolator::u16(progress, start.y, end.y),
            width: Interpolator::u16(progress, start.width, end.width),
            height: Interpolator::u16(progress, start.height, end.height),
        }
    }

    /// Eased interpolation
    pub fn eased<T>(progress: f32, easing: EasingFunction, start: T, end: T) -> T
    where
        T: Copy + std::ops::Sub<Output = T> + std::ops::Add<T, Output = T> + Into<f32> + From<f32>,
    {
        let eased = easing(progress);
        let start_f: f32 = start.into();
        let end_f: f32 = end.into();
        let result_f = start_f + (end_f - start_f) * eased;
        T::from(result_f)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_functions() {
        assert_eq!(easing::linear(0.0), 0.0);
        assert_eq!(easing::linear(0.5), 0.5);
        assert_eq!(easing::linear(1.0), 1.0);

        assert_eq!(easing::ease_in_quad(0.5), 0.25);
        assert_eq!(easing::ease_out_quad(0.5), 0.75);

        assert!(easing::ease_in_cubic(0.5) < easing::ease_out_cubic(0.5));
    }

    #[test]
    fn test_animation_properties_default() {
        let props = AnimationProperties::default();
        assert_eq!(props.duration, Duration::from_millis(300));
        assert_eq!(props.repetitions, 1);
        assert!(!props.alternate);
    }

    #[test]
    fn test_animation_properties_builder() {
        let props = AnimationProperties::new(Duration::from_millis(500))
            .with_easing(easing::ease_in_quad)
            .with_repetitions(3)
            .with_alternate(true);

        assert_eq!(props.duration, Duration::from_millis(500));
        assert_eq!(props.repetitions, 3);
        assert!(props.alternate);
    }

    #[test]
    fn test_animation_lifecycle() {
        let mut anim = Animation::new(
            "test".to_string(),
            AnimationType::Fade,
            AnimationProperties::new(Duration::from_millis(100)),
        );

        assert_eq!(anim.state(), AnimationState::Idle);
        assert!(!anim.is_active());

        anim.start();
        assert_eq!(anim.state(), AnimationState::Playing);
        assert!(anim.is_active());

        anim.pause();
        assert_eq!(anim.state(), AnimationState::Paused);

        anim.resume();
        assert_eq!(anim.state(), AnimationState::Playing);

        anim.cancel();
        assert_eq!(anim.state(), AnimationState::Cancelled);
    }

    #[test]
    fn test_animation_manager() {
        let mut manager = AnimationManager::default();
        let anim = Animation::new(
            "test".to_string(),
            AnimationType::Fade,
            AnimationProperties::new(Duration::from_millis(100)),
        );

        manager.add(anim);
        assert_eq!(manager.active_count(), 0);

        manager.start("test");
        assert_eq!(manager.active_count(), 1);

        manager.cancel("test");
        assert_eq!(manager.active_count(), 1); // Still counted until removed
    }

    #[test]
    fn test_interpolator() {
        assert_eq!(Interpolator::f32(0.0, 0.0, 100.0), 0.0);
        assert_eq!(Interpolator::f32(0.5, 0.0, 100.0), 50.0);
        assert_eq!(Interpolator::f32(1.0, 0.0, 100.0), 100.0);

        assert_eq!(Interpolator::u16(0.5, 0, 100), 50);
        assert_eq!(Interpolator::i16(0.5, -100, 100), 0);
    }

    #[test]
    fn test_interpolator_rect() {
        let start = Rect::new(0, 0, 10, 10);
        let end = Rect::new(10, 10, 20, 20);
        let result = Interpolator::rect(0.5, start, end);

        assert_eq!(result.x, 5);
        assert_eq!(result.y, 5);
        assert_eq!(result.width, 15);
        assert_eq!(result.height, 15);
    }
}
