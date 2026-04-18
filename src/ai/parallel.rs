// ai/parallel.rs - Parallel search for AI algorithms
use crate::core::{Board, Player, Cell};
use crate::ai::minimax::MinimaxAI;
use crate::ai::alphabeta::AlphaBetaAI;
use crate::ai::evaluation::BoardEvaluator;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::collections::VecDeque;

/// Parallel search configuration
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of worker threads
    pub num_workers: usize,
    /// Enable work stealing
    pub enable_work_stealing: bool,
    /// Enable dynamic load balancing
    pub enable_load_balancing: bool,
    /// Chunk size for work distribution
    pub chunk_size: usize,
    /// Maximum queue depth
    pub max_queue_depth: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            num_workers: num_cpus::get(),
            enable_work_stealing: true,
            enable_load_balancing: true,
            chunk_size: 4,
            max_queue_depth: 100,
        }
    }
}

impl ParallelConfig {
    /// Create config for small scale
    pub fn small() -> Self {
        Self {
            num_workers: 2,
            chunk_size: 2,
            ..Default::default()
        }
    }

    /// Create config for medium scale
    pub fn medium() -> Self {
        Self {
            num_workers: 4,
            ..Default::default()
        }
    }

    /// Create config for large scale
    pub fn large() -> Self {
        Self {
            num_workers: num_cpus::get(),
            chunk_size: 8,
            ..Default::default()
        }
    }
}

/// Search task for parallel execution
#[derive(Debug, Clone)]
pub struct SearchTask {
    /// Task ID
    pub id: usize,
    /// Board position
    pub board: Arc<Board>,
    /// Player to move
    pub player: Player,
    /// Search depth
    pub depth: usize,
    /// Alpha value for alpha-beta pruning
    pub alpha: f64,
    /// Beta value for alpha-beta pruning
    pub beta: f64,
    /// Parent task ID
    pub parent_id: Option<usize>,
}

impl SearchTask {
    /// Create a new search task
    pub fn new(
        id: usize,
        board: Arc<Board>,
        player: Player,
        depth: usize,
        alpha: f64,
        beta: f64,
    ) -> Self {
        Self {
            id,
            board,
            player,
            depth,
            alpha,
            beta,
            parent_id: None,
        }
    }

    /// Create with parent
    pub fn with_parent(
        id: usize,
        board: Arc<Board>,
        player: Player,
        depth: usize,
        alpha: f64,
        beta: f64,
        parent_id: usize,
    ) -> Self {
        Self {
            id,
            board,
            player,
            depth,
            alpha,
            beta,
            parent_id: Some(parent_id),
        }
    }

    /// Create subtask for child search
    pub fn create_subtask(
        &self,
        id: usize,
        board: Arc<Board>,
        player: Player,
        depth: usize,
    ) -> Self {
        Self::with_parent(
            id,
            board,
            player,
            depth,
            self.alpha,
            self.beta,
            self.id,
        )
    }
}

/// Search result from a worker
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Task ID
    pub task_id: usize,
    /// Best column to play
    pub best_column: Option<usize>,
    /// Score for the position
    pub score: f64,
    /// Number of positions evaluated
    pub positions_evaluated: usize,
    /// Time taken
    pub time_ms: u64,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(task_id: usize, best_column: Option<usize>, score: f64) -> Self {
        Self {
            task_id,
            best_column,
            score,
            positions_evaluated: 0,
            time_ms: 0,
        }
    }
}

/// Parallel search worker
struct SearchWorker {
    /// Worker ID
    id: usize,
    /// Receiver for tasks
    task_receiver: Receiver<SearchTask>,
    /// Sender for results
    result_sender: Sender<SearchResult>,
    /// Evaluator for positions
    evaluator: MinimaxAI,
}

impl SearchWorker {
    /// Create a new search worker
    pub fn new(
        id: usize,
        task_receiver: Receiver<SearchTask>,
        result_sender: Sender<SearchResult>,
        evaluator: MinimaxAI,
    ) -> Self {
        Self {
            id,
            task_receiver,
            result_sender,
            evaluator,
        }
    }

    /// Run the worker loop
    pub fn run(&mut self) {
        while let Ok(task) = self.task_receiver.recv() {
            let start = std::time::Instant::now();
            let result = self.execute_task(&task);
            let elapsed = start.elapsed().as_millis() as u64;

            let mut search_result = result;
            search_result.time_ms = elapsed;

            let _ = self.result_sender.send(search_result);
        }
    }

    /// Execute a single task
    fn execute_task(&mut self, task: &SearchTask) -> SearchResult {
        let best_column = if task.depth == 0 {
            None
        } else {
            // Simple evaluation for demo - would use full search in production
            self.evaluator.find_best_move(&task.board, task.player, task.depth)
        };

        let score = if let Some(col) = best_column {
            // Make move and evaluate
            if let Some(row) = task.board.find_empty_row(col) {
                let mut new_board = (*task.board).clone();
                new_board.set(row, col, if task.player == Player::Player1 {
                    Cell::Player1
                } else {
                    Cell::Player2
                });

                let opponent = if task.player == Player::Player1 {
                    Player::Player2
                } else {
                    Player::Player1
                };

                self.evaluator.evaluate(&new_board, opponent)
            } else {
                0.0
            }
        } else {
            0.0
        };

        SearchResult::new(task.id, best_column, score)
    }
}

/// Parallel search engine
pub struct ParallelSearchEngine {
    /// Configuration
    config: ParallelConfig,
    /// Task sender
    task_sender: Option<Sender<SearchTask>>,
    /// Result receiver
    result_receiver: Option<Receiver<SearchResult>>,
    /// Workers
    workers: Vec<thread::JoinHandle<()>>,
    /// Task queue
    task_queue: Arc<Mutex<VecDeque<SearchTask>>>,
    /// Results storage
    results: Arc<RwLock<Vec<SearchResult>>>,
}

impl ParallelSearchEngine {
    /// Create a new parallel search engine
    pub fn new(config: ParallelConfig) -> Self {
        let (task_sender, task_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();
        let task_queue = Arc::new(Mutex::new(VecDeque::new()));
        let results = Arc::new(RwLock::new(Vec::new()));

        Self {
            config,
            task_sender: Some(task_sender),
            result_receiver: Some(result_receiver),
            workers: Vec::new(),
            task_queue,
            results,
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(ParallelConfig::default())
    }

    /// Start the workers
    pub fn start(&mut self) {
        let task_sender = self.task_sender.take().unwrap();
        let (result_sender, result_receiver) = mpsc::channel();

        self.result_receiver = Some(result_receiver);

        for worker_id in 0..self.config.num_workers {
            let worker_task_receiver = task_sender.clone();
            let worker_result_sender = result_sender.clone();
            let evaluator = MinimaxAI::new(4);

            let worker = SearchWorker::new(
                worker_id,
                worker_task_receiver,
                worker_result_sender,
                evaluator,
            );

            let handle = thread::spawn(move || {
                let mut w = worker;
                w.run();
            });

            self.workers.push(handle);
        }
    }

    /// Submit a search task
    pub fn submit_task(&self, task: SearchTask) {
        if let Ok(mut queue) = self.task_queue.lock() {
            queue.push_back(task);
        }

        if let Some(ref sender) = self.task_sender {
            let _ = sender.send(task);
        }
    }

    /// Submit multiple tasks
    pub fn submit_tasks(&self, tasks: Vec<SearchTask>) {
        for task in tasks {
            self.submit_task(task);
        }
    }

    /// Collect all results
    pub fn collect_results(&mut self) -> Vec<SearchResult> {
        let mut results = Vec::new();

        if let Some(ref receiver) = self.result_receiver {
            // Collect results for each worker
            for _ in 0..self.workers.len() {
                if let Ok(result) = receiver.recv_timeout(std::time::Duration::from_secs(10)) {
                    results.push(result);
                }
            }
        }

        results
    }

    /// Wait for all tasks to complete
    pub fn wait_for_completion(&mut self) {
        self.collect_results();
    }

    /// Get results
    pub fn get_results(&self) -> Vec<SearchResult> {
        if let Ok(results) = self.results.read() {
            results.clone()
        } else {
            Vec::new()
        }
    }

    /// Shutdown the engine
    pub fn shutdown(mut self) {
        // Workers will stop when senders are dropped
        self.task_sender = None;
        self.result_receiver = None;

        // Wait for workers to finish
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

impl Drop for ParallelSearchEngine {
    fn drop(&mut self) {
        self.task_sender = None;
        self.result_receiver = None;
    }
}

/// Parallel minimax with YBWC (Young-Brothers Wait Concept)
pub struct ParallelMinimax {
    /// Configuration
    config: ParallelConfig,
    /// Base minimax AI
    base_ai: MinimaxAI,
}

impl ParallelMinimax {
    /// Create a new parallel minimax
    pub fn new(config: ParallelConfig, base_ai: MinimaxAI) -> Self {
        Self { config, base_ai }
    }

    /// Create with default configuration
    pub fn with_default_ai(depth: usize) -> Self {
        Self::new(
            ParallelConfig::default(),
            MinimaxAI::new(depth),
        )
    }

    /// Find best move using parallel search
    pub fn find_best_move(
        &self,
        board: &Board,
        player: Player,
        depth: usize,
    ) -> Option<usize> {
        let valid_moves = board.valid_moves();
        if valid_moves.is_empty() {
            return None;
        }

        // Use parallel evaluation for moves
        let results: Vec<_> = valid_moves
            .par_iter() // Requires rayon in real implementation
            .map(|&col| {
                let score = self.evaluate_move(board, col, player, depth);
                (col, score)
            })
            .collect();

        // Find best move
        results.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(col, _)| col)
    }

    /// Evaluate a single move
    fn evaluate_move(&self, board: &Board, col: usize, player: Player, depth: usize) -> f64 {
        if let Some(row) = board.find_empty_row(col) {
            let mut new_board = board.clone();
            new_board.set(row, col, if player == Player::Player1 {
                Cell::Player1
            } else {
                Cell::Player2
            });

            let opponent = if player == Player::Player1 {
                Player::Player2
            } else {
                Player::Player1
            };

            self.base_ai.evaluate(&new_board, opponent)
        } else {
            0.0
        }
    }
}

/// Work stealing queue for dynamic load balancing
pub struct WorkStealingQueue {
    /// Local queue
    local_queue: VecDeque<SearchTask>,
    /// Stealable queue
    steal_queue: Arc<Mutex<VecDeque<SearchTask>>>,
    /// Queue ID
    id: usize,
}

impl WorkStealingQueue {
    /// Create a new work stealing queue
    pub fn new(id: usize, steal_queue: Arc<Mutex<VecDeque<SearchTask>>>) -> Self {
        Self {
            local_queue: VecDeque::new(),
            steal_queue,
            id,
        }
    }

    /// Push task to local queue
    pub fn push(&mut self, task: SearchTask) {
        self.local_queue.push_back(task);
    }

    /// Pop task from local queue
    pub fn pop(&mut self) -> Option<SearchTask> {
        self.local_queue.pop_front()
    }

    /// Steal task from shared queue
    pub fn steal(&mut self) -> Option<SearchTask> {
        if let Ok(mut queue) = self.steal_queue.lock() {
            queue.pop_front()
        } else {
            None
        }
    }

    /// Check if local queue is empty
    pub fn is_empty(&self) -> bool {
        self.local_queue.is_empty()
    }

    /// Get task count
    pub fn len(&self) -> usize {
        self.local_queue.len()
    }
}

/// Parallel search coordinator
pub struct SearchCoordinator {
    /// Engine
    engine: ParallelSearchEngine,
    /// Task counter
    task_counter: Arc<Mutex<usize>>,
    /// Results map
    results_map: Arc<RwLock<std::collections::HashMap<usize, SearchResult>>>,
}

impl SearchCoordinator {
    /// Create a new search coordinator
    pub fn new(config: ParallelConfig) -> Self {
        Self {
            engine: ParallelSearchEngine::new(config),
            task_counter: Arc::new(Mutex::new(0)),
            results_map: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Start the coordinator
    pub fn start(&mut self) {
        self.engine.start();
    }

    /// Submit search for board
    pub fn submit_search(&mut self, board: &Board, player: Player, depth: usize) -> usize {
        let mut counter = self.task_counter.lock().unwrap();
        let task_id = *counter;
        *counter += 1;
        drop(counter);

        let task = SearchTask::new(
            task_id,
            Arc::new(board.clone()),
            player,
            depth,
            f64::NEG_INFINITY,
            f64::INFINITY,
        );

        self.engine.submit_task(task);
        task_id
    }

    /// Get result for task
    pub fn get_result(&self, task_id: usize) -> Option<SearchResult> {
        if let Ok(map) = self.results_map.read() {
            map.get(&task_id).cloned()
        } else {
            None
        }
    }

    /// Wait for all results
    pub fn wait_all(&mut self) -> Vec<SearchResult> {
        self.engine.wait_for_completion()
    }

    /// Shutdown coordinator
    pub fn shutdown(self) {
        self.engine.shutdown();
    }
}

/// Parallel statistics
#[derive(Debug, Clone, Default)]
pub struct ParallelStats {
    /// Total tasks submitted
    pub tasks_submitted: usize,
    /// Tasks completed
    pub tasks_completed: usize,
    /// Total positions evaluated
    pub total_positions_evaluated: usize,
    /// Total time spent
    pub total_time_ms: u64,
    /// Average time per task
    pub avg_time_per_task_ms: f64,
}

impl ParallelStats {
    /// Update with task completion
    pub fn update_task(&mut self, time_ms: u64, positions_evaluated: usize) {
        self.tasks_completed += 1;
        self.total_time_ms += time_ms;
        self.total_positions_evaluated += positions_evaluated;

        if self.tasks_completed > 0 {
            self.avg_time_per_task_ms = self.total_time_ms as f64 / self.tasks_completed as f64;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_config() {
        let config = ParallelConfig::default();
        assert!(config.num_workers > 0);
    }

    #[test]
    fn test_search_task_creation() {
        let board = Arc::new(Board::new(6, 7));
        let task = SearchTask::new(1, board, Player::Player1, 5, -1.0, 1.0);

        assert_eq!(task.id, 1);
        assert_eq!(task.depth, 5);
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult::new(1, Some(3), 0.5);

        assert_eq!(result.task_id, 1);
        assert_eq!(result.best_column, Some(3));
        assert_eq!(result.score, 0.5);
    }

    #[test]
    fn test_parallel_search_engine_creation() {
        let engine = ParallelSearchEngine::default();
        assert_eq!(engine.config.num_workers, num_cpus::get());
    }

    #[test]
    fn test_work_stealing_queue() {
        let steal_queue = Arc::new(Mutex::new(VecDeque::new()));
        let mut queue = WorkStealingQueue::new(0, steal_queue);

        let task = SearchTask::new(
            1,
            Arc::new(Board::new(6, 7)),
            Player::Player1,
            5,
            -1.0,
            1.0,
        );

        queue.push(task);
        assert_eq!(queue.len(), 1);

        let popped = queue.pop();
        assert!(popped.is_some());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_parallel_stats() {
        let mut stats = ParallelStats::default();

        stats.tasks_submitted = 10;
        stats.update_task(100, 1000);

        assert_eq!(stats.tasks_completed, 1);
        assert_eq!(stats.total_time_ms, 100);
        assert_eq!(stats.total_positions_evaluated, 1000);
    }

    #[test]
    fn test_search_coordinator() {
        let mut coordinator = SearchCoordinator::new(ParallelConfig::small());
        coordinator.start();

        let board = Board::new(6, 7);
        let task_id = coordinator.submit_search(&board, Player::Player1, 3);

        assert_eq!(task_id, 0);
    }
}
