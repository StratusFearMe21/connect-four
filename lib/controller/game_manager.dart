import 'package:connect_four/ui/console_ui.dart';

var grid = List.generate(6, (i) => List.filled(7, 0));
bool isPlayer1Turn = true;

/// Starts Game And Quits when prompted to
void startGame() {
  while (true) {
    String result = getUserInput("Ready to Play(y/N)");
    if (result.toLowerCase() == "n") {
      break;
    }

    // reset board for new game
    grid = List.generate(6, (i) => List.filled(7, 0));
    isPlayer1Turn = true;

    while (true) {
      bool gameOver = turn();
      if (gameOver) break;
      isPlayer1Turn = !isPlayer1Turn;
    }
  }
}

bool turn() {
  int player = isPlayer1Turn ? 1 : 2;

  // display current board state
  displayBoard(grid.expand((row) => row).toList());

  String input = getUserInput("Player $player - choose column (0-6):");
  int col = int.tryParse(input) ?? -1;

  if (!isValidPlacement(col)) {
    displayBoard(grid.expand((row) => row).toList());
    print("Invalid move. Try again.");
    return false;
  }

  // place piece
  for (int row = 5; row >= 0; row--) {
    if (grid[row][col] == 0) {
      grid[row][col] = player;
      break;
    }
  }

  // display updated board
  displayBoard(grid.expand((row) => row).toList());

  // win check
  if (hasWon()) {
    displayWinner("Player $player wins!");
    return true;
  }

  // draw check (count all 0s in grid)
  bool boardFull = !grid.any((row) => row.contains(0));

  if (boardFull) {
    displayWinner("It's a draw!");
    return true;
  }

  return false;
}

bool isValidPlacement(int col) {
  if (col < 0 || col >= 7) return false;
  return grid[0][col] == 0;
}


bool hasWon() {
  // horizontal
  for (int row = 0; row < 6; row++) {
    for (int col = 0; col < 4; col++) {
      int p = grid[row][col];
      if (p != 0 &&
          p == grid[row][col + 1] &&
          p == grid[row][col + 2] &&
          p == grid[row][col + 3]) {
        return true;
      }
    }
  }

  // vertical
  for (int col = 0; col < 7; col++) {
    for (int row = 0; row < 3; row++) {
      int p = grid[row][col];
      if (p != 0 &&
          p == grid[row + 1][col] &&
          p == grid[row + 2][col] &&
          p == grid[row + 3][col]) {
        return true;
      }
    }
  }

  // diagonal down-right
  for (int row = 0; row < 3; row++) {
    for (int col = 0; col < 4; col++) {
      int p = grid[row][col];
      if (p != 0 &&
          p == grid[row + 1][col + 1] &&
          p == grid[row + 2][col + 2] &&
          p == grid[row + 3][col + 3]) {
        return true;
      }
    }
  }

  // diagonal up-right
  for (int row = 3; row < 6; row++) {
    for (int col = 0; col < 4; col++) {
      int p = grid[row][col];
      if (p != 0 &&
          p == grid[row - 1][col + 1] &&
          p == grid[row - 2][col + 2] &&
          p == grid[row - 3][col + 3]) {
        return true;
      }
    }
  }

  return false;
}