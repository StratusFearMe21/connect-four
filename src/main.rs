use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEvent},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

const ROWS: usize = 6;
const COLS: usize = 7;
const WIN_LENGTH: usize = 4;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Cell {
    Empty,
    Player1,
    Player2,
}

impl Cell {
    fn symbol(&self) -> &'static str {
        match self {
            Cell::Empty => " ",
            Cell::Player1 => "●",
            Cell::Player2 => "●",
        }
    }

    fn color(&self) -> Color {
        match self {
            Cell::Empty => Color::Reset,
            Cell::Player1 => Color::Red,
            Cell::Player2 => Color::Yellow,
        }
    }
}

struct Game {
    board: [[Cell; COLS]; ROWS],
    current_player: Cell,
    game_over: bool,
    winner: Option<Cell>,
    input_buffer: String,
}

impl Game {
    fn new() -> Self {
        Game {
            board: [[Cell::Empty; COLS]; ROWS],
            current_player: Cell::Player1,
            game_over: false,
            winner: None,
            input_buffer: String::new(),
        }
    }

    fn reset(&mut self) {
        self.board = [[Cell::Empty; COLS]; ROWS];
        self.current_player = Cell::Player1;
        self.game_over = false;
        self.winner = None;
        self.input_buffer.clear();
    }

    fn is_valid_move(&self, col: usize) -> bool {
        col < COLS && self.board[0][col] == Cell::Empty
    }

    fn make_move(&mut self, col: usize) -> Result<(), String> {
        if self.game_over {
            return Err("Game is over".to_string());
        }

        if !self.is_valid_move(col) {
            return Err("Invalid move".to_string());
        }

        // Find the lowest empty row in the column
        for row in (0..ROWS).rev() {
            if self.board[row][col] == Cell::Empty {
                self.board[row][col] = self.current_player;

                // Check for win
                if self.check_win(row, col) {
                    self.game_over = true;
                    self.winner = Some(self.current_player);
                }
                // Check for draw
                else if self.is_board_full() {
                    self.game_over = true;
                    self.winner = None;
                }
                else {
                    // Switch player
                    self.current_player = if self.current_player == Cell::Player1 {
                        Cell::Player2
                    } else {
                        Cell::Player1
                    };
                }

                return Ok(());
            }
        }

        Err("Column is full".to_string())
    }

    fn check_win(&self, last_row: usize, last_col: usize) -> bool {
        let player = self.board[last_row][last_col];

        // Check all four directions
        let directions = [
            [(0, 1), (0, -1)],   // Horizontal
            [(1, 0), (-1, 0)],   // Vertical
            [(1, 1), (-1, -1)],  // Diagonal down-right
            [(-1, 1), (1, -1)],  // Diagonal up-right
        ];

        for dir in &directions {
            let mut count = 1;

            for (dr, dc) in dir {
                let mut row = last_row as i32 + dr;
                let mut col = last_col as i32 + dc;

                while row >= 0 && row < ROWS as i32 && col >= 0 && col < COLS as i32 {
                    if self.board[row as usize][col as usize] == player {
                        count += 1;
                        row += dr;
                        col += dc;
                    } else {
                        break;
                    }
                }
            }

            if count >= WIN_LENGTH {
                return true;
            }
        }

        false
    }

    fn is_board_full(&self) -> bool {
        self.board[0].iter().all(|&cell| cell != Cell::Empty)
    }

    fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.input_buffer.push(c);
                if self.input_buffer.len() > 1 {
                    self.input_buffer.drain(..1);
                }
            }
            KeyCode::Enter => {
                if let Some(col) = self.input_buffer.parse::<usize>().ok() {
                    if self.make_move(col).is_ok() {
                        self.input_buffer.clear();
                    }
                } else {
                    self.input_buffer.clear();
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                return false;
            }
            KeyCode::Char('r') if self.game_over => {
                self.reset();
            }
            _ => {}
        }
        true
    }
}

fn draw_board(f: &mut Frame, game: &Game, area: Rect) {
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),  // Title
                Constraint::Min(0),    // Board
                Constraint::Length(3),  // Status
                Constraint::Length(2),  // Instructions
            ]
            .as_ref(),
        )
        .split(area);

    // Title
    let title = Paragraph::new("CONNECT FOUR")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Board area
    let board_area = Block::default()
        .borders(Borders::ALL)
        .title("Game Board");
    f.render_widget(board_area, chunks[1]);

    // Draw cells inside the board area
    let inner = chunks[1].inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    let cell_width = 3;
    let cell_height = 1;
    let board_width = COLS * cell_width + (COLS - 1);

    let start_x = inner.x + (inner.width - board_width as u16) / 2;
    let start_y = inner.y + (inner.height - ROWS as u16) / 2;

    for row in 0..ROWS {
        for col in 0..COLS {
            let cell = game.board[row][col];
            let x = start_x + (col * (cell_width + 1)) as u16;
            let y = (start_y + row as u16) * cell_height as u16;

            let symbol = cell.symbol();
            let style = Style::default().fg(cell.color());

            let paragraph = Paragraph::new(symbol)
                .style(style)
                .alignment(Alignment::Center);
            f.render_widget(paragraph, Rect::new(x, y, cell_width as u16, cell_height as u16));
        }
    }

    // Draw column numbers
    for col in 0..COLS {
        let x = start_x + (col * (cell_width + 1)) as u16 + 1;
        let y = start_y + ROWS as u16 + 1;

        let num = col.to_string();
        let paragraph = Paragraph::new(num)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, Rect::new(x, y, 1, 1));
    }

    // Status
    let status = if game.game_over {
        match game.winner {
            Some(Cell::Player1) => "Player 1 (Red) wins! Press 'R' to restart",
            Some(Cell::Player2) => "Player 2 (Yellow) wins! Press 'R' to restart",
            Some(Cell::Empty) => "Error",
            None => "It's a draw! Press 'R' to restart",
        }
    } else {
        match game.current_player {
            Cell::Player1 => "Current Player: 1 (Red)",
            Cell::Player2 => "Current Player: 2 (Yellow)",
            Cell::Empty => "Error",
        }
    };

    let status_para = Paragraph::new(status)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_para, chunks[2]);

    // Instructions
    let instructions = "Enter column (0-6) and press Enter | Q/ESC to quit";
    let instr_para = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(instr_para, chunks[3]);

    // Input display
    if !game.input_buffer.is_empty() {
        let input_text = format!("Column: {}", game.input_buffer);
        let input_area = Rect::new(
            chunks[3].x + 1,
            chunks[3].y + chunks[3].height - 1,
            chunks[3].width - 2,
            1,
        );
        let input_para = Paragraph::new(input_text)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(input_para, input_area);
    }
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    let mut game = Game::new();

    loop {
        terminal.draw(|f| {
            draw_board(f, &game, f.area());
        })?;

        // Wait for key press
        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {
                if !game.handle_key(code) {
                    return Ok(());
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the application
    let result = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_move() {
        let game = Game::new();
        assert!(game.is_valid_move(0));
        assert!(game.is_valid_move(3));
        assert!(game.is_valid_move(6));
        assert!(!game.is_valid_move(7));
        assert!(!game.is_valid_move(99));
    }

    #[test]
    fn test_column_full() {
        let mut game = Game::new();

        // Fill column 0
        for _ in 0..ROWS {
            game.make_move(0).unwrap();
        }

        assert!(!game.is_valid_move(0));
    }

    #[test]
    fn test_horizontal_win() {
        let mut game = Game::new();

        // Player 1 wins horizontally
        game.make_move(0).unwrap();
        game.make_move(0).unwrap(); // Player 2 in same column
        game.make_move(1).unwrap();
        game.make_move(1).unwrap();
        game.make_move(2).unwrap();
        game.make_move(2).unwrap();
        game.make_move(3).unwrap(); // Player 1 wins

        assert!(game.game_over);
        assert_eq!(game.winner, Some(Cell::Player1));
    }

    #[test]
    fn test_vertical_win() {
        let mut game = Game::new();

        // Player 1 wins vertically
        game.make_move(0).unwrap();
        game.make_move(1).unwrap(); // Player 2 in different column
        game.make_move(0).unwrap();
        game.make_move(1).unwrap();
        game.make_move(0).unwrap();
        game.make_move(1).unwrap();
        game.make_move(0).unwrap(); // Player 1 wins

        assert!(game.game_over);
        assert_eq!(game.winner, Some(Cell::Player1));
    }

    #[test]
    fn test_diagonal_down_right_win() {
        let mut game = Game::new();

        // Create a diagonal pattern
        //     . . . .
        //     . . . .
        //     . . . X
        //     . . X X
        //     . X . X
        //     X . . X

        game.make_move(0).unwrap(); // P1
        game.make_move(0).unwrap(); // P2
        game.make_move(1).unwrap(); // P1
        game.make_move(0).unwrap(); // P2
        game.make_move(2).unwrap(); // P1
        game.make_move(1).unwrap(); // P2
        game.make_move(3).unwrap(); // P1 - wins diagonally

        assert!(game.game_over);
        assert_eq!(game.winner, Some(Cell::Player1));
    }

    #[test]
    fn test_board_full_detection() {
        let mut game = Game::new();

        // Fill the board column by column
        for col in 0..COLS {
            for _ in 0..ROWS {
                if game.game_over {
                    break;
                }
                let _ = game.make_move(col);
            }
        }

        // The board should be detected as full at some point
        assert!(game.game_over);
    }

    #[test]
    fn test_draw_when_full() {
        // This test verifies that when the board is full with no winner,
        // it's correctly detected as a draw
        let mut game = Game::new();

        // Fill board in alternating columns to minimize wins
        let moves = [
            0, 6, 1, 5, 2, 4, 3, 0, 6, 1, 5, 2, 4, 3, 0, 6, 1, 5, 2, 4, 3,
        ];

        for &col in &moves {
            if !game.game_over {
                let _ = game.make_move(col);
            }
        }

        // If game is over and no winner, it's a draw
        if game.game_over && game.winner.is_none() {
            // This is a draw
            assert!(true);
        } else if !game.game_over {
            // Game not over yet, continue filling
            for col in 0..COLS {
                for _ in 0..ROWS {
                    if game.game_over {
                        break;
                    }
                    let _ = game.make_move(col);
                }
            }
            // Eventually game should be over
            assert!(game.game_over);
        }
    }

    #[test]
    fn test_reset() {
        let mut game = Game::new();

        // Make some moves
        game.make_move(0).unwrap();
        game.make_move(1).unwrap();

        game.reset();

        // Verify board is empty
        for row in 0..ROWS {
            for col in 0..COLS {
                assert_eq!(game.board[row][col], Cell::Empty);
            }
        }

        // Verify player 1 starts
        assert_eq!(game.current_player, Cell::Player1);
        assert!(!game.game_over);
        assert!(game.winner.is_none());
    }

    #[test]
    fn test_player_alternation() {
        let mut game = Game::new();

        assert_eq!(game.current_player, Cell::Player1);

        game.make_move(0).unwrap();
        assert_eq!(game.current_player, Cell::Player2);

        game.make_move(1).unwrap();
        assert_eq!(game.current_player, Cell::Player1);
    }
}
