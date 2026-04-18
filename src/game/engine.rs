// game/engine.rs - Core game engine
use crate::core::board::Board;
use crate::core::cell::Cell;
use crate::core::coordinates::Position;
use crate::core::game_config::GameConfig;
use crate::core::game_state::{GamePhase, GameState, GameSnapshot};
use crate::core::move_result::{InvalidMoveReason, MoveResult};
use crate::core::player::{Player, Turn, TurnOrder};
use crate::core::validation::GameValidator;
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct GameEngine {
    state: Arc<Mutex<GameState>>,
    config: GameConfig,
    validator: GameValidator,
}

impl GameEngine {
    pub fn new(config: GameConfig) -> Result<Self, String> {
        config.validate().map_err(|e| e.join(", "))?;
        
        let validator = GameValidator::with_config(config.clone());
        let board = Board::new(config.rows, config.cols);
        let starting_player = config.to_variant().config().to_variant().config().board.is_empty_board() {
            Player::Player1
        } else {
            Player::Player2
        };
        
        let state = GameState::new(board, starting_player);
        
        Ok(GameEngine {
            state: Arc::new(Mutex::new(state)),
            config,
            validator,
        })
    }
    
    pub fn with_turn_order(config: GameConfig, turn_order: TurnOrder) -> Result<Self, String> {
        let mut engine = Self::new(config)?;
        let starting_player = turn_order.first_player();
        
        let mut state = engine.state.lock().unwrap();
        state.turn = Turn::first(starting_player);
        drop(state);
        
        Ok(engine)
    }
    
    pub fn make_move(&self, column: usize, player: Player) -> Result<MoveResult, String> {
        let mut state = self.state.lock().unwrap();
        
        if state.is_game_over() {
            return Ok(MoveResult::game_over(state.winner()));
        }
        
        let validation = self.validator.validate_move(&state, column, player);
        
        if !validation.is_valid() {
            return Ok(MoveResult::invalid(validation.invalid_reason().unwrap_or(InvalidMoveReason::Invalid)));
        }
        
        let board = state.board().clone();
        let row = board.lowest_empty_row(column).ok_or("Column is full")?;
        
        let mut new_board = board.clone();
        new_board.set_at(row, column, player.cell()).map_err(|e| e)?;
        
        let position = Position::new(row, column);
        let is_win = self.check_win(&new_board, player);
        let is_draw = new_board.is_full() && !is_win;
        
        {
            let state_guard = &mut *state;
            state_guard.board = new_board;
            
            if is_win {
                state_guard.end_with_winner(player);
            } else if is_draw {
                state_guard.end_as_draw();
            } else {
                state_guard.advance_turn();
            }
        }
        
        if is_win {
            Ok(MoveResult::winning_move(position))
        } else if is_draw {
            Ok(MoveResult::draw_move(position))
        } else {
            Ok(MoveResult::success(position))
        }
    }
    
    pub fn get_state(&self) -> GameState {
        self.state.lock().unwrap().clone()
    }
    
    pub fn get_snapshot(&self) -> GameSnapshot {
        self.state.lock().unwrap().snapshot()
    }
    
    pub fn restore_snapshot(&self, snapshot: GameSnapshot) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.restore(snapshot);
        Ok(())
    }
    
    pub fn reset(&self) -> Result<(), String> {
        let board = Board::new(self.config.rows, self.config.cols);
        let starting_player = Player::Player1;
        
        let mut state = self.state.lock().unwrap();
        *state = GameState::new(board, starting_player);
        
        Ok(())
    }
    
    pub fn pause(&self) {
        let mut state = self.state.lock().unwrap();
        state.set_phase(GamePhase::Paused);
    }
    
    pub fn resume(&self) {
        let mut state = self.state.lock().unwrap();
        state.set_phase(GamePhase::Playing);
    }
    
    pub fn is_paused(&self) -> bool {
        self.state.lock().unwrap().phase().is_paused()
    }
    
    pub fn current_player(&self) -> Player {
        self.state.lock().unwrap().current_player()
    }
    
    pub fn turn_number(&self) -> u32 {
        self.state.lock().unwrap().turn_number()
    }
    
    pub fn is_game_over(&self) -> bool {
        self.state.lock().unwrap().is_game_over()
    }
    
    pub fn winner(&self) -> Option<Player> {
        self.state.lock().unwrap().winner()
    }
    
    pub fn is_draw(&self) -> bool {
        self.state.lock().unwrap().is_draw()
    }
    
    pub fn valid_moves(&self) -> Vec<usize> {
        let state = self.state.lock().unwrap();
        let board = state.board();
        
        (0..board.cols())
            .filter(|&col| !board.is_column_full(col))
            .collect()
    }
    
    fn check_win(&self, board: &Board, player: Player) -> bool {
        use crate::core::coordinates::Direction;
        const WIN_LENGTH: usize = 4;
        
        for row in 0..board.rows() {
            for col in 0..board.cols() {
                let pos = Position::new(row, col);
                if board.get(&pos) == Some(player.cell()) {
                    for direction in Direction::all() {
                        if board.has_line(&pos, direction, player.cell(), WIN_LENGTH) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let config = GameConfig::standard();
        let engine = GameEngine::new(config).unwrap();
        assert_eq!(engine.current_player(), Player::Player1);
    }

    #[test]
    fn test_engine_make_move() {
        let config = GameConfig::standard();
        let engine = GameEngine::new(config).unwrap();
        let result = engine.make_move(3, Player::Player1).unwrap();
        assert!(result.is_success());
    }

    #[test]
    fn test_engine_valid_moves() {
        let config = GameConfig::standard();
        let engine = GameEngine::new(config).unwrap();
        let moves = engine.valid_moves();
        assert_eq!(moves.len(), 7);
    }
}
