// ai/strategy.rs - AI strategy selection and configuration
use crate::ai::minimax::MinimaxAI;
use crate::ai::alphabeta::AlphaBetaAI;
use crate::core::player::Player;

pub enum AIStrategy {
    Random,
    Minimax { depth: usize },
    AlphaBeta { depth: usize },
    MCTS { iterations: u32 },
    Neural { model_id: String },
    Hybrid { strategies: Vec<AIStrategy> },
}

impl AIStrategy {
    pub fn name(&self) -> &str {
        match self {
            AIStrategy::Random => "Random",
            AIStrategy::Minimax { .. } => "Minimax",
            AIStrategy::AlphaBeta { .. } => "Alpha-Beta Pruning",
            AIStrategy::MCTS { .. } => "Monte Carlo Tree Search",
            AIStrategy::Neural { .. } => "Neural Network",
            AIStrategy::Hybrid { .. } => "Hybrid",
        }
    }
}

pub struct AIPlayer {
    name: String,
    strategy: AIStrategy,
}

impl AIPlayer {
    pub fn new(name: String, strategy: AIStrategy) -> Self {
        AIPlayer { name, strategy }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn strategy(&self) -> &AIStrategy {
        &self.strategy
    }
}

pub struct StrategySelector {
    available_strategies: Vec<AIStrategy>,
}

impl StrategySelector {
    pub fn new() -> Self {
        let strategies = vec![
            AIStrategy::Minimax { depth: 3 },
            AIStrategy::Minimax { depth: 6 },
            AIStrategy::Minimax { depth: 9 },
            AIStrategy::AlphaBeta { depth: 4 },
            AIStrategy::AlphaBeta { depth: 7 },
            AIStrategy::AlphaBeta { depth: 10 },
            AIStrategy::MCTS { iterations: 1000 },
            AIStrategy::MCTS { iterations: 5000 },
            AIStrategy::MCTS { iterations: 10000 },
        ];
        
        StrategySelector {
            available_strategies: strategies,
        }
    }
    
    pub fn select_by_difficulty(&self, difficulty: DifficultyLevel) -> Option<&AIStrategy> {
        let index = match difficulty {
            DifficultyLevel::Easy => 0,
            DifficultyLevel::Medium => 3,
            DifficultyLevel::Hard => 6,
            DifficultyLevel::Expert => 8,
            DifficultyLevel::Master => 9,
        };
        
        self.available_strategies.get(index)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DifficultyLevel {
    Easy,
    Medium,
    Hard,
    Expert,
    Master,
}

impl DifficultyLevel {
    pub fn as_str(&self) -> &str {
        match self {
            DifficultyLevel::Easy => "Easy",
            DifficultyLevel::Medium => "Medium",
            DifficultyLevel::Hard => "Hard",
            DifficultyLevel::Expert => "Expert",
            DifficultyLevel::Master => "Master",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_name() {
        let strategy = AIStrategy::Minimax { depth: 5 };
        assert_eq!(strategy.name(), "Minimax");
    }

    #[test]
    fn test_ai_player() {
        let strategy = AIStrategy::Random;
        let player = AIPlayer::new("Bot 1".to_string(), strategy);
        assert_eq!(player.name(), "Bot 1");
    }

    #[test]
    fn test_difficulty_level() {
        assert_eq!(DifficultyLevel::Easy.as_str(), "Easy");
        assert_eq!(DifficultyLevel::Master.as_str(), "Master");
    }
}
