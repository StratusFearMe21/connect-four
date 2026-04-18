// Custom widgets for Connect Four UI
// Provides specialized widgets for game board, pieces, controls, and game-related UI elements

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, Paragraph, StatefulWidget, StatefulWidgetRef, Widget,
    },
    Frame,
};
use std::time::Duration;
use crate::core::{Board, Piece, Player, GameState};
use crate::ui::{Theme, StylePresets, PaletteColor};

// ============================================================================
// Widget Configuration
// ============================================================================

/// Configuration for board widget display
#[derive(Debug, Clone)]
pub struct BoardWidgetConfig {
    /// Width of each cell in characters
    pub cell_width: usize,
    /// Height of each cell in lines
    pub cell_height: usize,
    /// Show grid lines
    pub show_grid: bool,
    /// Highlight last move
    pub highlight_last: bool,
    /// Show column indicators
    pub show_indicators: bool,
    /// Animation duration for piece drops
    pub animation_duration: Duration,
    /// Use Unicode box drawing characters
    pub unicode_mode: bool,
}

impl Default for BoardWidgetConfig {
    fn default() -> Self {
        Self {
            cell_width: 3,
            cell_height: 2,
            show_grid: true,
            highlight_last: true,
            show_indicators: true,
            animation_duration: Duration::from_millis(300),
            unicode_mode: true,
        }
    }
}

/// Configuration for piece display
#[derive(Debug, Clone)]
pub struct PieceDisplayConfig {
    /// Character to use for pieces
    pub piece_char: char,
    /// Character to use for empty cells
    pub empty_char: char,
    /// Use filled circles (Unicode)
    pub use_circles: bool,
    /// Show player number on piece
    pub show_number: bool,
    /// Piece glow effect
    pub glow_effect: bool,
}

impl Default for PieceDisplayConfig {
    fn default() -> Self {
        Self {
            piece_char: '●',
            empty_char: '○',
            use_circles: true,
            show_number: false,
            glow_effect: false,
        }
    }
}

// ============================================================================
// Board Widget
// ============================================================================

/// Widget for displaying the Connect Four game board
pub struct BoardWidget<'a> {
    /// The game board to display
    board: &'a Board,
    /// Display configuration
    config: BoardWidgetConfig,
    /// Piece display configuration
    piece_config: PieceDisplayConfig,
    /// Current theme
    theme: &'a Theme,
    /// Position of last move (if any)
    last_move: Option<(usize, usize)>,
    /// Current game state
    game_state: GameState,
}

impl<'a> BoardWidget<'a> {
    pub fn new(board: &'a Board, theme: &'a Theme) -> Self {
        Self {
            board,
            config: BoardWidgetConfig::default(),
            piece_config: PieceDisplayConfig::default(),
            theme,
            last_move: None,
            game_state: GameState::InProgress,
        }
    }

    pub fn with_config(mut self, config: BoardWidgetConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_piece_config(mut self, piece_config: PieceDisplayConfig) -> Self {
        self.piece_config = piece_config;
        self
    }

    pub fn with_last_move(mut self, last_move: Option<(usize, usize)>) -> Self {
        self.last_move = last_move;
        self
    }

    pub fn with_game_state(mut self, game_state: GameState) -> Self {
        self.game_state = game_state;
        self
    }

    fn render_cell(&self, buffer: &mut Buffer, area: Rect, row: usize, col: usize) {
        let piece = self.board.get_piece(row, col);
        let is_last = self.last_move == Some((row, col));

        let (style, char) = match piece {
            None => (
                self.theme.get_style(PaletteColor::Background),
                if self.piece_config.use_circles {
                    '○'
                } else {
                    self.piece_config.empty_char
                }
            ),
            Some(p) => {
                let base_color = match p.player {
                    Player::Player1 => PaletteColor::Player1,
                    Player::Player2 => PaletteColor::Player2,
                };
                let mut style = self.theme.get_style(base_color);

                if is_last && self.config.highlight_last {
                    style = style.add_modifier(Modifier::BOLD | Modifier::REVERSED);
                }

                if self.piece_config.glow_effect {
                    style = style.add_modifier(Modifier::ITALIC);
                }

                let char = if self.piece_config.use_circles {
                    '●'
                } else if self.piece_config.show_number {
                    match p.player {
                        Player::Player1 => '1',
                        Player::Player2 => '2',
                    }
                } else {
                    self.piece_config.piece_char
                };

                (style, char)
            }
        };

        // Render the cell
        let inner_area = if self.config.show_grid {
            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width.saturating_sub(2),
                height: area.height.saturating_sub(2),
            }
        } else {
            area
        };

        for y in inner_area.top()..inner_area.bottom() {
            for x in inner_area.left()..inner_area.right() {
                buffer.get_mut(x, y).set_style(style);
                if x == inner_area.left() + inner_area.width / 2
                    && y == inner_area.top() + inner_area.height / 2
                {
                    buffer.get_mut(x, y).set_char(char);
                }
            }
        }

        // Draw grid if enabled
        if self.config.show_grid {
            self.draw_grid(buffer, area);
        }
    }

    fn draw_grid(&self, buffer: &mut Buffer, area: Rect) {
        let border_style = self.theme.get_style(PaletteColor::Border);

        // Horizontal lines
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if y == area.top() || y == area.bottom() - 1 {
                    buffer.get_mut(x, y)
                        .set_char(if self.config.unicode_mode {
                            symbols::line::HORIZONTAL
                        } else {
                            '-'
                        })
                        .set_style(border_style);
                }
            }
        }

        // Vertical lines
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                if x == area.left() || x == area.right() - 1 {
                    buffer.get_mut(x, y)
                        .set_char(if self.config.unicode_mode {
                            symbols::line::VERTICAL
                        } else {
                            '|'
                        })
                        .set_style(border_style);
                }
            }
        }

        // Corners
        let corners = [
            (area.left(), area.top()),
            (area.right() - 1, area.top()),
            (area.left(), area.bottom() - 1),
            (area.right() - 1, area.bottom() - 1),
        ];

        for (x, y) in corners {
            buffer.get_mut(x, y)
                .set_char(if self.config.unicode_mode {
                    if x == area.left() && y == area.top() {
                        symbols::line::TOP_LEFT_CORNER
                    } else if x == area.right() - 1 && y == area.top() {
                        symbols::line::TOP_RIGHT_CORNER
                    } else if x == area.left() && y == area.bottom() - 1 {
                        symbols::line::BOTTOM_LEFT_CORNER
                    } else {
                        symbols::line::BOTTOM_RIGHT_CORNER
                    }
                } else {
                    '+'
                })
                .set_style(border_style);
        }
    }

    fn render_column_indicators(&self, buffer: &mut Buffer, area: Rect) {
        if !self.config.show_indicators {
            return;
        }

        let indicator_area = Rect {
            x: area.x,
            y: area.y.saturating_sub(1),
            width: area.width,
            height: 1,
        };

        for col in 0..self.board.width() {
            let x = indicator_area.x + col * self.config.cell_width as u16
                + self.config.cell_width as u16 / 2;
            let char = char::from_digit(col as u32 + 1, 10).unwrap_or('?');
            buffer.get_mut(x, indicator_area.y)
                .set_char(char)
                .set_style(self.theme.get_style(PaletteColor::Muted));
        }
    }
}

impl<'a> Widget for BoardWidget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Render column indicators above the board
        self.render_column_indicators(buffer, area);

        // Calculate cell dimensions
        let cell_w = self.config.cell_width as u16;
        let cell_h = self.config.cell_height as u16;

        for row in 0..self.board.height() {
            for col in 0..self.board.width() {
                let cell_area = Rect {
                    x: area.x + col as u16 * cell_w,
                    y: area.y + row as u16 * cell_h,
                    width: cell_w,
                    height: cell_h,
                };
                self.render_cell(buffer, cell_area, row, col);
            }
        }

        // Highlight winning line if game is won
        if let GameState::Won(winner, line) = &self.game_state {
            for (row, col) in line {
                let cell_area = Rect {
                    x: area.x + *col as u16 * cell_w,
                    y: area.y + *row as u16 * cell_h,
                    width: cell_w,
                    height: cell_h,
                };
                // Draw win highlight
                for y in cell_area.top()..cell_area.bottom() {
                    for x in cell_area.left()..cell_area.right() {
                        let mut style = buffer.get(x, y).style();
                        style = style.add_modifier(Modifier::REVERSED | Modifier::BOLD);
                        buffer.get_mut(x, y).set_style(style);
                    }
                }
            }
        }
    }
}

// ============================================================================
// Column Selector Widget
// ============================================================================

/// Widget for column selection with visual feedback
pub struct ColumnSelectorWidget<'a> {
    /// Number of columns to display
    num_columns: usize,
    /// Currently selected column (0-indexed)
    selected: usize,
    /// Theme for styling
    theme: &'a Theme,
    /// Show column numbers
    show_numbers: bool,
    /// Show drop indicators
    show_drop_indicators: bool,
    /// Animation state
    animation_progress: f32,
}

impl<'a> ColumnSelectorWidget<'a> {
    pub fn new(num_columns: usize, theme: &'a Theme) -> Self {
        Self {
            num_columns,
            selected: 0,
            theme,
            show_numbers: true,
            show_drop_indicators: true,
            animation_progress: 0.0,
        }
    }

    pub fn with_selected(mut self, selected: usize) -> Self {
        self.selected = selected.min(self.num_columns - 1);
        self
    }

    pub fn with_numbers(mut self, show: bool) -> Self {
        self.show_numbers = show;
        self
    }

    pub fn with_drop_indicators(mut self, show: bool) -> Self {
        self.show_drop_indicators = show;
        self
    }

    pub fn with_animation(mut self, progress: f32) -> Self {
        self.animation_progress = progress.clamp(0.0, 1.0);
        self
    }

    pub fn next_column(&mut self) {
        self.selected = (self.selected + 1) % self.num_columns;
    }

    pub fn previous_column(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        if self.selected >= self.num_columns {
            self.selected = self.num_columns - 1;
        }
    }
}

impl<'a> Widget for ColumnSelectorWidget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let total_width = area.width as usize;
        let column_width = total_width / self.num_columns;

        for col in 0..self.num_columns {
            let x = area.x + col as u16 * column_width as u16;
            let is_selected = col == self.selected;

            let style = if is_selected {
                self.theme.get_style(PaletteColor::Primary)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                self.theme.get_style(PaletteColor::Muted)
            };

            // Draw column indicator
            let indicator = if self.show_numbers {
                char::from_digit(col as u32 + 1, 10).unwrap_or('?').to_string()
            } else {
                "↑".to_string()
            };

            let text = Text::from(vec![
                Line::from(vec![
                    Span::styled("┌", style),
                    Span::styled("─".repeat(column_width - 2), style),
                    Span::styled("┐", style),
                ]),
                Line::from(vec![
                    Span::styled("│", style),
                    Span::styled(
                        format!("{:^width$}", indicator, width = column_width - 2),
                        style,
                    ),
                    Span::styled("│", style),
                ]),
                Line::from(vec![
                    Span::styled("└", style),
                    Span::styled("─".repeat(column_width - 2), style),
                    Span::styled("┘", style),
                ]),
            ]);

            text.render(
                Rect {
                    x,
                    y: area.y,
                    width: column_width as u16,
                    height: 3,
                },
                buffer,
            );
        }

        // Draw drop indicator if enabled and selected
        if self.show_drop_indicators {
            let drop_x = area.x + self.selected as u16 * column_width as u16 + column_width as u16 / 2;
            for y in area.y..area.y + area.height {
                let progress_char = match (self.animation_progress * 4.0) as usize {
                    0 => '║',
                    1 => '│',
                    2 => '·',
                    _ => ' ',
                };
                buffer.get_mut(drop_x, y)
                    .set_char(progress_char)
                    .set_style(self.theme.get_style(PaletteColor::Primary));
            }
        }
    }
}

// ============================================================================
// Player Info Widget
// ============================================================================

/// Widget displaying player information
pub struct PlayerInfoWidget<'a> {
    /// Player 1 name
    player1_name: &'a str,
    /// Player 2 name
    player2_name: &'a str,
    /// Current turn
    current_player: Player,
    /// Theme
    theme: &'a Theme,
    /// Show scores
    show_scores: bool,
    /// Player 1 score
    player1_score: usize,
    /// Player 2 score
    player2_score: usize,
    /// Show game time
    show_time: bool,
    /// Game time in seconds
    game_time: usize,
}

impl<'a> PlayerInfoWidget<'a> {
    pub fn new(player1_name: &'a str, player2_name: &'a str, theme: &'a Theme) -> Self {
        Self {
            player1_name,
            player2_name,
            current_player: Player::Player1,
            theme,
            show_scores: false,
            player1_score: 0,
            player2_score: 0,
            show_time: false,
            game_time: 0,
        }
    }

    pub fn with_current_player(mut self, player: Player) -> Self {
        self.current_player = player;
        self
    }

    pub fn with_scores(mut self, p1: usize, p2: usize) -> Self {
        self.show_scores = true;
        self.player1_score = p1;
        self.player2_score = p2;
        self
    }

    pub fn with_time(mut self, time: usize) -> Self {
        self.show_time = true;
        self.game_time = time;
        self
    }
}

impl<'a> Widget for PlayerInfoWidget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);

        // Player 1 info
        let p1_style = self.theme.get_style(PaletteColor::Player1);
        let p1_active = self.current_player == Player::Player1;
        let p1_border_style = if p1_active {
            p1_style.add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            p1_style
        };

        let mut p1_lines = vec![
            Line::from(vec![
                Span::styled("● Player 1: ", p1_style),
                Span::styled(self.player1_name, p1_style),
            ]),
        ];

        if self.show_scores {
            p1_lines.push(Line::from(vec![
                Span::styled("Score: ", self.theme.get_style(PaletteColor::Muted)),
                Span::styled(self.player1_score.to_string(), p1_style),
            ]));
        }

        if p1_active {
            p1_lines.push(Line::from(vec![
                Span::styled("← YOUR TURN", p1_style.add_modifier(Modifier::BOLD)),
            ]));
        }

        let p1_widget = Paragraph::new(p1_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(p1_border_style),
            )
            .alignment(Alignment::Center);

        p1_widget.render(chunks[0], buffer);

        // Player 2 info
        let p2_style = self.theme.get_style(PaletteColor::Player2);
        let p2_active = self.current_player == Player::Player2;
        let p2_border_style = if p2_active {
            p2_style.add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            p2_style
        };

        let mut p2_lines = vec![
            Line::from(vec![
                Span::styled("● Player 2: ", p2_style),
                Span::styled(self.player2_name, p2_style),
            ]),
        ];

        if self.show_scores {
            p2_lines.push(Line::from(vec![
                Span::styled("Score: ", self.theme.get_style(PaletteColor::Muted)),
                Span::styled(self.player2_score.to_string(), p2_style),
            ]));
        }

        if p2_active {
            p2_lines.push(Line::from(vec![
                Span::styled("YOUR TURN →", p2_style.add_modifier(Modifier::BOLD)),
            ]));
        }

        let p2_widget = Paragraph::new(p2_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(p2_border_style),
            )
            .alignment(Alignment::Center);

        p2_widget.render(chunks[1], buffer);

        // Game time if enabled
        if self.show_time {
            let minutes = self.game_time / 60;
            let seconds = self.game_time % 60;
            let time_text = format!("Time: {:02}:{:02}", minutes, seconds);

            let time_paragraph = Paragraph::new(time_text)
                .style(self.theme.get_style(PaletteColor::Muted))
                .alignment(Alignment::Center);

            let time_area = Rect {
                x: area.x + area.width / 2 - 15,
                y: area.bottom() - 1,
                width: 30,
                height: 1,
            };

            time_paragraph.render(time_area, buffer);
        }
    }
}

// ============================================================================
// Game Status Widget
// ============================================================================

/// Widget displaying game status messages
pub struct GameStatusWidget<'a> {
    /// Status message
    message: &'a str,
    /// Secondary message (e.g., instructions)
    secondary_message: Option<&'a str>,
    /// Status type
    status_type: StatusType,
    /// Theme
    theme: &'a Theme,
    /// Show border
    show_border: bool,
}

/// Type of status message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusType {
    Info,
    Success,
    Warning,
    Error,
    Neutral,
}

impl<'a> GameStatusWidget<'a> {
    pub fn new(message: &'a str, status_type: StatusType, theme: &'a Theme) -> Self {
        Self {
            message,
            secondary_message: None,
            status_type,
            theme,
            show_border: true,
        }
    }

    pub fn with_secondary(mut self, message: &'a str) -> Self {
        self.secondary_message = Some(message);
        self
    }

    pub fn with_border(mut self, show: bool) -> Self {
        self.show_border = show;
        self
    }
}

impl<'a> Widget for GameStatusWidget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let style = match self.status_type {
            StatusType::Info => self.theme.get_style(PaletteColor::Info),
            StatusType::Success => self.theme.get_style(PaletteColor::Success),
            StatusType::Warning => self.theme.get_style(PaletteColor::Warning),
            StatusType::Error => self.theme.get_style(PaletteColor::Error),
            StatusType::Neutral => self.theme.get_style(PaletteColor::Text),
        };

        let mut lines = vec![Line::from(vec![
            Span::styled(self.message, style.add_modifier(Modifier::BOLD)),
        ])];

        if let Some(secondary) = self.secondary_message {
            lines.push(Line::from(vec![
                Span::styled(secondary, self.theme.get_style(PaletteColor::Muted)),
            ]));
        }

        let paragraph = if self.show_border {
            Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(style),
                )
                .alignment(Alignment::Center)
        } else {
            Paragraph::new(lines).alignment(Alignment::Center)
        };

        paragraph.render(area, buffer);
    }
}

// ============================================================================
// Move History Widget
// ============================================================================

/// Widget displaying move history
pub struct MoveHistoryWidget<'a> {
    /// List of moves (column numbers)
    moves: &'a [usize],
    /// Theme
    theme: &'a Theme,
    /// Maximum moves to display
    max_display: usize,
    /// Show move numbers
    show_numbers: bool,
}

impl<'a> MoveHistoryWidget<'a> {
    pub fn new(moves: &'a [usize], theme: &'a Theme) -> Self {
        Self {
            moves,
            theme,
            max_display: 20,
            show_numbers: true,
        }
    }

    pub fn with_max_display(mut self, max: usize) -> Self {
        self.max_display = max;
        self
    }

    pub fn with_numbers(mut self, show: bool) -> Self {
        self.show_numbers = show;
        self
    }
}

impl<'a> Widget for MoveHistoryWidget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let display_moves = if self.moves.len() > self.max_display {
            &self.moves[self.moves.len() - self.max_display..]
        } else {
            self.moves
        };

        let mut lines: Vec<Line> = display_moves
            .iter()
            .enumerate()
            .map(|(i, &col)| {
                let move_num = self.moves.len() - display_moves.len() + i + 1;
                let is_p1 = i % 2 == 0;

                let style = if is_p1 {
                    self.theme.get_style(PaletteColor::Player1)
                } else {
                    self.theme.get_style(PaletteColor::Player2)
                };

                if self.show_numbers {
                    Line::from(vec![
                        Span::styled(format!("{:>3}. ", move_num), self.theme.get_style(PaletteColor::Muted)),
                        Span::styled("Player ", style),
                        Span::styled(if is_p1 { "1" } else { "2" }, style),
                        Span::styled(" → Column ", style),
                        Span::styled((col + 1).to_string(), style.add_modifier(Modifier::BOLD)),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled("Column ", style),
                        Span::styled((col + 1).to_string(), style.add_modifier(Modifier::BOLD)),
                    ])
                }
            })
            .collect();

        if lines.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("No moves yet", self.theme.get_style(PaletteColor::Muted)),
            ]));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title("Move History")
                    .borders(Borders::ALL)
                    .border_style(self.theme.get_style(PaletteColor::Border)),
            )
            .alignment(Alignment::Left);

        paragraph.render(area, buffer);
    }
}

// ============================================================================
// Animated Text Widget
// ============================================================================

/// Widget with animated text effect
pub struct AnimatedTextWidget<'a> {
    /// Text to display
    text: &'a str,
    /// Current animation frame
    frame: usize,
    /// Total frames
    total_frames: usize,
    /// Theme
    theme: &'a Text,
    /// Animation effect
    effect: AnimationEffect,
}

/// Animation effects for text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationEffect {
    Fade,
    Pulse,
    Typewriter,
    Bounce,
    Rainbow,
}

impl<'a> AnimatedTextWidget<'a> {
    pub fn new(text: &'a str, theme: &'a Theme) -> Self {
        Self {
            text,
            frame: 0,
            total_frames: 60,
            theme,
            effect: AnimationEffect::Pulse,
        }
    }

    pub fn with_frame(mut self, frame: usize) -> Self {
        self.frame = frame % self.total_frames;
        self
    }

    pub fn with_effect(mut self, effect: AnimationEffect) -> Self {
        self.effect = effect;
        self
    }

    pub fn next_frame(&mut self) {
        self.frame = (self.frame + 1) % self.total_frames;
    }
}

impl<'a> Widget for AnimatedTextWidget<'a> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let chars: Vec<char> = self.text.chars().collect();
        let progress = self.frame as f32 / self.total_frames as f32;

        let line = match self.effect {
            AnimationEffect::Fade => {
                let alpha = (0.5 + 0.5 * (progress * std::f32::consts::PI * 2.0).sin()) as u8;
                Line::from(vec![Span::styled(
                    self.text,
                    self.theme.get_style(PaletteColor::Text)
                        .fg(Color::Rgb(alpha, alpha, alpha)),
                )])
            }
            AnimationEffect::Pulse => {
                let bold = (progress * 10.0) as usize % 2 == 0;
                Line::from(vec![Span::styled(
                    self.text,
                    self.theme.get_style(PaletteColor::Primary)
                        .add_modifier(if bold { Modifier::BOLD } else { Modifier::empty() }),
                )])
            }
            AnimationEffect::Typewriter => {
                let char_count = (progress * chars.len() as f32) as usize;
                let displayed: String = chars.iter().take(char_count).collect();
                Line::from(vec![Span::styled(
                    displayed,
                    self.theme.get_style(PaletteColor::Text),
                )])
            }
            AnimationEffect::Bounce => {
                let offset = ((progress * 10.0) as usize) % (area.width as usize + self.text.len());
                let padding = " ".repeat(offset);
                Line::from(vec![Span::styled(
                    format!("{}{}", padding, self.text),
                    self.theme.get_style(PaletteColor::Text),
                )])
            }
            AnimationEffect::Rainbow => {
                let colors = [
                    Color::Red,
                    Color::Yellow,
                    Color::Green,
                    Color::Cyan,
                    Color::Blue,
                    Color::Magenta,
                ];
                let spans: Vec<Span> = chars
                    .iter()
                    .enumerate()
                    .map(|(i, &c)| {
                        let color_idx = (i + self.frame) % colors.len();
                        Span::styled(c.to_string(), Style::default().fg(colors[color_idx]))
                    })
                    .collect();
                Line::from(spans)
            }
        };

        Paragraph::new(vec![line])
            .alignment(Alignment::Center)
            .render(area, buffer);
    }
}

// ============================================================================
// Progress Bar Widget
// ============================================================================

/// Custom progress bar widget
pub struct ProgressBarWidget {
    /// Progress value (0.0 to 1.0)
    progress: f32,
    /// Label to display
    label: String,
    /// Theme
    theme: Theme,
    /// Show percentage
    show_percentage: bool,
    /// Bar width in characters
    width: usize,
}

impl ProgressBarWidget {
    pub fn new(progress: f32, theme: Theme) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            label: String::new(),
            theme,
            show_percentage: true,
            width: 40,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }
}

impl Widget for ProgressBarWidget {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let filled = (self.progress * self.width as f32) as usize;
        let empty = self.width - filled;

        let filled_str = "█".repeat(filled);
        let empty_str = "░".repeat(empty);

        let mut text = format!("[{}{}]", filled_str, empty_str);

        if self.show_percentage {
            text.push_str(&format!(" {:.0}%", self.progress * 100.0));
        }

        if !self.label.is_empty() {
            text = format!("{} {}", self.label, text);
        }

        let paragraph = Paragraph::new(text)
            .style(self.theme.get_style(PaletteColor::Text))
            .alignment(Alignment::Left);

        paragraph.render(area, buffer);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_widget_config_default() {
        let config = BoardWidgetConfig::default();
        assert_eq!(config.cell_width, 3);
        assert_eq!(config.cell_height, 2);
        assert!(config.show_grid);
        assert!(config.highlight_last);
    }

    #[test]
    fn test_piece_display_config_default() {
        let config = PieceDisplayConfig::default();
        assert_eq!(config.piece_char, '●');
        assert_eq!(config.empty_char, '○');
        assert!(config.use_circles);
    }

    #[test]
    fn test_status_type_display() {
        let types = [
            StatusType::Info,
            StatusType::Success,
            StatusType::Warning,
            StatusType::Error,
            StatusType::Neutral,
        ];
        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_animation_effects() {
        let effects = [
            AnimationEffect::Fade,
            AnimationEffect::Pulse,
            AnimationEffect::Typewriter,
            AnimationEffect::Bounce,
            AnimationEffect::Rainbow,
        ];
        assert_eq!(effects.len(), 5);
    }

    #[test]
    fn test_column_selector_navigation() {
        let theme = Theme::default();
        let mut selector = ColumnSelectorWidget::new(7, &theme);

        assert_eq!(selector.selected, 0);
        selector.next_column();
        assert_eq!(selector.selected, 1);
        selector.previous_column();
        assert_eq!(selector.selected, 0);

        selector.selected = 6;
        selector.next_column();
        assert_eq!(selector.selected, 0);

        selector.previous_column();
        assert_eq!(selector.selected, 6);
    }

    #[test]
    fn test_progress_bar_clamp() {
        let theme = Theme::default();
        let bar = ProgressBarWidget::new(1.5, theme);
        assert_eq!(bar.progress, 1.0);

        let bar = ProgressBarWidget::new(-0.5, theme);
        assert_eq!(bar.progress, 0.0);
    }
}
