// ai/evaluation.rs - Board evaluation functions for AI
use crate::core::board::Board;
use crate::core::cell::Cell;
use crate::core::coordinates::{Direction, Position};
use crate::core::player::Player;

pub trait BoardEvaluator: Send + Sync {
    fn evaluate(&self, board: &Board, player: Player) -> i32;
}

pub struct SimpleEvaluator;

impl BoardEvaluator for SimpleEvaluator {
    fn evaluate(&self, board: &Board, player: Player) -> i32 {
        if self.check_win(board, player) {
            return 10000;
        }
        if self.check_win(board, player.opponent()) {
            return -10000;
        }
        
        let mut score = 0;
        
        for row in 0..board.rows() {
            for col in 0..board.cols() {
                if let Some(cell) = board.get_at(row, col) {
                    if cell == player.cell() {
                        score += self.evaluate_position(board, row, col, player);
                    } else if cell == player.opponent().cell() {
                        score -= self.evaluate_position(board, row, col, player.opponent());
                    }
                }
            }
        }
        
        score
    }
}

impl SimpleEvaluator {
    fn check_win(&self, board: &Board, player: Player) -> bool {
        const WIN_LENGTH: usize = 4;
        for row in 0..board.rows() {
            for col in 0..board.cols() {
                let pos = Position::new(row, col);
                if board.get(&pos) == Some(player.cell()) {
                    for direction in Direction::all().iter() {
                        if board.has_line(&pos, *direction, player.cell(), WIN_LENGTH) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    
    fn evaluate_position(&self, board: &Board, row: usize, col: usize, player: Player) -> i32 {
        let pos = Position::new(row, col);
        let mut score = 0;
        
        for direction in Direction::all() {
            let count = board.count_consecutive_bidirectional(&pos, direction, player.cell());
            score += match count {
                1 => 1,
                2 => 10,
                3 => 100,
                _ => 0,
            };
        }
        
        score
    }
}

pub struct AdvancedEvaluator {
    center_weight: i32,
    chain_weight: i32,
    blocking_weight: i32,
}

impl AdvancedEvaluator {
    pub fn new() -> Self {
        AdvancedEvaluator {
            center_weight: 10,
            chain_weight: 20,
            blocking_weight: 15,
        }
    }
}

impl Default for AdvancedEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl BoardEvaluator for AdvancedEvaluator {
    fn evaluate(&self, board: &Board, player: Player) -> i32 {
        if self.check_win(board, player) {
            return 100000;
        }
        if self.check_win(board, player.opponent()) {
            return -100000;
        }
        
        let mut score = 0;
        
        score += self.evaluate_center(board, player);
        score += self.evaluate_chains(board, player);
        score += self.evaluate_blocking(board, player);
        score += self.evaluate_threats(board, player);
        
        score
    }
}

impl AdvancedEvaluator {
    fn check_win(&self, board: &Board, player: Player) -> bool {
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
    
    fn evaluate_center(&self, board: &Board, player: Player) -> i32 {
        let center_col = board.cols() / 2;
        let mut score = 0;
        
        for row in 0..board.rows() {
            let distance = (center_col as i32 - row as i32).abs() as i32;
            let weight = self.center_weight * (board.cols() as i32 - distance);
            
            if board.get_at(row, center_col) == Some(player.cell()) {
                score += weight;
            } else if board.get_at(row, center_col) == Some(player.opponent().cell()) {
                score -= weight;
            }
        }
        
        score
    }
    
    fn evaluate_chains(&self, board: &Board, player: Player) -> i32 {
        let mut score = 0;
        
        for row in 0..board.rows() {
            for col in 0..board.cols() {
                let pos = Position::new(row, col);
                if board.get(&pos) == Some(player.cell()) {
                    for direction in Direction::all() {
                        let count = board.count_consecutive_bidirectional(&pos, direction, player.cell());
                        score += match count {
                            2 => self.chain_weight * 2,
                            3 => self.chain_weight * 10,
                            _ => 0,
                        };
                    }
                }
            }
        }
        
        score
    }
    
    fn evaluate_blocking(&self, board: &Board, player: Player) -> i32 {
        let mut score = 0;
        let opponent = player.opponent();
        
        for row in 0..board.rows() {
            for col in 0..board.cols() {
                let pos = Position::new(row, col);
                if board.get(&pos) == Some(opponent.cell()) {
                    for direction in Direction::all() {
                        let count = board.count_consecutive_bidirectional(&pos, direction, opponent.cell());
                        if count >= 2 {
                            score += self.blocking_weight * count as i32;
                        }
                    }
                }
            }
        }
        
        score
    }
    
    fn evaluate_threats(&self, board: &Board, player: Player) -> i32 {
        let mut score = 0;
        
        for col in 0..board.cols() {
            if let Some(row) = board.lowest_empty_row(col) {
                let mut test_board = board.clone();
                test_board.set_at(row, col, player.cell()).unwrap();
                
                if self.check_win(&test_board, player) {
                    score += 1000;
                }
            }
        }
        
        score
    }
}
