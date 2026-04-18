// Screen management for Connect Four UI
// Provides screen state management and navigation

use ratatui::layout::Rect;
use std::collections::HashMap;
use std::time::Instant;
use crate::ui::{Theme, PaletteColor};
use crate::ui::animation::AnimationManager;
use crate::ui::effects::{ParticleSystem, ScreenShake, FlashEffect};

// ============================================================================
// Screen ID
// ============================================================================

/// Unique identifier for screens
pub type ScreenId = String;

// ============================================================================
// Screen Stack
// ============================================================================

/// Stack of active screens for navigation
#[derive(Debug, Clone)]
pub struct ScreenStack {
    /// Screen IDs in stack order (top is last)
    screens: Vec<ScreenId>,
    /// Maximum stack size
    max_size: usize,
}

impl ScreenStack {
    pub fn new(max_size: usize) -> Self {
        Self {
            screens: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, id: ScreenId) {
        if self.screens.len() >= self.max_size {
            self.screens.remove(0);
        }
        self.screens.push(id);
    }

    pub fn pop(&mut self) -> Option<ScreenId> {
        self.screens.pop()
    }

    pub fn peek(&self) -> Option<&ScreenId> {
        self.screens.last()
    }

    pub fn current(&self) -> Option<&ScreenId> {
        self.peek()
    }

    pub fn clear(&mut self) {
        self.screens.clear();
    }

    pub fn len(&self) -> usize {
        self.screens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.screens.is_empty()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.screens.iter().any(|s| s == id)
    }

    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.screens.iter().position(|s| s == id) {
            self.screens.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ScreenId> {
        self.screens.iter()
    }
}

// ============================================================================
// Screen State
// ============================================================================

/// State of a screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenState {
    /// Not initialized
    Uninitialized,
    /// Loading
    Loading,
    /// Active and visible
    Active,
    /// Paused (screen underneath is active)
    Paused,
    /// Transitioning
    Transitioning,
    /// Being destroyed
    Destroying,
}

// ============================================================================
// Screen Transition
// ============================================================================

/// Type of screen transition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenTransition {
    /// No transition
    None,
    /// Fade to new screen
    Fade,
    /// Slide from left
    SlideLeft,
    /// Slide from right
    SlideRight,
    /// Slide from top
    SlideTop,
    /// Slide from bottom
    SlideBottom,
    /// Scale in/out
    Scale,
    /// Custom
    Custom,
}

// ============================================================================
// Screen Context
// ============================================================================

/// Context passed to screen lifecycle methods
pub struct ScreenContext {
    /// Screen ID
    pub id: ScreenId,
    /// Theme reference
    pub theme: Theme,
    /// Screen area
    pub area: Rect,
    /// Creation time
    pub created_at: Instant,
    /// User data
    pub user_data: HashMap<String, String>,
}

impl ScreenContext {
    pub fn new(id: ScreenId, theme: Theme, area: Rect) -> Self {
        Self {
            id,
            theme,
            area,
            created_at: Instant::now(),
            user_data: HashMap::new(),
        }
    }

    pub fn with_user_data(mut self, key: String, value: String) -> Self {
        self.user_data.insert(key, value);
        self
    }

    pub fn get_user_data(&self, key: &str) -> Option<&String> {
        self.user_data.get(key)
    }
}

// ============================================================================
// Screen
// ============================================================================

/// Represents a screen in the UI
pub trait Screen: Send + Sync {
    /// Get screen ID
    fn id(&self) -> &ScreenId;

    /// Called when screen is created
    fn on_create(&mut self, _ctx: &ScreenContext) {}

    /// Called when screen becomes active
    fn on_activate(&mut self, _ctx: &ScreenContext) {}

    /// Called when screen becomes inactive (another screen pushed on top)
    fn on_deactivate(&mut self, _ctx: &ScreenContext) {}

    /// Called when screen is resumed (screen on top popped)
    fn on_resume(&mut self, _ctx: &ScreenContext) {}

    /// Called when screen is destroyed
    fn on_destroy(&mut self, _ctx: &ScreenContext) {}

    /// Called every frame when screen is active
    fn on_update(&mut self, _ctx: &ScreenContext, _delta: std::time::Duration) {}

    /// Render the screen
    fn render(&self, ctx: &ScreenContext, frame: &mut ratatui::Frame);

    /// Get screen state
    fn state(&self) -> ScreenState {
        ScreenState::Active
    }

    /// Set screen state
    fn set_state(&mut self, _state: ScreenState) {}

    /// Get preferred transition type
    fn transition_type(&self) -> ScreenTransition {
        ScreenTransition::None
    }

    /// Get animation manager reference (optional)
    fn animation_manager(&self) -> Option<&AnimationManager> {
        None
    }

    /// Get animation manager reference mut (optional)
    fn animation_manager_mut(&mut self) -> Option<&mut AnimationManager> {
        None
    }

    /// Get particle system reference (optional)
    fn particle_system(&self) -> Option<&ParticleSystem> {
        None
    }

    /// Get particle system reference mut (optional)
    fn particle_system_mut(&mut self) -> Option<&mut ParticleSystem> {
        None
    }

    /// Get screen shake reference (optional)
    fn screen_shake(&self) -> Option<&ScreenShake> {
        None
    }

    /// Get screen shake reference mut (optional)
    fn screen_shake_mut(&mut self) -> Option<&mut ScreenShake> {
        None
    }

    /// Get flash effect reference (optional)
    fn flash_effect(&self) -> Option<&FlashEffect> {
        None
    }

    /// Get flash effect reference mut (optional)
    fn flash_effect_mut(&mut self) -> Option<&mut FlashEffect> {
        None
    }

    /// Clone screen (for screen manager)
    fn clone_screen(&self) -> Box<dyn Screen>;
}

// ============================================================================
// Screen Manager
// ============================================================================

/// Manages all screens and navigation
pub struct ScreenManager {
    /// Registered screens
    screens: HashMap<ScreenId, Box<dyn Screen>>,
    /// Screen stack
    stack: ScreenStack,
    /// Current transition
    transition: Option<ScreenTransition>,
    /// Transition progress (0.0 to 1.0)
    transition_progress: f32,
    /// Theme
    theme: Theme,
    /// Screen area
    area: Rect,
    /// Previous screen area (for transitions)
    previous_area: Option<Rect>,
}

impl ScreenManager {
    pub fn new(theme: Theme) -> Self {
        Self {
            screens: HashMap::new(),
            stack: ScreenStack::new(10),
            transition: None,
            transition_progress: 0.0,
            theme,
            area: Rect::default(),
            previous_area: None,
        }
    }

    pub fn register_screen(&mut self, screen: Box<dyn Screen>) {
        let id = screen.id().clone();
        self.screens.insert(id, screen);
    }

    pub fn unregister_screen(&mut self, id: &ScreenId) -> Option<Box<dyn Screen>> {
        self.screens.remove(id)
    }

    pub fn get_screen(&self, id: &ScreenId) -> Option<&dyn Screen> {
        self.screens.get(id).map(|s| s.as_ref())
    }

    pub fn get_screen_mut(&mut self, id: &ScreenId) -> Option<&mut dyn Screen> {
        self.screens.get_mut(id).map(|s| s.as_mut())
    }

    pub fn current_screen(&self) -> Option<&dyn Screen> {
        self.stack.current().and_then(|id| self.get_screen(id))
    }

    pub fn current_screen_mut(&mut self) -> Option<&mut dyn Screen> {
        let id = self.stack.current()?.clone();
        self.get_screen_mut(&id)
    }

    pub fn push_screen(&mut self, id: ScreenId) -> Result<(), String> {
        if !self.screens.contains_key(&id) {
            return Err(format!("Screen '{}' not registered", id));
        }

        // Deactivate current screen if exists
        if let Some(current_id) = self.stack.current() {
            if let Some(screen) = self.get_screen_mut(current_id) {
                screen.set_state(ScreenState::Paused);
                screen.on_deactivate(&self.create_context(current_id.clone()));
            }
        }

        // Activate new screen
        self.stack.push(id.clone());
        if let Some(screen) = self.get_screen_mut(&id) {
            if screen.state() == ScreenState::Uninitialized {
                let ctx = self.create_context(id.clone());
                screen.on_create(&ctx);
            }
            screen.set_state(ScreenState::Active);
            screen.on_activate(&self.create_context(id));
        }

        Ok(())
    }

    pub fn pop_screen(&mut self) -> Option<ScreenId> {
        let popped_id = self.stack.pop()?;

        // Destroy popped screen
        if let Some(screen) = self.get_screen_mut(&popped_id) {
            screen.set_state(ScreenState::Destroying);
            screen.on_destroy(&self.create_context(popped_id.clone()));
        }

        // Resume screen underneath
        if let Some(current_id) = self.stack.current() {
            if let Some(screen) = self.get_screen_mut(current_id) {
                screen.set_state(ScreenState::Active);
                screen.on_resume(&self.create_context(current_id.clone()));
            }
        }

        Some(popped_id)
    }

    pub fn replace_screen(&mut self, id: ScreenId) -> Result<(), String> {
        self.pop_screen();
        self.push_screen(id)
    }

    pub fn clear_stack(&mut self) {
        while let Some(id) = self.stack.pop() {
            if let Some(screen) = self.get_screen_mut(&id) {
                screen.set_state(ScreenState::Destroying);
                screen.on_destroy(&self.create_context(id.clone()));
            }
        }
    }

    pub fn navigate_to(&mut self, id: ScreenId) -> Result<(), String> {
        self.clear_stack();
        self.push_screen(id)
    }

    pub fn update(&mut self, delta: std::time::Duration) {
        // Update current screen
        if let Some(id) = self.stack.current().cloned() {
            if let Some(screen) = self.get_screen_mut(&id) {
                screen.on_update(&self.create_context(id.clone()), delta);
            }
        }

        // Update transition
        if self.transition.is_some() {
            self.transition_progress += delta.as_secs_f32() * 2.0; // 0.5 second transition
            if self.transition_progress >= 1.0 {
                self.transition = None;
                self.transition_progress = 0.0;
            }
        }
    }

    pub fn render(&self, frame: &mut ratatui::Frame) {
        if let Some(id) = self.stack.current() {
            if let Some(screen) = self.get_screen(id) {
                let ctx = self.create_context(id.clone());
                screen.render(&ctx, frame);
            }
        }
    }

    pub fn set_area(&mut self, area: Rect) {
        self.previous_area = Some(self.area);
        self.area = area;
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn stack(&self) -> &ScreenStack {
        &self.stack
    }

    pub fn in_transition(&self) -> bool {
        self.transition.is_some()
    }

    pub fn transition_progress(&self) -> f32 {
        self.transition_progress
    }

    fn create_context(&self, id: ScreenId) -> ScreenContext {
        ScreenContext::new(id, self.theme.clone(), self.area)
    }
}

impl Default for ScreenManager {
    fn default() -> Self {
        Self::new(Theme::default())
    }
}

// ============================================================================
// Base Screen
// ============================================================================

/// Base implementation of Screen for easy screen creation
pub struct BaseScreen {
    id: ScreenId,
    state: ScreenState,
    transition_type: ScreenTransition,
}

impl BaseScreen {
    pub fn new(id: ScreenId) -> Self {
        Self {
            id,
            state: ScreenState::Uninitialized,
            transition_type: ScreenTransition::None,
        }
    }

    pub fn with_transition_type(mut self, transition: ScreenTransition) -> Self {
        self.transition_type = transition;
        self
    }
}

impl Screen for BaseScreen {
    fn id(&self) -> &ScreenId {
        &self.id
    }

    fn state(&self) -> ScreenState {
        self.state
    }

    fn set_state(&mut self, state: ScreenState) {
        self.state = state;
    }

    fn transition_type(&self) -> ScreenTransition {
        self.transition_type
    }

    fn render(&self, _ctx: &ScreenContext, _frame: &mut ratatui::Frame) {
        // Override in subclasses
    }

    fn clone_screen(&self) -> Box<dyn Screen> {
        Box::new(BaseScreen {
            id: self.id.clone(),
            state: self.state,
            transition_type: self.transition_type,
        })
    }
}

// ============================================================================
// Screen Builder
// ============================================================================

/// Builder for creating screens
pub struct ScreenBuilder {
    id: ScreenId,
    transition_type: ScreenTransition,
}

impl ScreenBuilder {
    pub fn new(id: ScreenId) -> Self {
        Self {
            id,
            transition_type: ScreenTransition::None,
        }
    }

    pub fn with_transition(mut self, transition: ScreenTransition) -> Self {
        self.transition_type = transition;
        self
    }

    pub fn build<F>(self, render_fn: F) -> BaseScreen
    where
        F: Fn(&ScreenContext, &mut ratatui::Frame) + Send + Sync + 'static,
    {
        BaseScreen::new(self.id.clone()).with_transition_type(self.transition_type)
    }
}

// ============================================================================
// Common Screen IDs
// ============================================================================

/// Common screen ID constants
pub mod screen_ids {
    pub const MAIN_MENU: &str = "main_menu";
    pub const GAME: &str = "game";
    pub const SETTINGS: &str = "settings";
    pub const CREDITS: &str = "credits";
    pub const PAUSE: &str = "pause";
    pub const GAME_OVER: &str = "game_over";
    pub const MULTIPLAYER_LOBBY: &str = "multiplayer_lobby";
    pub const ACHIEVEMENTS: &str = "achievements";
    pub const STATS: &str = "stats";
    pub const TUTORIAL: &str = "tutorial";
    pub const PUZZLE_SELECT: &str = "puzzle_select";
    pub const PUZZLE: &str = "puzzle";
    pub const TOURNAMENT: &str = "tournament";
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_screen_stack() {
        let mut stack = ScreenStack::new(5);

        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);

        stack.push("screen1".to_string());
        stack.push("screen2".to_string());
        stack.push("screen3".to_string());

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.current(), Some(&"screen3".to_string()));

        let popped = stack.pop();
        assert_eq!(popped, Some("screen3".to_string()));
        assert_eq!(stack.current(), Some(&"screen2".to_string()));

        assert!(stack.contains("screen1"));
        assert!(!stack.contains("screen3"));
    }

    #[test]
    fn test_screen_stack_max_size() {
        let mut stack = ScreenStack::new(3);

        stack.push("1".to_string());
        stack.push("2".to_string());
        stack.push("3".to_string());
        stack.push("4".to_string());

        assert_eq!(stack.len(), 3);
        assert!(!stack.contains("1"));
        assert!(stack.contains("4"));
    }

    #[test]
    fn test_screen_manager_registration() {
        let mut manager = ScreenManager::new(Theme::default());

        let screen = BaseScreen::new("test".to_string());
        manager.register_screen(Box::new(screen));

        assert!(manager.get_screen(&"test".to_string()).is_some());
    }

    #[test]
    fn test_screen_manager_navigation() {
        let mut manager = ScreenManager::new(Theme::default());

        let screen1 = BaseScreen::new("screen1".to_string());
        let screen2 = BaseScreen::new("screen2".to_string());

        manager.register_screen(Box::new(screen1));
        manager.register_screen(Box::new(screen2));

        manager.push_screen("screen1".to_string()).unwrap();
        assert_eq!(manager.stack().current(), Some(&"screen1".to_string()));

        manager.push_screen("screen2".to_string()).unwrap();
        assert_eq!(manager.stack().current(), Some(&"screen2".to_string()));

        let popped = manager.pop_screen();
        assert_eq!(popped, Some("screen2".to_string()));
        assert_eq!(manager.stack().current(), Some(&"screen1".to_string()));
    }

    #[test]
    fn test_screen_context() {
        let ctx = ScreenContext::new("test".to_string(), Theme::default(), Rect::new(0, 0, 100, 50))
            .with_user_data("key".to_string(), "value".to_string());

        assert_eq!(ctx.id, "test");
        assert_eq!(ctx.get_user_data("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_base_screen() {
        let mut screen = BaseScreen::new("test".to_string())
            .with_transition_type(ScreenTransition::Fade);

        assert_eq!(screen.id(), &"test".to_string());
        assert_eq!(screen.state(), ScreenState::Uninitialized);
        assert_eq!(screen.transition_type(), ScreenTransition::Fade);

        screen.set_state(ScreenState::Active);
        assert_eq!(screen.state(), ScreenState::Active);
    }

    #[test]
    fn test_screen_builder() {
        let builder = ScreenBuilder::new("test".to_string())
            .with_transition(ScreenTransition::SlideLeft);

        // Builder creates screens
        let _screen = builder.build();
    }

    #[test]
    fn test_screen_manager_clear_stack() {
        let mut manager = ScreenManager::new(Theme::default());

        manager.register_screen(Box::new(BaseScreen::new("1".to_string())));
        manager.register_screen(Box::new(BaseScreen::new("2".to_string())));
        manager.register_screen(Box::new(BaseScreen::new("3".to_string())));

        manager.push_screen("1".to_string()).unwrap();
        manager.push_screen("2".to_string()).unwrap();
        manager.push_screen("3".to_string()).unwrap();

        manager.clear_stack();

        assert!(manager.stack().is_empty());
    }
}
