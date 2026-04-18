// core/game_state.rs - Game state management
use crate::core::board::Board;
use crate::core::cell::Cell;
use crate::core::coordinates::Position;
use crate::core::player::{Player, Turn};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use uuid::Uuid;

/// The current state of a game
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct GameState {
    /// Unique game identifier
    pub game_id: Uuid,
    /// The game board
    pub board: Board,
    /// Current turn information
    pub turn: Turn,
    /// Whether the game is over
    pub game_over: bool,
    /// The winner, if any
    pub winner: Option<Player>,
    /// Whether the game ended in a draw
    pub is_draw: bool,
    /// When the game started
    pub start_time: DateTime<Utc>,
    /// When the game ended (if over)
    pub end_time: Option<DateTime<Utc>>,
    /// Move history
    pub moves: Vec<GameMove>,
    /// Current game phase
    pub phase: GamePhase,
}

impl GameState {
    /// Creates a new game state
    pub fn new(board: Board, starting_player: Player) -> Self {
        let now = Utc::now();
        GameState {
            game_id: Uuid::new_v4(),
            board,
            turn: Turn::first(starting_player),
            game_over: false,
            winner: None,
            is_draw: false,
            start_time: now,
            end_time: None,
            moves: Vec::new(),
            phase: GamePhase::Playing,
        }
    }

    /// Creates a standard game state
    pub fn standard() -> Self {
        Self::new(Board::standard(), Player::Player1)
    }

    /// Returns the game ID
    pub const fn game_id(&self) -> Uuid {
        self.game_id
    }

    /// Returns the board
    pub const fn board(&self) -> &Board {
        &self.board
    }

    /// Returns the current turn
    pub const fn turn(&self) -> &Turn {
        &self.turn
    }

    /// Returns the current player
    pub const fn current_player(&self) -> Player {
        self.turn.current_player
    }

    /// Returns the turn number
    pub const fn turn_number(&self) -> u32 {
        self.turn.turn_number
    }

    /// Returns true if the game is over
    pub const fn is_game_over(&self) -> bool {
        self.game_over
    }

    /// Returns the winner if the game is over and someone won
    pub const fn winner(&self) -> Option<Player> {
        self.winner
    }

    /// Returns true if the game ended in a draw
    pub const fn is_draw(&self) -> bool {
        self.is_draw
    }

    /// Returns the move history
    pub const fn moves(&self) -> &[GameMove] {
        &self.moves
    }

    /// Returns the number of moves made
    pub fn move_count(&self) -> usize {
        self.moves.len()
    }

    /// Returns the last move
    pub fn last_move(&self) -> Option<&GameMove> {
        self.moves.last()
    }

    /// Returns the game phase
    pub const fn phase(&self) -> GamePhase {
        self.phase
    }

    /// Returns the duration of the game so far
    pub fn duration(&self) -> chrono::Duration {
        let end = self.end_time.unwrap_or_else(Utc::now);
        end.signed_duration_since(self.start_time)
    }

    /// Checks if a player has won
    pub fn check_win(&self, player: Player) -> bool {
        self.check_win_for_player(player)
    }

    /// Internal win checking for a specific player
    fn check_win_for_player(&self, player: Player) -> bool {
        const WIN_LENGTH: usize = 4;

        for row in 0..self.board.rows() {
            for col in 0..self.board.cols() {
                let pos = Position::new(row, col);
                let cell = self.board.get(&pos);

                if cell == Some(player.cell()) {
                    // Check all four directions
                    if self.check_directional_win(&pos, player, WIN_LENGTH) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Checks for a win in all four directions from a position
    fn check_directional_win(&self, pos: &Position, player: Player, length: usize) -> bool {
        use crate::core::coordinates::Direction;

        let directions = [
            Direction::East,
            Direction::South,
            Direction::Southeast,
            Direction::Southwest,
        ];

        directions
            .iter()
            .any(|&dir| self.board.has_line(pos, dir, player.cell(), length))
    }

    /// Checks if the board is full (draw condition)
    pub fn is_board_full(&self) -> bool {
        self.board.is_full()
    }

    /// Records a move in the game state
    pub fn record_move(&mut self, game_move: GameMove) {
        self.moves.push(game_move);
    }

    /// Advances to the next turn
    pub fn advance_turn(&mut self) {
        self.turn = self.turn.next();
    }

    /// Ends the game with a winner
    pub fn end_with_winner(&mut self, winner: Player) {
        self.game_over = true;
        self.winner = Some(winner);
        self.is_draw = false;
        self.end_time = Some(Utc::now());
        self.phase = GamePhase::Ended;
    }

    /// Ends the game in a draw
    pub fn end_as_draw(&mut self) {
        self.game_over = true;
        self.winner = None;
        self.is_draw = true;
        self.end_time = Some(Utc::now());
        self.phase = GamePhase::Ended;
    }

    /// Sets the game phase
    pub fn set_phase(&mut self, phase: GamePhase) {
        self.phase = phase;
    }

    /// Creates a snapshot of the current state
    pub fn snapshot(&self) -> GameSnapshot {
        GameSnapshot {
            game_id: self.game_id,
            board: self.board.clone(),
            turn: self.turn,
            game_over: self.game_over,
            winner: self.winner,
            is_draw: self.is_draw,
            move_count: self.moves.len(),
            phase: self.phase,
        }
    }

    /// Restores from a snapshot
    pub fn restore(&mut self, snapshot: GameSnapshot) {
        self.game_id = snapshot.game_id;
        self.board = snapshot.board;
        self.turn = snapshot.turn;
        self.game_over = snapshot.game_over;
        self.winner = snapshot.winner;
        self.is_draw = snapshot.is_draw;
        self.moves.truncate(snapshot.move_count);
        self.phase = snapshot.phase;
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Game ID: {}", self.game_id)?;
        writeln!(f, "Turn: {}", self.turn)?;
        writeln!(f, "Phase: {:?}", self.phase)?;
        writeln!(f, "Moves: {}", self.move_count())?;
        write!(f, "{}", self.board)
    }
}

/// A move made in the game
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct GameMove {
    /// Move number (starting from 1)
    pub move_number: u32,
    /// The player who made the move
    pub player: Player,
    /// The column where the piece was placed
    pub column: usize,
    /// The row where the piece landed
    pub row: usize,
    /// The position where the piece landed
    pub position: Position,
    /// When the move was made
    pub timestamp: DateTime<Utc>,
    /// Time taken to make this move (in milliseconds)
    pub time_taken_ms: Option<u64>,
    /// Whether this was a winning move
    pub is_winning_move: bool,
}

impl GameMove {
    /// Creates a new game move
    pub fn new(
        move_number: u32,
        player: Player,
        column: usize,
        row: usize,
    ) -> Self {
        GameMove {
            move_number,
            player,
            column,
            row,
            position: Position::new(row, column),
            timestamp: Utc::now(),
            time_taken_ms: None,
            is_winning_move: false,
        }
    }

    /// Returns the move number
    pub const fn number(&self) -> u32 {
        self.move_number
    }

    /// Returns the player who made the move
    pub const fn player(&self) -> Player {
        self.player
    }

    /// Returns the column
    pub const fn column(&self) -> usize {
        self.column
    }

    /// Returns the row
    pub const fn row(&self) -> usize {
        self.row
    }

    /// Returns the position
    pub const fn position(&self) -> Position {
        self.position
    }
}

/// Game phases
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum GamePhase {
    /// Game is waiting to start
    Waiting,
    /// Game is in progress
    Playing,
    /// Game is paused
    Paused,
    /// Game has ended
    Ended,
}

impl GamePhase {
    /// Returns true if the game can accept moves
    pub const fn can_play(&self) -> bool {
        matches!(self, GamePhase::Playing)
    }

    /// Returns true if the game is paused
    pub const fn is_paused(&self) -> bool {
        matches!(self, GamePhase::Paused)
    }

    /// Returns true if the game is over
    pub const fn is_over(&self) -> bool {
        matches!(self, GamePhase::Ended)
    }
}

/// A compact snapshot of game state
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub game_id: Uuid,
    pub board: Board,
    pub turn: Turn,
    pub game_over: bool,
    pub winner: Option<Player>,
    pub is_draw: bool,
    pub move_count: usize,
    pub phase: GamePhase,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_new() {
        let state = GameState::standard();
        assert!(!state.is_game_over());
        assert_eq!(state.current_player(), Player::Player1);
        assert_eq!(state.turn_number(), 1);
    }

    #[test]
    fn test_game_state_advance_turn() {
        let mut state = GameState::standard();
        state.advance_turn();
        assert_eq!(state.current_player(), Player::Player2);
        assert_eq!(state.turn_number(), 2);
    }

    #[test]
    fn test_game_state_end_with_winner() {
        let mut state = GameState::standard();
        state.end_with_winner(Player::Player1);
        assert!(state.is_game_over());
        assert_eq!(state.winner(), Some(Player::Player1));
        assert!(!state.is_draw());
    }

    #[test]
    fn test_game_state_end_as_draw() {
        let mut state = GameState::standard();
        state.end_as_draw();
        assert!(state.is_game_over());
        assert!(state.winner().is_none());
        assert!(state.is_draw());
    }

    #[test]
    fn test_game_move_new() {
        let game_move = GameMove::new(1, Player::Player1, 3, 5);
        assert_eq!(game_move.number(), 1);
        assert_eq!(game_move.player(), Player::Player1);
        assert_eq!(game_move.column(), 3);
        assert_eq!(game_move.row(), 5);
    }

    #[test]
    fn test_game_phase_can_play() {
        assert!(GamePhase::Playing.can_play());
        assert!(!GamePhase::Waiting.can_play());
        assert!(!GamePhase::Ended.can_play());
    }
}
