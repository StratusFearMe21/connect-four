// game/controller.rs - Game controller for managing game flow
use crate::core::player::Player;
use crate::core::game_config::GameConfig;
use crate::core::game_state::GameState;
use crate::game::engine::GameEngine;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct GameController {
    engine: Arc<GameEngine>,
    state: Arc<RwLock<ControllerState>>,
}

#[derive(Clone)]
pub struct ControllerState {
    pub paused: bool,
    pub speed_multiplier: f32,
    pub auto_play: bool,
    pub record_replay: bool,
}

impl Default for ControllerState {
    fn default() -> Self {
        ControllerState {
            paused: false,
            speed_multiplier: 1.0,
            auto_play: false,
            record_replay: true,
        }
    }
}

impl GameController {
    pub fn new(engine: Arc<GameEngine>) -> Self {
        let state = Arc::new(RwLock::new(ControllerState::default()));
        
        GameController {
            engine,
            state,
        }
    }
    
    pub fn with_config(config: GameConfig) -> Result<Self, String> {
        let engine = Arc::new(GameEngine::new(config)?);
        Ok(Self::new(engine))
    }
    
    pub async fn start_game(&self) -> Result<(), String> {
        let mut state = self.state.write().await;
        state.paused = false;
        Ok(())
    }
    
    pub async fn pause_game(&self) {
        self.engine.pause();
        let mut state = self.state.write().await;
        state.paused = true;
    }
    
    pub async fn resume_game(&self) {
        self.engine.resume();
        let mut state = self.state.write().await;
        state.paused = false;
    }
    
    pub async fn reset_game(&self) -> Result<(), String> {
        self.engine.reset()
    }
    
    pub async fn make_move(&self, column: usize, player: Player) -> Result<crate::core::move_result::MoveResult, String> {
        let state = self.state.read().await;
        if state.paused {
            return Err("Game is paused".to_string());
        }
        drop(state);
        
        self.engine.make_move(column, player)
    }
    
    pub async fn get_game_state(&self) -> GameState {
        self.engine.get_state()
    }
    
    pub async fn set_speed_multiplier(&self, multiplier: f32) {
        let mut state = self.state.write().await;
        state.speed_multiplier = multiplier.clamp(0.1, 10.0);
    }
    
    pub async fn get_speed_multiplier(&self) -> f32 {
        let state = self.state.read().await;
        state.speed_multiplier
    }
    
    pub async fn set_auto_play(&self, enabled: bool) {
        let mut state = self.state.write().await;
        state.auto_play = enabled;
    }
    
    pub async fn is_auto_play(&self) -> bool {
        let state = self.state.read().await;
        state.auto_play
    }
    
    pub async fn is_paused(&self) -> bool {
        let state = self.state.read().await;
        state.paused
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_controller_new() {
        let config = GameConfig::standard();
        let controller = GameController::with_config(config).unwrap();
        assert!(!controller.is_paused().await);
    }
}
