# AGENTS.md

This document helps agents work effectively in the Connect Four Dart project.

## Project Overview

This is a command-line Connect Four game written in Dart. It features a console-based UI with colored pieces and turn-based gameplay for two players.

## Essential Commands

### Dependency Management
```bash
dart pub get          # Install/update dependencies
```

### Running the Application
```bash
dart run              # Run the application (requires entry point in bin/)
```

### Testing
```bash
dart test             # Run all tests
dart test test/specific_test.dart  # Run specific test file
```

### Analysis and Linting
```bash
dart analyze          # Run static analysis with lints
```

## Project Structure

```
connect-four-rs/
├── lib/
│   ├── controller/
│   │   └── game_manager.dart    # Game logic and state management
│   └── ui/
│       └── console_ui.dart      # Console UI functions
├── pubspec.yaml                 # Dependencies and project config
├── analysis_options.yaml        # Linting rules (uses package:lints/recommended.yaml)
└── README.md
```

### Architecture
The project follows an MVC-like separation:
- **Model**: Game state (managed in `game_manager.dart`)
- **View**: Console UI (in `console_ui.dart`)
- **Controller**: Game flow and logic (in `game_manager.dart`)

### Code Organization

**lib/controller/game_manager.dart**:
- Game state management (global variables)
- Game flow control (`startGame`, `turn`)
- Move validation (`isValidPlacement`)
- Win detection (`hasWon`) - checks horizontal, vertical, and diagonal patterns

**lib/ui/console_ui.dart**:
- Terminal display with ANSI escape codes
- Board rendering with colored pieces (Red = Player 1, Yellow = Player 2)
- User input handling
- Winner display

## Code Patterns and Conventions

### Naming Conventions
- Functions and variables: `camelCase`
- Types: `PascalCase`
- Private members: `_prefixedWithUnderscore` (not currently used)
- Files: `snake_case.dart`

### Import Style
```dart
import 'dart:io';
import 'package:connect_four/ui/console_ui.dart';
```

### Indentation and Formatting
- 2-space indentation (Dart standard)
- Uses `package:lints/recommended.yaml` for consistent style

### State Management
- Game state is managed using module-level global variables in `game_manager.dart`:
  - `grid`: 6x7 List of Lists (0 = empty, 1 = player 1, 2 = player 2)
  - `isPlayer1Turn`: Boolean for turn tracking

### Game Logic Patterns
- Grid access: `grid[row][col]` (6 rows, 7 columns)
- Column validation: Check if top row is empty: `grid[0][col] == 0`
- Piece placement: Find lowest empty row in column (from row 5 up to 0)
- Win checking: Four-directional scan (horizontal, vertical, diagonal-down, diagonal-up)

### UI Patterns
- Terminal clearing: `stdout.write('\x1B[2J\x1B[0;0H')`
- Colored output: ANSI escape codes for red (`\x1B[31m`) and yellow (`\x1B[33m`)
- Input handling: `stdin.readLineSync()` with null coalescing

### Input Validation Pattern
```dart
String input = getUserInput("Prompt:");
int col = int.tryParse(input) ?? -1;  // Returns -1 on parse failure
```

## Dependencies

### Production Dependencies
- `path: ^1.9.0` - Path manipulation utilities

### Development Dependencies
- `lints: ^6.0.0` - Linting rules and static analysis
- `test: ^1.25.6` - Testing framework

## Important Gotchas

### Entry Point
- **No bin/ directory exists** - The project needs an entry point file (typically `bin/main.dart`) to run
- Currently, the library code exists but no executable entry point

### Global State
- Game state uses module-level variables rather than classes
- State is reset explicitly when starting a new game in `startGame()`

### Terminal UI
- Uses raw ANSI escape codes directly (not wrapped in a library)
- Terminal clearing and color codes are hardcoded strings

### No Tests Yet
- No test/ directory exists
- No unit tests for game logic or UI functions
- Consider adding tests for:
  - Win detection algorithm
  - Move validation
  - Board state management

### Board Coordinate System
- Grid is 6 rows × 7 columns (0-indexed)
- Row 0 is top, Row 5 is bottom
- Column 0-6 left to right
- Pieces "fall" to bottom: placement searches from row 5 upward

## Dart SDK Version

- Minimum SDK: ^3.11.3
- Use `dart --version` to check installed version

## Analysis

The project uses `dart analyze` with the `package:lints/recommended.yaml` rule set. Run analysis before committing changes:

```bash
dart analyze
```

## Testing Recommendations

When adding tests:
1. Create `test/` directory
2. Follow Dart test conventions: `test/game_manager_test.dart`, `test/console_ui_test.dart`
3. Test critical game logic:
   - Win detection in all four directions
   - Boundary conditions for column input
   - Board full detection
   - Player turn alternation

## Common Tasks

### Adding a new feature
1. Update game logic in `lib/controller/game_manager.dart`
2. Update UI in `lib/ui/console_ui.dart` if needed
3. Add dependencies to `pubspec.yaml` if required
4. Run `dart pub get` after modifying dependencies
5. Run `dart analyze` to check for issues

### Running the game
Currently needs entry point. To run, create `bin/main.dart`:
```dart
import 'package:connect_four/controller/game_manager.dart';

void main() {
  startGame();
}
```
Then run with `dart run`

### Debugging
- Use `print()` statements for console debugging
- Check state of `grid` variable during gameplay
- Verify win detection with different board configurations
