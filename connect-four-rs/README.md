# Connect Four in Rust

A terminal-based Connect Four game implemented in Rust using the [ratatui](https://github.com/ratatui-org/ratatui) TUI framework.

## Features

- Two-player local gameplay
- Terminal UI with colored pieces (Red for Player 1, Yellow for Player 2)
- Real-time win detection (horizontal, vertical, and diagonal)
- Draw detection when board is full
- Keyboard controls for easy gameplay
- Game restart functionality

## Controls

- **0-6**: Enter column number
- **Enter**: Submit move
- **R**: Restart game (when game is over)
- **Q / ESC**: Quit game

## How to Play

1. Run the game with `cargo run`
2. Enter a column number (0-6) and press Enter to place your piece
3. Pieces fall to the lowest available position in the column
4. Alternate turns with the other player
5. First player to connect 4 pieces wins!
6. Press 'R' to start a new game after the game ends

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

## Testing

The project includes comprehensive unit tests for game logic:

```bash
cargo test
```

Tests cover:
- Move validation
- Win detection in all directions
- Board full detection
- Player turn alternation
- Game reset functionality

## Dependencies

- `ratatui`: Terminal UI framework
- `crossterm`: Terminal handling for keyboard events

## Implementation Details

### Game State
- 6×7 board with `Cell` enum (Empty, Player1, Player2)
- Tracks current player and game status
- Win detection scans all four directions from the last move

### Architecture
- **Game struct**: Manages board state, current player, and game logic
- **Cell enum**: Represents board cells with associated colors and symbols
- **draw_board**: Renders the game state using ratatui widgets
- **Event loop**: Handles keyboard input and updates terminal

### Win Detection
The game checks for wins by scanning four directions from the last placed piece:
- Horizontal (left/right)
- Vertical (up/down)
- Diagonal down-right
- Diagonal up-right

Each direction is checked bidirectionally to count consecutive pieces of the same color.

## License

This project is provided as-is for educational purposes.
