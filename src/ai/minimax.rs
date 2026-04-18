// ai/minimax.rs - Minimax AI algorithm implementation
use crate::core::board::Board;
use crate::core::cell::Cell;
use crate::core::coordinates::Position;
use crate::core::player::Player;
use crate::ai::evaluation::BoardEvaluator;
use std::time::{Duration, Instant};

/// Minimax AI player with configurable depth
pub struct MinimaxAI {
    depth: usize,
    max_depth: usize,
    nodes_evaluated: u64,
    cutoffs: u64,
    evaluator: Box<dyn BoardEvaluator>,
}

impl MinimaxAI {
    /// Creates a new minimax AI with the given depth
    pub fn new(depth: usize) -> Self {
        MinimaxAI {
            depth,
            max_depth: depth,
            nodes_evaluated: 0,
            cutoffs: 0,
            evaluator: Box::new(crate::ai::evaluation::SimpleEvaluator),
        }
    }

    /// Creates a minimax AI with a custom evaluator
    pub fn with_evaluator(depth: usize, evaluator: Box<dyn BoardEvaluator>) -> Self {
        MinimaxAI {
            depth,
            max_depth: depth,
            nodes_evaluated: 0,
            cutoffs: 0,
            evaluator,
        }
    }

    /// Sets the search depth
    pub fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
        self.max_depth = depth;
    }

    /// Returns the current search depth
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the number of nodes evaluated
    pub const fn nodes_evaluated(&self) -> u64 {
        self.nodes_evaluated
    }

    /// Returns the number of cutoffs
    pub const fn cutoffs(&self) -> u64 {
        self.cutoffs
    }

    /// Resets statistics
    pub fn reset_stats(&mut self) {
        self.nodes_evaluated = 0;
        self.cutoffs = 0;
    }

    /// Selects the best move using minimax
    pub fn select_move(&mut self, board: &Board, player: Player) -> Option<usize> {
        self.reset_stats();

        let columns: Vec<usize> = (0..board.cols())
            .filter(|&col| !board.is_column_full(col))
            .collect();

        if columns.is_empty() {
            return None;
        }

        let mut best_column = columns[0];
        let mut best_score = i32::MIN;

        for &col in &columns {
            let mut test_board = board.clone();
            if let Some(row) = board.lowest_empty_row(col) {
                test_board.set_at(row, col, player.cell()).unwrap();

                let score = self.minimize(&test_board, self.depth - 1, player.opponent());

                if score > best_score {
                    best_score = score;
                    best_column = col;
                }
            }
        }

        Some(best_column)
    }

    /// Selects the best move with a time limit
    pub fn select_move_with_time_limit(
        &mut self,
        board: &Board,
        player: Player,
        time_limit: Duration,
    ) -> Option<usize> {
        let start_time = Instant::now();
        let mut result = None;

        // Iterative deepening
        for depth in 1..=self.max_depth {
            self.depth = depth;
            let move_col = self.select_move(board, player);
            result = move_col;

            if start_time.elapsed() >= time_limit {
                break;
            }
        }

        result
    }

    /// Maximizing player (current player)
    fn maximize(&mut self, board: &Board, depth: usize, player: Player) -> i32 {
        if depth == 0 || self.is_terminal(board, player) {
            self.nodes_evaluated += 1;
            return self.evaluator.evaluate(board, player);
        }

        let mut max_score = i32::MIN;

        for col in 0..board.cols() {
            if !board.is_column_full(col) {
                let mut test_board = board.clone();
                if let Some(row) = board.lowest_empty_row(col) {
                    test_board.set_at(row, col, player.cell()).unwrap();

                    let score = self.minimize(&test_board, depth - 1, player.opponent());
                    max_score = max_score.max(score);
                }
            }
        }

        max_score
    }

    /// Minimizing player (opponent)
    fn minimize(&mut self, board: &Board, depth: usize, player: Player) -> i32 {
        if depth == 0 || self.is_terminal(board, player) {
            self.nodes_evaluated += 1;
            return self.evaluator.evaluate(board, player.opponent());
        }

        let mut min_score = i32::MAX;

        for col in 0..board.cols() {
            if !board.is_column_full(col) {
                let mut test_board = board.clone();
                if let Some(row) = board.lowest_empty_row(col) {
                    test_board.set_at(row, col, player.cell()).unwrap();

                    let score = self.maximize(&test_board, depth - 1, player.opponent());
                    min_score = min_score.min(score);
                }
            }
        }

        min_score
    }

    /// Checks if the current position is terminal (game over or win)
    fn is_terminal(&self, board: &Board, player: Player) -> bool {
        self.check_win(board, Player::Player1)
            || self.check_win(board, Player::Player2)
            || board.is_full()
    }

    /// Checks if a player has won
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

    /// Evaluates all moves and returns their scores
    pub fn evaluate_moves(
        &mut self,
        board: &Board,
        player: Player,
    ) -> Vec<(usize, i32)> {
        let mut move_scores = Vec::new();

        for col in 0..board.cols() {
            if !board.is_column_full(col) {
                let mut test_board = board.clone();
                if let Some(row) = board.lowest_empty_row(col) {
                    test_board.set_at(row, col, player.cell()).unwrap();

                    let score = self.minimize(&test_board, self.depth - 1, player.opponent());
                    move_scores.push((col, score));
                }
            }
        }

        move_scores
    }

    /// Returns the principal variation (best line of play)
    pub fn principal_variation(
        &mut self,
        board: &Board,
        player: Player,
    ) -> Vec<usize> {
        let mut pv = Vec::new();
        let mut current_board = board.clone();
        let mut current_player = player;

        for _ in 0..self.depth {
            if let Some(col) = self.select_move(&current_board, current_player) {
                pv.push(col);

                if let Some(row) = current_board.lowest_empty_row(col) {
                    current_board.set_at(row, col, current_player.cell()).unwrap();
                }

                current_player = current_player.opponent();
            } else {
                break;
            }
        }

        pv
    }
}

impl Default for MinimaxAI {
    fn default() -> Self {
        Self::new(6)
    }
}

/// NegaMax variant of minimax
pub struct NegaMaxAI {
    depth: usize,
    nodes_evaluated: u64,
    evaluator: Box<dyn BoardEvaluator>,
}

impl NegaMaxAI {
    pub fn new(depth: usize) -> Self {
        NegaMaxAI {
            depth,
            nodes_evaluated: 0,
            evaluator: Box::new(crate::ai::evaluation::SimpleEvaluator),
        }
    }

    pub fn select_move(&mut self, board: &Board, player: Player) -> Option<usize> {
        self.nodes_evaluated = 0;

        let columns: Vec<usize> = (0..board.cols())
            .filter(|&col| !board.is_column_full(col))
            .collect();

        if columns.is_empty() {
            return None;
        }

        let mut best_column = columns[0];
        let mut best_score = i32::MIN;

        for &col in &columns {
            let mut test_board = board.clone();
            if let Some(row) = board.lowest_empty_row(col) {
                test_board.set_at(row, col, player.cell()).unwrap();

                let score = -self.negamax(&test_board, self.depth - 1, -i32::MAX, i32::MAX, player);

                if score > best_score {
                    best_score = score;
                    best_column = col;
                }
            }
        }

        Some(best_column)
    }

    fn negamax(
        &mut self,
        board: &Board,
        depth: usize,
        mut alpha: i32,
        _beta: i32,
        player: Player,
    ) -> i32 {
        if depth == 0 || self.is_terminal(board, player) {
            self.nodes_evaluated += 1;
            let perspective = if player == Player::Player1 { 1 } else { -1 };
            return self.evaluator.evaluate(board, player) * perspective;
        }

        let mut max_score = i32::MIN;

        for col in 0..board.cols() {
            if !board.is_column_full(col) {
                let mut test_board = board.clone();
                if let Some(row) = board.lowest_empty_row(col) {
                    test_board.set_at(row, col, player.cell()).unwrap();

                    let score = -self.negamax(
                        &test_board,
                        depth - 1,
                        -(_beta),
                        -alpha,
                        player.opponent(),
                    );

                    max_score = max_score.max(score);
                    alpha = alpha.max(score);

                    if alpha >= _beta {
                        break;
                    }
                }
            }
        }

        max_score
    }

    fn is_terminal(&self, board: &Board, player: Player) -> bool {
        self.check_win(board, Player::Player1)
            || self.check_win(board, Player::Player2)
            || board.is_full()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimax_new() {
        let ai = MinimaxAI::new(4);
        assert_eq!(ai.depth(), 4);
    }

    #[test]
    fn test_minimax_select_move() {
        let mut ai = MinimaxAI::new(3);
        let board = Board::standard();
        let player = Player::Player1;

        let column = ai.select_move(&board, player);
        assert!(column.is_some());
        assert!(column.unwrap() < 7);
    }

    #[test]
    fn test_minimax_reset_stats() {
        let mut ai = MinimaxAI::new(3);
        let board = Board::standard();

        ai.select_move(&board, Player::Player1);
        assert!(ai.nodes_evaluated() > 0);

        ai.reset_stats();
        assert_eq!(ai.nodes_evaluated(), 0);
    }

    #[test]
    fn test_negamax_new() {
        let ai = NegaMaxAI::new(4);
        assert_eq!(ai.depth, 4);
    }

    #[test]
    fn test_negamax_select_move() {
        let mut ai = NegaMaxAI::new(3);
        let board = Board::standard();
        let player = Player::Player1;

        let column = ai.select_move(&board, player);
        assert!(column.is_some());
    }
}
