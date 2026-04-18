// ai/time_management.rs - Time management for AI search
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Time control type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeControl {
    /// Fixed time per move (milliseconds)
    FixedTime(u64),
    /// Tournament time control (moves in time)
    Tournament { moves: u32, time: u64, increment: u64 },
    /// Sudden death (total time)
    SuddenDeath(u64),
    /// No time limit
    Unlimited,
}

impl TimeControl {
    /// Create fixed time control
    pub fn fixed(milliseconds: u64) -> Self {
        Self::FixedTime(milliseconds)
    }

    /// Create tournament time control
    pub fn tournament(moves: u32, time_sec: u64, increment_sec: u64) -> Self {
        Self::Tournament {
            moves,
            time: time_sec * 1000,
            increment: increment_sec * 1000,
        }
    }

    /// Create sudden death time control
    pub fn sudden_death(seconds: u64) -> Self {
        Self::SuddenDeath(seconds * 1000)
    }

    /// Get initial time budget
    pub fn initial_budget(&self) -> Option<Duration> {
        match self {
            Self::FixedTime(ms) => Some(Duration::from_millis(*ms)),
            Self::Tournament { time, .. } => Some(Duration::from_millis(*time)),
            Self::SuddenDeath(ms) => Some(Duration::from_millis(*ms)),
            Self::Unlimited => None,
        }
    }

    /// Get increment
    pub fn increment(&self) -> Duration {
        match self {
            Self::Tournament { increment, .. } => Duration::from_millis(*increment),
            _ => Duration::ZERO,
        }
    }
}

/// Time management configuration
#[derive(Debug, Clone)]
pub struct TimeManagementConfig {
    /// Base time allocation percentage
    pub base_allocation: f64,
    /// Emergency time threshold (when to panic)
    pub emergency_threshold: f64,
    /// Depth scaling factor
    pub depth_scaling: f64,
    /// Branching factor estimate
    pub branching_factor: f64,
    /// Minimum search time
    pub min_search_ms: u64,
    /// Maximum search time
    pub max_search_ms: u64,
}

impl Default for TimeManagementConfig {
    fn default() -> Self {
        Self {
            base_allocation: 0.05,
            emergency_threshold: 0.1,
            depth_scaling: 1.5,
            branching_factor: 7.0,
            min_search_ms: 10,
            max_search_ms: 60000,
        }
    }
}

/// Time manager for AI search
pub struct TimeManager {
    /// Time control
    time_control: TimeControl,
    /// Configuration
    config: TimeManagementConfig,
    /// Start time of search
    start_time: Instant,
    /// Time budget for current search
    time_budget: Duration,
    /// Flag to indicate stop
    stop_flag: Arc<AtomicBool>,
    /// Number of moves made
    moves_made: u32,
    /// Total time used
    total_time_used: Duration,
}

impl TimeManager {
    /// Create a new time manager
    pub fn new(time_control: TimeControl, config: TimeManagementConfig) -> Self {
        let initial_budget = time_control.initial_budget().unwrap_or(Duration::from_secs(60));

        Self {
            time_control,
            config,
            start_time: Instant::now(),
            time_budget: initial_budget,
            stop_flag: Arc::new(AtomicBool::new(false)),
            moves_made: 0,
            total_time_used: Duration::ZERO,
        }
    }

    /// Create with default configuration
    pub fn with_default_config(time_control: TimeControl) -> Self {
        Self::new(time_control, TimeManagementConfig::default())
    }

    /// Start a new search
    pub fn start_search(&mut self) {
        self.start_time = Instant::now();
        self.stop_flag.store(false, Ordering::Relaxed);

        // Calculate time budget for this move
        self.time_budget = self.calculate_budget();
    }

    /// Calculate time budget for current move
    fn calculate_budget(&self) -> Duration {
        let remaining = self.get_remaining_time();
        if remaining.is_zero() {
            return Duration::ZERO;
        }

        // Base allocation
        let budget = remaining.mul_f64(self.config.base_allocation);

        // Apply depth scaling for deeper searches
        let scaled = budget.mul_f64(1.0 / self.config.depth_scaling);

        // Clamp to min/max
        let min_time = Duration::from_millis(self.config.min_search_ms);
        let max_time = Duration::from_millis(self.config.max_search_ms);

        scaled.max(min_time).min(max_time).min(remaining)
    }

    /// Get remaining time
    fn get_remaining_time(&self) -> Duration {
        match self.time_control {
            TimeControl::FixedTime(_) => Duration::from_millis(self.time_budget.as_millis() as u64),
            TimeControl::Tournament { moves, .. } => {
                let moves_per_game = moves as f64;
                let avg_time = self.total_time_used.div_f64(self.moves_made as f64);
                let estimated_total = avg_time.mul_f64(moves_per_game);
                let remaining = self.time_budget.saturating_sub(self.total_time_used);
                remaining.max(Duration::from_millis(100))
            }
            TimeControl::SuddenDeath(_) => {
                self.time_budget.saturating_sub(self.total_time_used)
            }
            TimeControl::Unlimited => Duration::from_secs(86400), // 24 hours
        }
    }

    /// Check if time is up
    pub fn is_time_up(&self) -> bool {
        if self.stop_flag.load(Ordering::Relaxed) {
            return true;
        }

        match self.time_control {
            TimeControl::Unlimited => false,
            _ => {
                let elapsed = self.elapsed();
                elapsed >= self.time_budget
            }
        }
    }

    /// Check for emergency (very low time remaining)
    pub fn is_emergency(&self) -> bool {
        match self.time_control {
            TimeControl::Unlimited => false,
            _ => {
                let elapsed = self.elapsed();
                let remaining = self.time_budget.saturating_sub(elapsed);
                let threshold = self.time_budget.mul_f64(self.config.emergency_threshold);
                remaining <= threshold
            }
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get time budget
    pub fn time_budget(&self) -> Duration {
        self.time_budget
    }

    /// Get remaining time budget
    pub fn remaining_budget(&self) -> Duration {
        self.time_budget.saturating_sub(self.elapsed())
    }

    /// Get time usage as percentage
    pub fn time_usage_percentage(&self) -> f64 {
        if self.time_budget.is_zero() {
            0.0
        } else {
            self.elapsed().as_secs_f64() / self.time_budget.as_secs_f64() * 100.0
        }
    }

    /// Stop the search
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    /// Get stop flag
    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop_flag)
    }

    /// Update after making a move
    pub fn update_move(&mut self) {
        let elapsed = self.elapsed();
        self.total_time_used += elapsed;
        self.moves_made += 1;

        // Add increment if applicable
        let increment = self.time_control.increment();
        if !increment.is_zero() {
            self.time_budget += increment;
        }
    }

    /// Estimate time for next depth
    pub fn estimate_next_depth(&self, current_depth: u32, time_used: Duration) -> Duration {
        let branching = self.config.branching_factor;
        let depth_increase = branching.powi(current_depth as i32);
        time_used.mul_f64(depth_increase)
    }

    /// Check if we can search deeper
    pub fn can_search_deeper(&self, current_depth: u32, time_used: Duration) -> bool {
        let estimated = self.estimate_next_depth(current_depth, time_used);
        estimated < self.remaining_budget()
    }

    /// Get statistics
    pub fn stats(&self) -> TimeManagerStats {
        TimeManagerStats {
            elapsed: self.elapsed(),
            remaining: self.remaining_budget(),
            budget: self.time_budget,
            moves_made: self.moves_made,
            total_time_used: self.total_time_used,
            time_control: self.time_control,
        }
    }
}

/// Time manager statistics
#[derive(Debug, Clone)]
pub struct TimeManagerStats {
    /// Time elapsed in current search
    pub elapsed: Duration,
    /// Remaining time budget
    pub remaining: Duration,
    /// Total time budget
    pub budget: Duration,
    /// Number of moves made
    pub moves_made: u32,
    /// Total time used so far
    pub total_time_used: Duration,
    /// Time control type
    pub time_control: TimeControl,
}

impl TimeManagerStats {
    /// Display statistics
    pub fn display(&self) {
        println!("Time Management Stats:");
        println!("  Elapsed: {:?}", self.elapsed);
        println!("  Remaining: {:?}", self.remaining);
        println!("  Budget: {:?}", self.budget);
        println!("  Moves made: {}", self.moves_made);
        println!("  Total time used: {:?}", self.total_time_used);
    }
}

/// Adaptive time manager that adjusts based on position complexity
pub struct AdaptiveTimeManager {
    /// Base time manager
    time_manager: TimeManager,
    /// Complexity evaluator
    complexity: ComplexityEvaluator,
}

impl AdaptiveTimeManager {
    /// Create a new adaptive time manager
    pub fn new(time_control: TimeControl, config: TimeManagementConfig) -> Self {
        Self {
            time_manager: TimeManager::new(time_control, config),
            complexity: ComplexityEvaluator::new(),
        }
    }

    /// Start a new search with complexity consideration
    pub fn start_search(&mut self, board: &crate::core::Board, player: crate::core::Player) {
        self.time_manager.start_search();

        // Adjust time budget based on complexity
        let complexity_score = self.complexity.evaluate(board, player);
        let adjustment = self.adjust_for_complexity(complexity_score);

        // Adjust budget
        self.time_manager.time_budget = self.time_manager.time_budget.mul_f64(adjustment);
    }

    /// Adjust time budget based on complexity
    fn adjust_for_complexity(&self, complexity: f64) -> f64 {
        // Scale between 0.5x and 2.0x based on complexity
        0.5 + (complexity.min(1.0).max(0.0) * 1.5)
    }

    /// Delegate to time manager
    pub fn is_time_up(&self) -> bool {
        self.time_manager.is_time_up()
    }

    pub fn elapsed(&self) -> Duration {
        self.time_manager.elapsed()
    }

    pub fn stop(&self) {
        self.time_manager.stop();
    }

    pub fn update_move(&mut self) {
        self.time_manager.update_move();
    }
}

/// Complexity evaluator for adaptive time management
pub struct ComplexityEvaluator {
    /// Weights for different complexity factors
    weights: ComplexityWeights,
}

/// Complexity evaluation weights
#[derive(Debug, Clone)]
pub struct ComplexityWeights {
    /// Weight for piece mobility
    pub mobility_weight: f64,
    /// Weight for threats on board
    pub threat_weight: f64,
    /// Weight for tactical opportunities
    pub tactical_weight: f64,
    /// Weight for piece density
    pub density_weight: f64,
}

impl Default for ComplexityWeights {
    fn default() -> Self {
        Self {
            mobility_weight: 0.3,
            threat_weight: 0.4,
            tactical_weight: 0.2,
            density_weight: 0.1,
        }
    }
}

impl ComplexityEvaluator {
    /// Create a new complexity evaluator
    pub fn new() -> Self {
        Self {
            weights: ComplexityWeights::default(),
        }
    }

    /// Evaluate position complexity (0.0 to 1.0)
    pub fn evaluate(&self, board: &crate::core::Board, _player: crate::core::Player) -> f64 {
        let mut complexity = 0.0;

        // Mobility complexity
        let mobility = self.evaluate_mobility(board);
        complexity += mobility * self.weights.mobility_weight;

        // Threat complexity
        let threats = self.evaluate_threats(board);
        complexity += threats * self.weights.threat_weight;

        // Tactical complexity
        let tactical = self.evaluate_tactical(board);
        complexity += tactical * self.weights.tactical_weight;

        // Density complexity
        let density = self.evaluate_density(board);
        complexity += density * self.weights.density_weight;

        complexity.min(1.0).max(0.0)
    }

    /// Evaluate mobility complexity
    fn evaluate_mobility(&self, board: &crate::core::Board) -> f64 {
        let valid_moves = board.valid_moves().len();
        let max_moves = board.cols();
        valid_moves as f64 / max_moves as f64
    }

    /// Evaluate threat complexity
    fn evaluate_threats(&self, _board: &crate::core::Board) -> f64 {
        // Simplified threat detection
        0.3
    }

    /// Evaluate tactical complexity
    fn evaluate_tactical(&self, _board: &crate::core::Board) -> f64 {
        // Simplified tactical detection
        0.2
    }

    /// Evaluate density complexity
    fn evaluate_density(&self, board: &crate::core::Board) -> f64 {
        let filled = board.rows() * board.cols() - board.empty_count();
        let total = board.rows() * board.cols();
        filled as f64 / total as f64
    }
}

impl Default for ComplexityEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Time allocation strategy
#[derive(Debug, Clone, Copy)]
pub enum AllocationStrategy {
    /// Equal time distribution
    Equal,
    /// Progressive (more time for critical moves)
    Progressive,
    /// Aggressive (use more time early)
    Aggressive,
    /// Conservative (conserve time)
    Conservative,
}

/// Time allocator for distributing time across moves
pub struct TimeAllocator {
    /// Strategy
    strategy: AllocationStrategy,
    /// Total time
    total_time: Duration,
    /// Number of moves remaining
    moves_remaining: u32,
}

impl TimeAllocator {
    /// Create a new time allocator
    pub fn new(strategy: AllocationStrategy, total_time: Duration, moves_remaining: u32) -> Self {
        Self {
            strategy,
            total_time,
            moves_remaining,
        }
    }

    /// Allocate time for current move
    pub fn allocate(&self, move_number: u32) -> Duration {
        match self.strategy {
            AllocationStrategy::Equal => {
                self.total_time.div_f64(self.moves_remaining as f64)
            }
            AllocationStrategy::Progressive => {
                // More time for later moves
                let progress = move_number as f64 / (move_number + self.moves_remaining) as f64;
                let multiplier = 0.5 + progress;
                self.total_time.div_f64(self.moves_remaining as f64).mul_f64(multiplier)
            }
            AllocationStrategy::Aggressive => {
                // Use more time early
                let remaining_factor = self.moves_remaining as f64 / 20.0; // Assume 20 moves total
                self.total_time.div_f64(remaining_factor.max(1.0))
            }
            AllocationStrategy::Conservative => {
                // Conserve time
                self.total_time.div_f64(self.moves_remaining as f64).mul_f64(0.8)
            }
        }
    }
}

/// Time tracker for profiling
pub struct TimeTracker {
    /// Timer starts
    starts: Vec<(&'static str, Instant)>,
    /// Timer durations
    durations: Vec<(&'static str, Duration)>,
}

impl TimeTracker {
    /// Create a new time tracker
    pub fn new() -> Self {
        Self {
            starts: Vec::new(),
            durations: Vec::new(),
        }
    }

    /// Start a timer
    pub fn start(&mut self, name: &'static str) {
        self.starts.push((name, Instant::now()));
    }

    /// Stop a timer
    pub fn stop(&mut self, name: &'static str) -> Option<Duration> {
        if let Some(idx) = self.starts.iter().position(|(n, _)| *n == name) {
            let (_, start) = self.starts.remove(idx);
            let duration = start.elapsed();
            self.durations.push((name, duration));
            Some(duration)
        } else {
            None
        }
    }

    /// Get all durations
    pub fn durations(&self) -> &[(&'static str, Duration)] {
        &self.durations
    }

    /// Display timing information
    pub fn display(&self) {
        println!("Timing Information:");
        for (name, duration) in &self.durations {
            println!("  {}: {:?}", name, duration);
        }
    }
}

impl Default for TimeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_control_fixed() {
        let tc = TimeControl::fixed(1000);
        assert_eq!(tc.initial_budget(), Some(Duration::from_millis(1000)));
    }

    #[test]
    fn test_time_control_tournament() {
        let tc = TimeControl::tournament(40, 300, 10);
        assert_eq!(tc.increment(), Duration::from_secs(10));
    }

    #[test]
    fn test_time_manager_creation() {
        let tm = TimeManager::with_default_config(TimeControl::fixed(1000));
        assert_eq!(tm.moves_made, 0);
    }

    #[test]
    fn test_time_manager_start_search() {
        let mut tm = TimeManager::with_default_config(TimeControl::fixed(1000));
        tm.start_search();
        assert!(!tm.elapsed().is_zero());
    }

    #[test]
    fn test_time_manager_is_time_up() {
        let mut tm = TimeManager::with_default_config(TimeControl::fixed(1));
        tm.start_search();
        std::thread::sleep(Duration::from_millis(2));
        assert!(tm.is_time_up());
    }

    #[test]
    fn test_time_manager_unlimited() {
        let mut tm = TimeManager::with_default_config(TimeControl::Unlimited);
        tm.start_search();
        assert!(!tm.is_time_up());
    }

    #[test]
    fn test_time_tracker() {
        let mut tracker = TimeTracker::new();
        tracker.start("test");
        std::thread::sleep(Duration::from_millis(10));
        tracker.stop("test");

        let durations = tracker.durations();
        assert_eq!(durations.len(), 1);
        assert!(durations[0].1.as_millis() >= 10);
    }

    #[test]
    fn test_time_allocator() {
        let allocator = TimeAllocator::new(
            AllocationStrategy::Equal,
            Duration::from_secs(100),
            20
        );

        let allocated = allocator.allocate(10);
        assert_eq!(allocated, Duration::from_secs(5));
    }
}
