// ai/alphabeta.rs - Alpha-Beta pruning AI algorithm
use crate::core::board::Board;
use crate::core::cell::Cell;
use crate::core::coordinates::Position;
use crate::core::player::Player;
use crate::ai::evaluation::BoardEvaluator;
use std::time::{Duration, Instant};

pub struct AlphaBetaAI {
    depth: usize,
    max_depth: usize,
    nodes_evaluated: u64,
    cutoffs: u64,
    evaluator: Box<dyn BoardEvaluator>,
    use_iterative_deepening: bool,
}

impl AlphaBetaAI {
    pub fn new(depth: usize) -> Self {
        AlphaBetaAI {
            depth,
            max_depth: depth,
            nodes_evaluated: 0,
            cutoffs: 0,
            evaluator: Box::new(crate::ai::evaluation::SimpleEvaluator),
            use_iterative_deepening: true,
        }
    }

    pub fn with_evaluator(depth: usize, evaluator: Box<dyn BoardEvaluator>) -> Self {
        AlphaBetaAI {
            depth,
            max_depth: depth,
            nodes_evaluated: 0,
            cutoffs: 0,
            evaluator,
            use_iterative_deepening: true,
        }
    }

    pub const fn nodes_evaluated(&self) -> u64 {
        self.nodes_evaluated
    }

    pub const fn cutoffs(&self) -> u64 {
        self.cutoffs
    }

    pub fn reset_stats(&mut self) {
        self.nodes_evaluated = 0;
        self.cutoffs = 0;
    }

    pub fn select_move(&mut self, board: &Board, player: Player) -> Option<usize> {
        self.reset_stats();
        let mut best_score = i32::MIN;
        let mut best_column = None;

        for col in 0..board.cols() {
            if !board.is_column_full(col) {
                let mut test_board = board.clone();
                if let Some(row) = board.lowest_empty_row(col) {
                    test_board.set_at(row, col, player.cell()).unwrap();
                    let score = self.minimax_alpha_beta(
                        &test_board, self.depth - 1, i32::MIN + 1, i32::MAX - 1, false, player
                    );
                    if score > best_score {
                        best_score = score;
                        best_column = Some(col);
                    }
                }
            }
        }
        best_column
    }

    fn minimax_alpha_beta(
        &mut self,
        board: &Board,
        depth: usize,
        mut alpha: i32,
        beta: i32,
        is_maximizing: bool,
        player: Player,
    ) -> i32 {
        if depth == 0 || self.is_terminal(board, player) {
            self.nodes_evaluated += 1;
            return self.evaluator.evaluate(board, if is_maximizing { player } else { player.opponent() });
        }

        if is_maximizing {
            let mut max_eval = i32::MIN;
            for col in 0..board.cols() {
                if !board.is_column_full(col) {
                    let mut test_board = board.clone();
                    if let Some(row) = board.lowest_empty_row(col) {
                        test_board.set_at(row, col, player.cell()).unwrap();
                        let eval = self.minimax_alpha_beta(&test_board, depth - 1, alpha, beta, false, player);
                        max_eval = max_eval.max(eval);
                        alpha = alpha.max(eval);
                        if beta <= alpha {
                            self.cutoffs += 1;
                            break;
                        }
                    }
                }
            }
            max_eval
        } else {
            let mut min_eval = i32::MAX;
            for col in 0..board.cols() {
                if !board.is_column_full(col) {
                    let mut test_board = board.clone();
                    if let Some(row) = board.lowest_empty_row(col) {
                        test_board.set_at(row, col, player.opponent().cell()).unwrap();
                        let eval = self.minimax_alpha_beta(&test_board, depth - 1, alpha, beta, true, player);
                        min_eval = min_eval.min(eval);
                        beta = beta.min(eval);
                        if beta <= alpha {
                            self.cutoffs += 1;
                            break;
                        }
                    }
                }
            }
            min_eval
        }
    }

    fn is_terminal(&self, board: &Board, player: Player) -> bool {
        self.check_win(board, Player::Player1) || self.check_win(board, Player::Player2) || board.is_full()
    }

    fn check_win(&self, board: &Board, player: Player) -> bool {
        use crate::core::coordinates::Direction;
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
}
