
import 'package:connect_four/ui/console_ui.dart';

var grid =  List.generate(6, (i) => List.filled(7, 0));
bool isPlayer1Turn = true;

/// Starts Game And Quits when prompted to
void startGame(){
  while(true){
    String result = getUserInput("Ready to Play(y/N)");
    if(result== "n"){
      break;
    }
    while(true){
      while(turn()) {}
      isPlayer1Turn ^= true; //flips the value
    }
  }
}

//bool
bool turn(){

  return false;
}

//bool
void isValidPlacement(List<int> pos){

}

//bool
void hasWon(){

}

