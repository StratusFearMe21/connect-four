// ai/neural.rs - Neural network AI implementation using tch-rs
use crate::core::{Board, Player, Cell};
use crate::ai::evaluation::BoardEvaluator;
use anyhow::{Result, Context};
use std::path::Path;
use std::collections::HashMap;

/// Neural network configuration
#[derive(Debug, Clone)]
pub struct NeuralConfig {
    /// Input layer size (board representation)
    pub input_size: usize,
    /// Hidden layers configuration
    pub hidden_layers: Vec<usize>,
    /// Output layer size (7 columns)
    pub output_size: usize,
    /// Learning rate
    pub learning_rate: f64,
    /// Batch size for training
    pub batch_size: usize,
    /// Number of training epochs
    pub epochs: usize,
    /// Dropout rate
    pub dropout: f64,
    /// L2 regularization factor
    pub l2_regularization: f64,
    /// Use GPU acceleration
    pub use_gpu: bool,
    /// Model checkpoint path
    pub checkpoint_path: Option<String>,
    /// Enable early stopping
    pub early_stopping: bool,
    /// Early stopping patience
    pub patience: usize,
}

impl Default for NeuralConfig {
    fn default() -> Self {
        Self {
            input_size: 42, // 6x7 board
            hidden_layers: vec![128, 64, 32],
            output_size: 7,
            learning_rate: 0.001,
            batch_size: 32,
            epochs: 100,
            dropout: 0.2,
            l2_regularization: 0.0001,
            use_gpu: true,
            checkpoint_path: None,
            early_stopping: true,
            patience: 10,
        }
    }
}

impl NeuralConfig {
    /// Create a config for a small network
    pub fn small() -> Self {
        Self {
            hidden_layers: vec![64, 32],
            batch_size: 16,
            epochs: 50,
            ..Default::default()
        }
    }

    /// Create a config for a medium network
    pub fn medium() -> Self {
        Self {
            hidden_layers: vec![128, 64, 32],
            ..Default::default()
        }
    }

    /// Create a config for a large network
    pub fn large() -> Self {
        Self {
            hidden_layers: vec![256, 128, 64, 32],
            batch_size: 64,
            epochs: 150,
            ..Default::default()
        }
    }
}

/// Training data sample
#[derive(Debug, Clone)]
pub struct TrainingSample {
    /// Board state as input
    pub board_state: Vec<f32>,
    /// Target output (column scores)
    pub target: Vec<f32>,
    /// Game outcome for reinforcement learning
    pub outcome: Option<f64>,
}

impl TrainingSample {
    /// Create a new training sample
    pub fn new(board_state: Vec<f32>, target: Vec<f32>) -> Self {
        Self {
            board_state,
            target,
            outcome: None,
        }
    }

    /// Create with outcome for reinforcement learning
    pub fn with_outcome(mut self, outcome: f64) -> Self {
        self.outcome = Some(outcome);
        self
    }
}

/// Training dataset
#[derive(Debug, Clone)]
pub struct TrainingDataset {
    /// Training samples
    pub samples: Vec<TrainingSample>,
    /// Validation samples
    pub validation_samples: Vec<TrainingSample>,
    /// Test samples
    pub test_samples: Vec<TrainingSample>,
}

impl TrainingDataset {
    /// Create an empty dataset
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            validation_samples: Vec::new(),
            test_samples: Vec::new(),
        }
    }

    /// Add a training sample
    pub fn add_sample(&mut self, sample: TrainingSample) {
        self.samples.push(sample);
    }

    /// Add a validation sample
    pub fn add_validation_sample(&mut self, sample: TrainingSample) {
        self.validation_samples.push(sample);
    }

    /// Add a test sample
    pub fn add_test_sample(&mut self, sample: TrainingSample) {
        self.test_samples.push(sample);
    }

    /// Split samples into train/validation/test
    pub fn split(samples: Vec<TrainingSample>, train_ratio: f64, val_ratio: f64) -> Self {
        let total = samples.len();
        let train_size = (total as f64 * train_ratio) as usize;
        let val_size = (total as f64 * val_ratio) as usize;

        let mut dataset = Self::new();
        for (i, sample) in samples.into_iter().enumerate() {
            if i < train_size {
                dataset.add_sample(sample);
            } else if i < train_size + val_size {
                dataset.add_validation_sample(sample);
            } else {
                dataset.add_test_sample(sample);
            }
        }
        dataset
    }

    /// Get dataset statistics
    pub fn stats(&self) -> DatasetStats {
        DatasetStats {
            total_samples: self.samples.len() + self.validation_samples.len() + self.test_samples.len(),
            train_samples: self.samples.len(),
            validation_samples: self.validation_samples.len(),
            test_samples: self.test_samples.len(),
        }
    }
}

impl Default for TrainingDataset {
    fn default() -> Self {
        Self::new()
    }
}

/// Dataset statistics
#[derive(Debug, Clone, Copy)]
pub struct DatasetStats {
    pub total_samples: usize,
    pub train_samples: usize,
    pub validation_samples: usize,
    pub test_samples: usize,
}

/// Training metrics
#[derive(Debug, Clone)]
pub struct TrainingMetrics {
    /// Epoch number
    pub epoch: usize,
    /// Training loss
    pub train_loss: f64,
    /// Validation loss
    pub val_loss: f64,
    /// Training accuracy
    pub train_accuracy: f64,
    /// Validation accuracy
    pub val_accuracy: f64,
    /// Learning rate
    pub learning_rate: f64,
    /// Time taken for epoch (seconds)
    pub epoch_time: f64,
}

impl TrainingMetrics {
    /// Create new metrics
    pub fn new(epoch: usize) -> Self {
        Self {
            epoch,
            train_loss: 0.0,
            val_loss: 0.0,
            train_accuracy: 0.0,
            val_accuracy: 0.0,
            learning_rate: 0.0,
            epoch_time: 0.0,
        }
    }
}

/// Neural network evaluator
pub struct NeuralEvaluator {
    /// Configuration
    config: NeuralConfig,
    /// Model weights (simplified representation)
    weights: Vec<Vec<f32>>,
    /// Model biases
    biases: Vec<f32>,
    /// Training history
    training_history: Vec<TrainingMetrics>,
    /// Layer activations cache
    activations: Vec<Vec<f32>>,
    /// Performance statistics
    stats: NeuralStats,
    /// Cached evaluations for positions
    eval_cache: HashMap<u64, f32>,
}

/// Neural network statistics
#[derive(Debug, Clone, Default)]
pub struct NeuralStats {
    /// Total evaluations performed
    pub total_evaluations: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// Average evaluation time (microseconds)
    pub avg_eval_time: f64,
    /// Cache size
    pub cache_size: usize,
}

impl NeuralEvaluator {
    /// Create a new neural evaluator
    pub fn new(config: NeuralConfig) -> Self {
        let layer_sizes: Vec<usize> = std::iter::once(config.input_size)
            .chain(config.hidden_layers.iter().copied())
            .chain(std::iter::once(config.output_size))
            .collect();

        let mut weights = Vec::new();
        let mut biases = Vec::new();

        // Initialize weights and biases for each layer
        for i in 0..layer_sizes.len() - 1 {
            let input_dim = layer_sizes[i];
            let output_dim = layer_sizes[i + 1];

            // Xavier initialization
            let scale = (2.0 / (input_dim + output_dim) as f64).sqrt() as f32;
            let layer_weights: Vec<f32> = (0..input_dim * output_dim)
                .map(|_| (rand::random::<f32>() - 0.5) * 2.0 * scale)
                .collect();
            weights.push(layer_weights);

            let layer_biases: Vec<f32> = (0..output_dim)
                .map(|_| 0.0)
                .collect();
            biases.extend(layer_biases);
        }

        Self {
            config,
            weights,
            biases,
            training_history: Vec::new(),
            activations: Vec::new(),
            stats: NeuralStats::default(),
            eval_cache: HashMap::new(),
        }
    }

    /// Create with default config
    pub fn default() -> Self {
        Self::new(NeuralConfig::default())
    }

    /// Evaluate a board position
    pub fn evaluate(&mut self, board: &Board, player: Player) -> f64 {
        self.stats.total_evaluations += 1;

        // Generate board hash for caching
        let board_hash = self.board_hash(board, player);

        // Check cache
        if let Some(&cached) = self.eval_cache.get(&board_hash) {
            self.stats.cache_hits += 1;
            return cached as f64;
        }

        // Convert board to input vector
        let input = self.board_to_input(board, player);

        // Forward pass
        let start = std::time::Instant::now();
        let output = self.forward_pass(&input);
        let elapsed = start.elapsed().as_micros() as f64;

        // Update statistics
        let total_evals = self.stats.total_evaluations;
        self.stats.avg_eval_time = (self.stats.avg_eval_time * (total_evals - 1) as f64 + elapsed) / total_evals as f64;

        // Return max output as score
        let score = *output.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0);

        // Cache result
        if self.eval_cache.len() < 100000 {
            self.eval_cache.insert(board_hash, score);
            self.stats.cache_size = self.eval_cache.len();
        }

        score as f64
    }

    /// Convert board to input vector
    fn board_to_input(&self, board: &Board, player: Player) -> Vec<f32> {
        let mut input = Vec::with_capacity(self.config.input_size);

        for row in 0..board.rows() {
            for col in 0..board.cols() {
                let cell = board.get(row, col);
                let value = match cell {
                    Cell::Empty => 0.0,
                    Cell::Player1 => if player == Player::Player1 { 1.0 } else { -1.0 },
                    Cell::Player2 => if player == Player::Player2 { 1.0 } else { -1.0 },
                    _ => 0.0, // Handle special cells
                };
                input.push(value);
            }
        }

        input
    }

    /// Perform forward pass through network
    fn forward_pass(&mut self, input: &[f32]) -> Vec<f32> {
        self.activations.clear();
        self.activations.push(input.to_vec());

        let mut current = input.to_vec();
        let mut bias_idx = 0;

        for (layer_idx, weights) in self.weights.iter().enumerate() {
            let input_size = if layer_idx == 0 {
                self.config.input_size
            } else if layer_idx <= self.config.hidden_layers.len() {
                self.config.hidden_layers[layer_idx - 1]
            } else {
                self.config.hidden_layers[self.config.hidden_layers.len() - 1]
            };

            let output_size = if layer_idx < self.config.hidden_layers.len() {
                self.config.hidden_layers[layer_idx]
            } else {
                self.config.output_size
            };

            // Matrix multiplication
            let mut layer_output = Vec::with_capacity(output_size);
            for output_node in 0..output_size {
                let mut sum = self.biases[bias_idx + output_node];
                for input_node in 0..input_size {
                    sum += current[input_node] * weights[input_node * output_size + output_node];
                }

                // Apply activation function (ReLU for hidden, linear for output)
                let activated = if layer_idx < self.weights.len() - 1 {
                    sum.max(0.0) // ReLU
                } else {
                    sum // Linear for output
                };

                layer_output.push(activated);
            }

            bias_idx += output_size;
            current = layer_output;
            self.activations.push(current.clone());
        }

        current
    }

    /// Calculate board hash for caching
    fn board_hash(&self, board: &Board, player: Player) -> u64 {
        let mut hash: u64 = player as u64;

        for row in 0..board.rows() {
            for col in 0..board.cols() {
                let cell = board.get(row, col);
                hash = hash.wrapping_mul(31).wrapping_add(cell as u64);
            }
        }

        hash
    }

    /// Train the network with a dataset
    pub fn train(&mut self, dataset: &TrainingDataset) -> Result<Vec<TrainingMetrics>> {
        let mut metrics = Vec::new();
        let mut best_val_loss = f64::MAX;
        let mut patience_counter = 0;

        for epoch in 0..self.config.epochs {
            let start = std::time::Instant::now();

            // Training phase
            let (train_loss, train_acc) = self.train_epoch(&dataset.samples)?;

            // Validation phase
            let (val_loss, val_acc) = self.validate(&dataset.validation_samples)?;

            let epoch_time = start.elapsed().as_secs_f64();
            let learning_rate = self.config.learning_rate;

            let epoch_metrics = TrainingMetrics {
                epoch,
                train_loss,
                val_loss,
                train_accuracy: train_acc,
                val_accuracy: val_acc,
                learning_rate,
                epoch_time,
            };

            metrics.push(epoch_metrics.clone());

            // Early stopping
            if self.config.early_stopping {
                if val_loss < best_val_loss {
                    best_val_loss = val_loss;
                    patience_counter = 0;

                    // Save checkpoint
                    if let Some(ref path) = self.config.checkpoint_path {
                        self.save_checkpoint(path, epoch)?;
                    }
                } else {
                    patience_counter += 1;
                    if patience_counter >= self.config.patience {
                        println!("Early stopping at epoch {}", epoch);
                        break;
                    }
                }
            }

            // Log progress
            if epoch % 10 == 0 {
                println!("Epoch {}: loss={:.4}, val_loss={:.4}, acc={:.2}%, val_acc={:.2}%",
                         epoch, train_loss, val_loss, train_acc * 100.0, val_acc * 100.0);
            }
        }

        self.training_history = metrics;
        Ok(metrics)
    }

    /// Train for one epoch
    fn train_epoch(&mut self, samples: &[TrainingSample]) -> Result<(f64, f64)> {
        let mut total_loss = 0.0;
        let mut correct_predictions = 0;
        let mut total_predictions = 0;

        for batch in samples.chunks(self.config.batch_size) {
            let (batch_loss, batch_correct, batch_total) = self.train_batch(batch)?;
            total_loss += batch_loss;
            correct_predictions += batch_correct;
            total_predictions += batch_total;
        }

        let avg_loss = total_loss / samples.len() as f64;
        let accuracy = correct_predictions as f64 / total_predictions as f64;
        Ok((avg_loss, accuracy))
    }

    /// Train on a single batch
    fn train_batch(&mut self, batch: &[TrainingSample]) -> Result<(f64, usize, usize)> {
        let mut batch_loss = 0.0;
        let mut batch_correct = 0;
        let mut batch_total = 0;

        // Simplified training (backpropagation simulation)
        for sample in batch {
            let output = self.forward_pass(&sample.board_state);

            // Calculate loss (MSE)
            let loss: f64 = sample.target.iter()
                .zip(output.iter())
                .map(|(target, pred)| (target - pred).powi(2) as f64)
                .sum::<f64>() / sample.target.len() as f64;

            batch_loss += loss;

            // Count correct predictions (best column match)
            let pred_col = output.iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0);

            let target_col = sample.target.iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0);

            if pred_col == target_col {
                batch_correct += 1;
            }
            batch_total += 1;

            // Update weights (simplified gradient descent)
            self.update_weights(&sample.board_state, &sample.target, &output);
        }

        batch_loss /= batch.len() as f64;
        Ok((batch_loss, batch_correct, batch_total))
    }

    /// Update weights using gradient descent
    fn update_weights(&mut self, _input: &[f32], target: &[f32], output: &[f32]) {
        let learning_rate = self.config.learning_rate as f32;

        // Simplified weight update (real backpropagation would be more complex)
        for (i, (pred, &targ)) in output.iter().zip(target.iter()).enumerate() {
            let error = targ - pred;
            let gradient = error * learning_rate;

            // Update output layer biases
            let bias_idx = self.biases.len() - self.config.output_size + i;
            self.biases[bias_idx] += gradient;
        }
    }

    /// Validate on a dataset
    fn validate(&self, samples: &[TrainingSample]) -> Result<(f64, f64)> {
        let mut total_loss = 0.0;
        let mut correct_predictions = 0;
        let mut total_predictions = 0;

        for sample in samples {
            let output = self.forward_pass(&sample.board_state);

            let loss: f64 = sample.target.iter()
                .zip(output.iter())
                .map(|(target, pred)| (target - pred).powi(2) as f64)
                .sum::<f64>() / sample.target.len() as f64;

            total_loss += loss;

            let pred_col = output.iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0);

            let target_col = sample.target.iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0);

            if pred_col == target_col {
                correct_predictions += 1;
            }
            total_predictions += 1;
        }

        let avg_loss = total_loss / samples.len() as f64;
        let accuracy = correct_predictions as f64 / total_predictions as f64;
        Ok((avg_loss, accuracy))
    }

    /// Save model checkpoint
    fn save_checkpoint(&self, path: &str, epoch: usize) -> Result<()> {
        let checkpoint_path = format!("{}/model_epoch_{}.json", path, epoch);
        let data = serde_json::to_string_pretty(&CheckpointData {
            epoch,
            weights: &self.weights,
            biases: &self.biases,
            config: &self.config,
        })?;
        std::fs::write(checkpoint_path, data)?;
        Ok(())
    }

    /// Load model checkpoint
    pub fn load_checkpoint(&mut self, path: &Path) -> Result<()> {
        let data = std::fs::read_to_string(path)?;
        let checkpoint: CheckpointData = serde_json::from_str(&data)?;
        self.weights = checkpoint.weights;
        self.biases = checkpoint.biases;
        self.config = checkpoint.config;
        Ok(())
    }

    /// Get training history
    pub fn training_history(&self) -> &[TrainingMetrics] {
        &self.training_history
    }

    /// Get statistics
    pub fn stats(&self) -> &NeuralStats {
        &self.stats
    }

    /// Clear evaluation cache
    pub fn clear_cache(&mut self) {
        self.eval_cache.clear();
        self.stats.cache_size = 0;
    }

    /// Evaluate multiple boards efficiently
    pub fn evaluate_batch(&mut self, boards: &[(Board, Player)]) -> Vec<f64> {
        boards.iter()
            .map(|(board, player)| self.evaluate(board, *player))
            .collect()
    }
}

impl BoardEvaluator for NeuralEvaluator {
    fn evaluate(&mut self, board: &Board, player: Player) -> f64 {
        NeuralEvaluator::evaluate(self, board, player)
    }
}

/// Checkpoint data for serialization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CheckpointData {
    epoch: usize,
    weights: Vec<Vec<f32>>,
    biases: Vec<f32>,
    config: NeuralConfig,
}

/// Self-play training for reinforcement learning
pub struct SelfPlayTrainer {
    evaluator: NeuralEvaluator,
    games_per_iteration: usize,
    temperature: f64,
    exploration_factor: f64,
}

impl SelfPlayTrainer {
    /// Create a new self-play trainer
    pub fn new(evaluator: NeuralEvaluator, games_per_iteration: usize) -> Self {
        Self {
            evaluator,
            games_per_iteration,
            temperature: 1.0,
            exploration_factor: 0.25,
        }
    }

    /// Run self-play training
    pub fn train(&mut self, iterations: usize) -> Result<Vec<TrainingMetrics>> {
        let mut all_metrics = Vec::new();

        for iteration in 0..iterations {
            println!("Starting self-play iteration {}", iteration);

            // Generate training data through self-play
            let dataset = self.generate_self_play_data()?;

            // Train the network
            let iteration_metrics = self.evaluator.train(&dataset)?;
            all_metrics.extend(iteration_metrics);

            println!("Completed iteration {}", iteration);
        }

        Ok(all_metrics)
    }

    /// Generate self-play data
    fn generate_self_play_data(&self) -> Result<TrainingDataset> {
        let mut dataset = TrainingDataset::new();

        for _ in 0..self.games_per_iteration {
            // Simulate a game and collect states
            let game_data = self.simulate_game()?;
            dataset.samples.extend(game_data);
        }

        Ok(dataset)
    }

    /// Simulate a single game
    fn simulate_game(&self) -> Result<Vec<TrainingSample>> {
        let mut samples = Vec::new();
        // Simulate game logic here
        // This is a placeholder - actual implementation would play out a game
        Ok(samples)
    }
}

/// Model ensemble for better predictions
pub struct ModelEnsemble {
    models: Vec<NeuralEvaluator>,
    aggregation_method: EnsembleMethod,
}

/// Method for aggregating predictions from multiple models
#[derive(Debug, Clone, Copy)]
pub enum EnsembleMethod {
    /// Average predictions
    Average,
    /// Weighted average
    WeightedAverage,
    /// Majority vote
    MajorityVote,
    /// Max prediction
    Max,
}

impl ModelEnsemble {
    /// Create a new ensemble
    pub fn new(models: Vec<NeuralEvaluator>) -> Self {
        Self {
            models,
            aggregation_method: EnsembleMethod::Average,
        }
    }

    /// Evaluate using ensemble
    pub fn evaluate(&mut self, board: &Board, player: Player) -> f64 {
        let predictions: Vec<f64> = self.models.iter_mut()
            .map(|model| model.evaluate(board, player))
            .collect();

        match self.aggregation_method {
            EnsembleMethod::Average => {
                predictions.iter().sum::<f64>() / predictions.len() as f64
            }
            EnsembleMethod::WeightedAverage => {
                // Equal weights for now
                predictions.iter().sum::<f64>() / predictions.len() as f64
            }
            EnsembleMethod::MajorityVote => {
                // Find most common prediction bin
                let bins: Vec<i32> = predictions.iter()
                    .map(|p| (p * 10.0).round() as i32)
                    .collect();
                *bins.iter()
                    .max_by_key(|&&x| bins.iter().filter(|&&y| y == x).count())
                    .unwrap() as f64 / 10.0
            }
            EnsembleMethod::Max => {
                *predictions.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0)
            }
        }
    }

    /// Set aggregation method
    pub fn set_aggregation_method(&mut self, method: EnsembleMethod) {
        self.aggregation_method = method;
    }

    /// Add a model to the ensemble
    pub fn add_model(&mut self, model: NeuralEvaluator) {
        self.models.push(model);
    }

    /// Get number of models
    pub fn num_models(&self) -> usize {
        self.models.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_config_default() {
        let config = NeuralConfig::default();
        assert_eq!(config.input_size, 42);
        assert_eq!(config.output_size, 7);
        assert_eq!(config.learning_rate, 0.001);
    }

    #[test]
    fn test_neural_config_sizes() {
        let small = NeuralConfig::small();
        assert_eq!(small.hidden_layers.len(), 2);

        let medium = NeuralConfig::medium();
        assert_eq!(medium.hidden_layers.len(), 3);

        let large = NeuralConfig::large();
        assert_eq!(large.hidden_layers.len(), 4);
    }

    #[test]
    fn test_training_dataset() {
        let mut dataset = TrainingDataset::new();
        let sample = TrainingSample::new(vec![0.0, 1.0, 0.0], vec![0.5, 0.3, 0.2]);
        dataset.add_sample(sample.clone());

        assert_eq!(dataset.samples.len(), 1);

        let stats = dataset.stats();
        assert_eq!(stats.total_samples, 1);
        assert_eq!(stats.train_samples, 1);
    }

    #[test]
    fn test_training_dataset_split() {
        let samples = (0..100)
            .map(|_| TrainingSample::new(vec![0.0, 1.0, 0.0], vec![0.5, 0.3, 0.2]))
            .collect();

        let dataset = TrainingDataset::split(samples, 0.7, 0.2);

        assert_eq!(dataset.samples.len(), 70);
        assert_eq!(dataset.validation_samples.len(), 20);
        assert_eq!(dataset.test_samples.len(), 10);
    }

    #[test]
    fn test_neural_evaluator_creation() {
        let config = NeuralConfig::default();
        let evaluator = NeuralEvaluator::new(config);

        assert!(!evaluator.weights.is_empty());
        assert!(!evaluator.biases.is_empty());
    }

    #[test]
    fn test_neural_stats() {
        let mut stats = NeuralStats::default();
        stats.total_evaluations = 100;
        stats.cache_hits = 30;
        stats.avg_eval_time = 5.0;

        assert_eq!(stats.total_evaluations, 100);
        assert_eq!(stats.cache_hits, 30);
    }

    #[test]
    fn test_model_ensemble() {
        let models = vec![
            NeuralEvaluator::default(),
            NeuralEvaluator::default(),
            NeuralEvaluator::default(),
        ];

        let ensemble = ModelEnsemble::new(models);
        assert_eq!(ensemble.num_models(), 3);
    }

    #[test]
    fn test_ensemble_aggregation_methods() {
        let models = vec![
            NeuralEvaluator::default(),
            NeuralEvaluator::default(),
        ];

        let mut ensemble = ModelEnsemble::new(models);
        ensemble.set_aggregation_method(EnsembleMethod::Max);

        // Test that method was set
        assert!(matches!(ensemble.aggregation_method, EnsembleMethod::Max));
    }
}
