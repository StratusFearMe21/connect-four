// Visual effects for Connect Four UI
// Provides particle effects, screen shakes, and other visual polish

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::ui::{Theme, PaletteColor};

// ============================================================================
// Effect Configuration
// ============================================================================

/// Configuration for visual effects
#[derive(Debug, Clone, Copy)]
pub struct EffectsConfig {
    /// Enable particle effects
    pub enable_particles: bool,
    /// Max particles per effect
    pub max_particles: usize,
    /// Enable screen shake
    pub enable_shake: bool,
    /// Enable glow effects
    pub enable_glow: bool,
    /// Enable particle trails
    pub enable_trails: bool,
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self {
            enable_particles: true,
            max_particles: 100,
            enable_shake: true,
            enable_glow: true,
            enable_trails: true,
        }
    }
}

impl EffectsConfig {
    pub fn with_particles(mut self, enable: bool) -> Self {
        self.enable_particles = enable;
        self
    }

    pub fn with_shake(mut self, enable: bool) -> Self {
        self.enable_shake = enable;
        self
    }

    pub fn with_glow(mut self, enable: bool) -> Self {
        self.enable_glow = enable;
        self
    }
}

// ============================================================================
// Particle
// ============================================================================

/// Single particle in a particle effect
#[derive(Debug, Clone)]
pub struct Particle {
    /// Position
    pub x: f32,
    pub y: f32,
    /// Velocity
    pub vx: f32,
    pub vy: f32,
    /// Life (0.0 to 1.0)
    pub life: f32,
    /// Max life
    pub max_life: f32,
    /// Size
    pub size: f32,
    /// Color
    pub color: Color,
    /// Fade rate
    pub fade_rate: f32,
    /// Gravity
    pub gravity: f32,
    /// Trail positions
    pub trail: Vec<(f32, f32)>,
    /// Has trail
    pub has_trail: bool,
}

impl Particle {
    pub fn new(x: f32, y: f32, color: Color) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            life: 1.0,
            max_life: 1.0,
            size: 1.0,
            color,
            fade_rate: 1.0,
            gravity: 0.0,
            trail: Vec::new(),
            has_trail: false,
        }
    }

    pub fn with_velocity(mut self, vx: f32, vy: f32) -> Self {
        self.vx = vx;
        self.vy = vy;
        self
    }

    pub fn with_life(mut self, life: f32) -> Self {
        self.max_life = life;
        self.life = life;
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn with_fade_rate(mut self, rate: f32) -> Self {
        self.fade_rate = rate;
        self
    }

    pub fn with_gravity(mut self, gravity: f32) -> Self {
        self.gravity = gravity;
        self
    }

    pub fn with_trail(mut self, trail: bool) -> Self {
        self.has_trail = trail;
        self
    }

    pub fn update(&mut self, delta: Duration) -> bool {
        let dt = delta.as_secs_f32();

        // Update position
        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // Apply gravity
        self.vy += self.gravity * dt;

        // Update life
        self.life -= self.fade_rate * dt;

        // Update trail
        if self.has_trail && dt > 0.0 {
            self.trail.push((self.x, self.y));
            if self.trail.len() > 10 {
                self.trail.remove(0);
            }
        }

        self.life > 0.0
    }

    pub fn is_alive(&self) -> bool {
        self.life > 0.0
    }

    pub fn alpha(&self) -> u8 {
        ((self.life / self.max_life) * 255.0) as u8
    }

    pub fn current_size(&self) -> f32 {
        self.size * (self.life / self.max_life).max(0.1)
    }
}

// ============================================================================
// Particle Emitter
// ============================================================================

/// Emits particles with specific properties
pub struct ParticleEmitter {
    /// Emitter position
    pub x: f32,
    pub y: f32,
    /// Emission rate (particles per second)
    pub rate: f32,
    /// Particle color
    pub color: Color,
    /// Particle lifetime range
    pub lifetime_range: (f32, f32),
    /// Velocity range (vx_min, vx_max, vy_min, vy_max)
    pub velocity_range: (f32, f32, f32, f32),
    /// Size range
    pub size_range: (f32, f32),
    /// Gravity
    pub gravity: f32,
    /// Emit enabled
    pub enabled: bool,
    /// Time since last emit
    emit_timer: f32,
}

impl ParticleEmitter {
    pub fn new(x: f32, y: f32, color: Color) -> Self {
        Self {
            x,
            y,
            rate: 10.0,
            color,
            lifetime_range: (0.5, 1.5),
            velocity_range: (-50.0, 50.0, -50.0, 50.0),
            size_range: (1.0, 3.0),
            gravity: 0.0,
            enabled: true,
            emit_timer: 0.0,
        }
    }

    pub fn with_rate(mut self, rate: f32) -> Self {
        self.rate = rate;
        self
    }

    pub fn with_lifetime(mut self, min: f32, max: f32) -> Self {
        self.lifetime_range = (min, max);
        self
    }

    pub fn with_velocity(mut self, vx_min: f32, vx_max: f32, vy_min: f32, vy_max: f32) -> Self {
        self.velocity_range = (vx_min, vx_max, vy_min, vy_max);
        self
    }

    pub fn with_size(mut self, min: f32, max: f32) -> Self {
        self.size_range = (min, max);
        self
    }

    pub fn with_gravity(mut self, gravity: f32) -> Self {
        self.gravity = gravity;
        self
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn emit(&mut self, delta: Duration) -> Option<Particle> {
        if !self.enabled {
            return None;
        }

        let dt = delta.as_secs_f32();
        self.emit_timer += dt;

        let particles_to_emit = (self.emit_timer * self.rate) as i32;
        self.emit_timer %= 1.0 / self.rate;

        if particles_to_emit > 0 {
            let mut rng = fastrand::Rng::new();

            let vx = rng.f32_range(self.velocity_range.0, self.velocity_range.1);
            let vy = rng.f32_range(self.velocity_range.2, self.velocity_range.3);
            let life = rng.f32_range(self.lifetime_range.0, self.lifetime_range.1);
            let size = rng.f32_range(self.size_range.0, self.size_range.1);

            Some(
                Particle::new(self.x, self.y, self.color)
                    .with_velocity(vx, vy)
                    .with_life(life)
                    .with_size(size)
                    .with_gravity(self.gravity),
            )
        } else {
            None
        }
    }
}

// ============================================================================
// Particle System
// ============================================================================

/// Manages multiple particle effects
pub struct ParticleSystem {
    /// Configuration
    config: EffectsConfig,
    /// Active particles
    particles: Vec<Particle>,
    /// Emitters
    emitters: HashMap<String, ParticleEmitter>,
}

impl ParticleSystem {
    pub fn new(config: EffectsConfig) -> Self {
        Self {
            config,
            particles: Vec::new(),
            emitters: HashMap::new(),
        }
    }

    pub fn add_emitter(&mut self, id: String, emitter: ParticleEmitter) {
        self.emitters.insert(id, emitter);
    }

    pub fn remove_emitter(&mut self, id: &str) {
        self.emitters.remove(id);
    }

    pub fn get_emitter_mut(&mut self, id: &str) -> Option<&mut ParticleEmitter> {
        self.emitters.get_mut(id)
    }

    pub fn emit_particles(&mut self, id: &str, delta: Duration) {
        if let Some(emitter) = self.emitters.get_mut(id) {
            while self.particles.len() < self.config.max_particles {
                if let Some(particle) = emitter.emit(delta) {
                    self.particles.push(particle);
                } else {
                    break;
                }
            }
        }
    }

    pub fn add_particle(&mut self, particle: Particle) {
        if self.particles.len() < self.config.max_particles {
            self.particles.push(particle);
        }
    }

    pub fn add_particles(&mut self, particles: Vec<Particle>) {
        let remaining = self.config.max_particles.saturating_sub(self.particles.len());
        self.particles.extend(particles.into_iter().take(remaining));
    }

    pub fn update(&mut self, delta: Duration) {
        // Update emitters
        for emitter in self.emitters.values_mut() {
            if let Some(particle) = emitter.emit(delta) {
                if self.particles.len() < self.config.max_particles {
                    self.particles.push(particle);
                }
            }
        }

        // Update particles
        self.particles.retain(|p| p.is_alive());
        for particle in &mut self.particles {
            particle.update(delta);
        }
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new(EffectsConfig::default())
    }
}

// ============================================================================
// Common Particle Effects
// ============================================================================

/// Pre-configured particle effects for common UI events
pub struct ParticleEffects;

impl ParticleEffects {
    /// Create explosion effect
    pub fn explosion(x: f32, y: f32, color: Color, count: usize) -> Vec<Particle> {
        let mut particles = Vec::new();
        let mut rng = fastrand::Rng::new();

        for _ in 0..count {
            let angle = rng.f32() * std::f32::consts::PI * 2.0;
            let speed = rng.f32_range(50.0, 150.0);
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;

            particles.push(
                Particle::new(x, y, color)
                    .with_velocity(vx, vy)
                    .with_life(rng.f32_range(0.3, 0.8))
                    .with_size(rng.f32_range(1.0, 3.0))
                    .with_gravity(200.0),
            );
        }

        particles
    }

    /// Create spark effect
    pub fn sparks(x: f32, y: f32, color: Color, count: usize) -> Vec<Particle> {
        let mut particles = Vec::new();
        let mut rng = fastrand::Rng::new();

        for _ in 0..count {
            let vx = rng.f32_range(-100.0, 100.0);
            let vy = rng.f32_range(-200.0, -50.0);

            particles.push(
                Particle::new(x, y, color)
                    .with_velocity(vx, vy)
                    .with_life(rng.f32_range(0.2, 0.5))
                    .with_size(rng.f32_range(0.5, 1.5))
                    .with_gravity(500.0),
            );
        }

        particles
    }

    /// Create trail effect
    pub fn trail(x: f32, y: f32, color: Color, count: usize) -> Vec<Particle> {
        let mut particles = Vec::new();
        let mut rng = fastrand::Rng::new();

        for i in 0..count {
            particles.push(
                Particle::new(x, y, color)
                    .with_velocity(0.0, 0.0)
                    .with_life(0.1 * (count - i) as f32)
                    .with_size(rng.f32_range(1.0, 2.0))
                    .with_trail(true),
            );
        }

        particles
    }

    /// Create dust effect
    pub fn dust(x: f32, y: f32, color: Color, count: usize) -> Vec<Particle> {
        let mut particles = Vec::new();
        let mut rng = fastrand::Rng::new();

        for _ in 0..count {
            particles.push(
                Particle::new(x, y, color)
                    .with_velocity(
                        rng.f32_range(-20.0, 20.0),
                        rng.f32_range(-30.0, -10.0),
                    )
                    .with_life(rng.f32_range(1.0, 2.0))
                    .with_size(rng.f32_range(0.5, 1.5))
                    .with_gravity(50.0),
            );
        }

        particles
    }
}

// ============================================================================
// Screen Shake
// ============================================================================

/// Screen shake effect
pub struct ScreenShake {
    /// Shake intensity
    intensity: f32,
    /// Shake duration
    duration: Duration,
    /// Current time
    current_time: Duration,
    /// Decay rate
    decay: f32,
    /// Active state
    active: bool,
}

impl ScreenShake {
    pub fn new(intensity: f32, duration: Duration) -> Self {
        Self {
            intensity,
            duration,
            current_time: Duration::ZERO,
            decay: 1.0,
            active: false,
        }
    }

    pub fn start(&mut self, intensity: f32, duration: Duration) {
        self.intensity = intensity;
        self.duration = duration;
        self.current_time = Duration::ZERO;
        self.active = true;
    }

    pub fn update(&mut self, delta: Duration) -> Option<(i16, i16)> {
        if !self.active {
            return None;
        }

        self.current_time += delta;

        if self.current_time >= self.duration {
            self.active = false;
            return None;
        }

        // Calculate current intensity based on decay
        let progress = self.current_time.as_secs_f32() / self.duration.as_secs_f32();
        let current_intensity = self.intensity * (1.0 - progress).max(0.0);

        // Generate shake offset
        let mut rng = fastrand::Rng::new();
        let offset_x = (rng.f32_range(-1.0, 1.0) * current_intensity) as i16;
        let offset_y = (rng.f32_range(-1.0, 1.0) * current_intensity) as i16;

        Some((offset_x, offset_y))
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn stop(&mut self) {
        self.active = false;
    }
}

// ============================================================================
// Glow Effect
// ============================================================================

/// Glow effect for elements
pub struct GlowEffect {
    /// Glow intensity (0.0 to 1.0)
    intensity: f32,
    /// Glow radius
    radius: u16,
    /// Glow color
    color: Color,
    /// Pulsing enabled
    pulsing: bool,
    /// Pulse speed
    pulse_speed: f32,
    /// Pulse phase
    pulse_phase: f32,
}

impl GlowEffect {
    pub fn new(color: Color) -> Self {
        Self {
            intensity: 0.5,
            radius: 2,
            color,
            pulsing: false,
            pulse_speed: 2.0,
            pulse_phase: 0.0,
        }
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.clamp(0.0, 1.0);
        self
    }

    pub fn with_radius(mut self, radius: u16) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_pulsing(mut self, pulsing: bool, speed: f32) -> Self {
        self.pulsing = pulsing;
        self.pulse_speed = speed;
        self
    }

    pub fn update(&mut self, delta: Duration) {
        if self.pulsing {
            let dt = delta.as_secs_f32();
            self.pulse_phase += dt * self.pulse_speed;
        }
    }

    pub fn current_intensity(&self) -> f32 {
        if self.pulsing {
            self.intensity * (0.5 + 0.5 * (self.pulse_phase * std::f32::consts::PI * 2.0).sin())
        } else {
            self.intensity
        }
    }

    pub fn style(&self) -> Style {
        let alpha = (self.current_intensity() * 255.0) as u8;
        Style::default().fg(self.color)
    }

    pub fn apply_to_area(&self, area: Rect) -> Vec<Rect> {
        let mut result = Vec::new();
        let intensity = self.current_intensity();

        for layer in 0..self.radius {
            let layer_intensity = intensity * (1.0 - layer as f32 / self.radius as f32);
            if layer_intensity > 0.01 {
                result.push(Rect {
                    x: area.x.saturating_sub(layer),
                    y: area.y.saturating_sub(layer),
                    width: area.width + layer * 2,
                    height: area.height + layer * 2,
                });
            }
        }

        result
    }
}

// ============================================================================
// Flash Effect
// ============================================================================

/// Screen flash effect
pub struct FlashEffect {
    /// Flash color
    color: Color,
    /// Duration
    duration: Duration,
    /// Current time
    current_time: Duration,
    /// Active state
    active: bool,
}

impl FlashEffect {
    pub fn new(color: Color, duration: Duration) -> Self {
        Self {
            color,
            duration,
            current_time: Duration::ZERO,
            active: false,
        }
    }

    pub fn trigger(&mut self, color: Color, duration: Duration) {
        self.color = color;
        self.duration = duration;
        self.current_time = Duration::ZERO;
        self.active = true;
    }

    pub fn update(&mut self, delta: Duration) -> Option<(Color, u8)> {
        if !self.active {
            return None;
        }

        self.current_time += delta;

        if self.current_time >= self.duration {
            self.active = false;
            return None;
        }

        // Calculate fade out
        let progress = self.current_time.as_secs_f32() / self.duration.as_secs_f32();
        let alpha = ((1.0 - progress) * 255.0) as u8;

        Some((self.color, alpha))
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

// ============================================================================
// Text Typewriter Effect
// ============================================================================

/// Typewriter effect for text display
pub struct TypewriterEffect {
    /// Full text
    text: String,
    /// Current display length
    current_length: usize,
    /// Characters per second
    pub speed: f32,
    /// Random variation in speed
    pub variation: f32,
    /// Time accumulator
    accumulator: f32,
    /// Complete state
    complete: bool,
}

impl TypewriterEffect {
    pub fn new(text: String) -> Self {
        Self {
            text,
            current_length: 0,
            speed: 20.0,
            variation: 0.2,
            accumulator: 0.0,
            complete: false,
        }
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn with_variation(mut self, variation: f32) -> Self {
        self.variation = variation;
        self
    }

    pub fn update(&mut self, delta: Duration) {
        if self.complete {
            return;
        }

        let dt = delta.as_secs_f32();
        let mut rng = fastrand::Rng::new();

        self.accumulator += dt * self.speed * (1.0 + rng.f32_range(-self.variation, self.variation));

        while self.accumulator >= 1.0 && self.current_length < self.text.len() {
            self.accumulator -= 1.0;
            self.current_length += 1;

            if self.current_length >= self.text.len() {
                self.complete = true;
                break;
            }
        }
    }

    pub fn current_text(&self) -> &str {
        &self.text[..self.current_length.min(self.text.len())]
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn reset(&mut self) {
        self.current_length = 0;
        self.accumulator = 0.0;
        self.complete = false;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effects_config_default() {
        let config = EffectsConfig::default();
        assert!(config.enable_particles);
        assert!(config.enable_shake);
        assert_eq!(config.max_particles, 100);
    }

    #[test]
    fn test_particle_creation() {
        let particle = Particle::new(10.0, 20.0, Color::Red)
            .with_velocity(5.0, 10.0)
            .with_life(1.0)
            .with_size(2.0);

        assert_eq!(particle.x, 10.0);
        assert_eq!(particle.y, 20.0);
        assert_eq!(particle.vx, 5.0);
        assert_eq!(particle.vy, 10.0);
        assert_eq!(particle.size, 2.0);
    }

    #[test]
    fn test_particle_update() {
        let mut particle = Particle::new(0.0, 0.0, Color::Red)
            .with_velocity(10.0, 20.0)
            .with_life(1.0)
            .with_fade_rate(1.0);

        let alive = particle.update(Duration::from_millis(500));
        assert!(alive);

        let alive = particle.update(Duration::from_millis(600));
        assert!(!alive);
    }

    #[test]
    fn test_particle_emitter() {
        let mut emitter = ParticleEmitter::new(0.0, 0.0, Color::Red)
            .with_rate(10.0);

        let particle = emitter.emit(Duration::from_millis(100));
        assert!(particle.is_some());

        let particle = emitter.emit(Duration::from_millis(1));
        assert!(particle.is_none());
    }

    #[test]
    fn test_particle_system() {
        let mut system = ParticleSystem::default();

        let particles = ParticleEffects::explosion(50.0, 50.0, Color::Red, 10);
        system.add_particles(particles);

        assert_eq!(system.particle_count(), 10);

        system.update(Duration::from_millis(100));
        assert!(system.particle_count() > 0);
    }

    #[test]
    fn test_screen_shake() {
        let mut shake = ScreenShake::new(5.0, Duration::from_millis(100));
        assert!(!shake.is_active());

        shake.start(5.0, Duration::from_millis(100));
        assert!(shake.is_active());

        let offset = shake.update(Duration::from_millis(50));
        assert!(offset.is_some());

        let offset = shake.update(Duration::from_millis(100));
        assert!(!shake.is_active());
    }

    #[test]
    fn test_glow_effect() {
        let mut glow = GlowEffect::new(Color::Blue)
            .with_intensity(0.7)
            .with_radius(3)
            .with_pulsing(true, 1.0);

        glow.update(Duration::from_millis(100));

        let intensity = glow.current_intensity();
        assert!(intensity >= 0.0 && intensity <= 1.0);
    }

    #[test]
    fn test_typewriter_effect() {
        let mut effect = TypewriterEffect::new("Hello, World!".to_string())
            .with_speed(20.0);

        assert_eq!(effect.current_text(), "");
        assert!(!effect.is_complete());

        effect.update(Duration::from_millis(100));
        assert!(effect.current_text().len() > 0);

        effect.update(Duration::from_secs(1));
        assert!(effect.is_complete());
        assert_eq!(effect.current_text(), "Hello, World!");
    }

    #[test]
    fn test_particle_effects_explosion() {
        let particles = ParticleEffects::explosion(50.0, 50.0, Color::Red, 20);
        assert_eq!(particles.len(), 20);

        for p in &particles {
            assert!(p.vx.abs() > 0.0 || p.vy.abs() > 0.0);
        }
    }
}
