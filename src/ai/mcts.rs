// ai/mcts.rs - Monte Carlo Tree Search AI
use crate::core::board::Board;
use crate::core::cell::Cell;
use crate::core::player::Player;
use rand::Rng;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MCTSNode {
    board: Board,
    player: Player,
    visits: u32,
    wins: u32,
    children: Vec<usize>,
    parent: Option<usize>,
    move_made: Option<usize>,
}

pub struct MCTSAI {
    iterations: u32,
    exploration_constant: f64,
    nodes: Vec<MCTSNode>,
    root: Option<usize>,
}

impl MCTSAI {
    pub fn new(iterations: u32) -> Self {
        MCTSAI {
            iterations,
            exploration_constant: 1.414,
            nodes: Vec::new(),
            root: None,
        }
    }
    
    pub fn with_exploration(iterations: u32, c: f64) -> Self {
        MCTSAI {
            iterations,
            exploration_constant: c,
            nodes: Vec::new(),
            root: None,
        }
    }
    
    pub fn select_move(&mut self, board: &Board, player: Player) -> Option<usize> {
        self.nodes.clear();
        self.root = Some(self.create_node(board.clone(), player, None, None));
        
        for _ in 0..self.iterations {
            if let Some(root_idx) = self.root {
                self.run_iteration(root_idx);
            }
        }
        
        self.get_best_move()
    }
    
    fn create_node(&mut self, board: Board, player: Player, parent: Option<usize>, move_made: Option<usize>) -> usize {
        let node = MCTSNode {
            board,
            player,
            visits: 0,
            wins: 0,
            children: Vec::new(),
            parent,
            move_made,
        };
        
        self.nodes.push(node);
        self.nodes.len() - 1
    }
    
    fn run_iteration(&mut self, node_idx: usize) {
        let mut current = node_idx;
        let path = vec![current];
        
        while !self.nodes[current].children.is_empty() {
            current = self.select_child(current);
            path.push(current);
        }
        
        if self.nodes[current].visits > 0 {
            self.expand_node(current);
            if let Some(&child) = self.nodes[current].children.last() {
                path.push(child);
            }
        }
        
        let leaf = *path.last().unwrap();
        let winner = self.simulate(leaf);
        self.backpropagate(path, winner);
    }
    
    fn select_child(&self, node_idx: usize) -> usize {
        let node = &self.nodes[node_idx];
        let log_visits = (node.visits as f64).ln();
        
        node.children
            .iter()
            .max_by(|a, b| {
                let ucb_a = self.ucb(*a, log_visits);
                let ucb_b = self.ucb(*b, log_visits);
                ucb_a.partial_cmp(&ucb_b).unwrap()
            })
            .copied()
            .unwrap()
    }
    
    fn ucb(&self, child_idx: usize, log_parent_visits: f64) -> f64 {
        let child = &self.nodes[child_idx];
        
        if child.visits == 0 {
            return f64::INFINITY;
        }
        
        let win_rate = child.wins as f64 / child.visits as f64;
        let exploration = self.exploration_constant * (log_parent_visits / child.visits as f64).sqrt();
        
        win_rate + exploration
    }
    
    fn expand_node(&mut self, node_idx: usize) {
        let node = &self.nodes[node_idx];
        let board = &node.board;
        let player = node.player;
        
        for col in 0..board.cols() {
            if !board.is_column_full(col) {
                let mut new_board = board.clone();
                if let Some(row) = board.lowest_empty_row(col) {
                    new_board.set_at(row, col, player.cell()).unwrap();
                    let child_idx = self.create_node(new_board, player.opponent(), Some(node_idx), Some(col));
                    self.nodes[node_idx].children.push(child_idx);
                }
            }
        }
    }
    
    fn simulate(&self, node_idx: usize) -> Option<Player> {
        let mut board = self.nodes[node_idx].board.clone();
        let mut player = self.nodes[node_idx].player;
        let mut rng = rand::thread_rng();
        
        loop {
            if self.check_win(&board, Player::Player1) {
                return Some(Player::Player1);
            }
            if self.check_win(&board, Player::Player2) {
                return Some(Player::Player2);
            }
            if board.is_full() {
                return None;
            }
            
            let valid_moves: Vec<usize> = (0..board.cols())
                .filter(|&col| !board.is_column_full(col))
                .collect();
            
            if let Some(&col) = valid_moves.choose(&mut rng) {
                if let Some(row) = board.lowest_empty_row(col) {
                    board.set_at(row, col, player.cell()).unwrap();
                }
                player = player.opponent();
            } else {
                break;
            }
        }
        
        None
    }
    
    fn backpropagate(&mut self, mut path: Vec<usize>, winner: Option<Player>) {
        while let Some(node_idx) = path.pop() {
            self.nodes[node_idx].visits += 1;
            
            if let Some(winner) = winner {
                let node_player = self.nodes[node_idx].player;
                if winner == node_player.opponent() {
                    self.nodes[node_idx].wins += 1;
                }
            }
        }
    }
    
    fn get_best_move(&self) -> Option<usize> {
        if let Some(root_idx) = self.root {
            let root = &self.nodes[root_idx];
            
            root.children
                .iter()
                .max_by_key(|&&child_idx| {
                    let child = &self.nodes[child_idx];
                    child.visits
                })
                .map(|&child_idx| self.nodes[child_idx].move_made)
                .flatten()
        } else {
            None
        }
    }
    
    fn check_win(&self, board: &Board, player: Player) -> bool {
        use crate::core::coordinates::Direction;
        const WIN_LENGTH: usize = 4;
        
        for row in 0..board.rows() {
            for col in 0..board.cols() {
                let pos = crate::core::coordinates::Position::new(row, col);
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
    fn test_mcts_new() {
        let ai = MCTSAI::new(100);
        assert_eq!(ai.iterations, 100);
    }

    #[test]
    fn test_mcts_select_move() {
        let mut ai = MCTSAI::new(50);
        let board = Board::new(6, 7);
        let player = Player::Player1;
        
        let column = ai.select_move(&board, player);
        assert!(column.is_some());
    }
}
