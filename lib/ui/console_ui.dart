import 'dart:io';

void displayBoard(List<int> board){
  // clear screen
  stdout.write('\x1B[2J\x1B[0;0H');

  const reset = '\x1B[0m';
  const red = '\x1B[31m';
  const yellow = '\x1B[33m';
  const white = '\x1B[37m';

  // header
  stdout.writeln('====================');
  stdout.writeln('   CONNECT 4');
  stdout.writeln('====================\n');

  // print board
  for (int row = 0; row < 6; row++) {
    for (int col = 0; col < 7; col++) {
      int val = board[row * 7 + col];

      if (val == 1) {
        stdout.write('$red● $reset');
      } else if (val == 2) {
        stdout.write('$yellow● $reset');
      } else {
        stdout.write('$white. $reset');
      }
    }
    stdout.writeln();
  }

  // column labels
  stdout.writeln('0 1 2 3 4 5 6\n');

  // determine player from count
  int count1 = board.where((e) => e == 1).length;
  int count2 = board.where((e) => e == 2).length;

  bool isPlayer1Turn = count1 == count2;

  if (isPlayer1Turn) {
    stdout.writeln('Current Player: 1 (${red}Red$reset)');
  } else {
    stdout.writeln('Current Player: 2 (${yellow}Yellow$reset)');
  }
}

String getUserInput(String? prompt){
  if (prompt != null) {
    stdout.writeln(prompt);
  }

  String? input = stdin.readLineSync();

  return input ?? '';
}

void displayWinner(String winner){
  // clear screen
  stdout.write('\x1B[2J\x1B[0;0H');

  stdout.writeln('====================');
  stdout.writeln('      $winner');
  stdout.writeln('====================');
}