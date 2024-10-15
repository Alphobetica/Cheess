use cheess::*;
use cheess::PieceColour::*;
use std::time::Duration;



const EMPTY: u8 = 0;
const PAWN: u8 = 10;
const BISHOP: u8 = 13;
const KNIGHT: u8 = 12;
const ROOK: u8 = 11;
const QUEEN: u8 = 14;
const KING: u8 = 15;

// #[test]
pub fn gogo() {
let premove_board = (vec![
      ROOK,  KNIGHT, BISHOP, QUEEN, KING,  BISHOP, KNIGHT, ROOK,
      PAWN,  PAWN,   PAWN,   PAWN,  PAWN,  PAWN,   PAWN,   PAWN,
      EMPTY, EMPTY,  EMPTY,  EMPTY, EMPTY, EMPTY,  EMPTY,  EMPTY,
      EMPTY, EMPTY,  EMPTY,  EMPTY, EMPTY, EMPTY,  EMPTY,  EMPTY,
      EMPTY, EMPTY,  EMPTY,  EMPTY, EMPTY, EMPTY,  EMPTY,  EMPTY,
      EMPTY, EMPTY,  EMPTY,  EMPTY, EMPTY, EMPTY,  EMPTY,  EMPTY,
      PAWN,  PAWN,   PAWN,   PAWN,  PAWN,  PAWN,   PAWN,   PAWN,
      ROOK,  KNIGHT, BISHOP, QUEEN, KING,  BISHOP, KNIGHT, ROOK,],
    
    vec![
      White, White, White, White, White, White, White, White,
      White, White, White, White, White, White, White, White,
      Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
      Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
      Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
      Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
      Black, Black, Black, Black, Black, Black, Black, Black,
      Black, Black, Black, Black, Black, Black, Black, Black]);

let translation = (Coordinates{x:2, y:6}, Coordinates{x:2, y:7});
  
  let piece_board = vec![
    ROOK,  KNIGHT, BISHOP, QUEEN, KING,  BISHOP, KNIGHT, ROOK,
    PAWN,  PAWN,   PAWN,   PAWN,  PAWN,  PAWN,   PAWN,   PAWN,
    EMPTY, EMPTY,  EMPTY,  EMPTY, EMPTY, EMPTY,  EMPTY,  EMPTY,
    EMPTY, EMPTY,  EMPTY,  EMPTY, EMPTY, EMPTY,  EMPTY,  EMPTY,
    EMPTY, EMPTY,  EMPTY,  EMPTY, EMPTY, EMPTY,  EMPTY,  EMPTY,
    EMPTY, EMPTY,  EMPTY,  EMPTY, EMPTY, EMPTY,  EMPTY,  EMPTY,
    PAWN,  PAWN,   PAWN,   PAWN,  PAWN,  PAWN,   PAWN,   PAWN,
    ROOK,  KNIGHT, BISHOP, QUEEN, KING,  BISHOP, KNIGHT, ROOK,];
 
  let piece_color = vec![
    White, White, White, White, White, White, White, White,
    White, White, White, White, White, White, White, White,
    Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
    Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
    Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
    Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
    Black, Black, Black, Black, Black, Black, Black, Black,
    Black, Black, Black, Black, Black, Black, Black, Black];



let mut state: GameState = GameState { 
  board: (piece_board, piece_color),
  move_list: PlayerValidMoves{ black: MoveList::new(), white: MoveList::new()},
  last_move: Some(translation),
  player_turn: 1,     //when white takes turn add 1 when black takes turn -1
  white_can_castle_queenside: true,
  black_can_castle_queenside: true,
  white_can_castle_kingside: true, 
  black_can_castle_kingside: true,
  last_capture_or_pawn_move: 0,
  table_states_since_last_capture_or_pawn_move: vec![boardrep_to_bitboard(&premove_board.clone())],
  en_passant_possible: false, //placeholder dont forget
  white_timer: Duration::from_secs(300), 
  black_timer: Duration::from_secs(300),
  turn_counter: 0,
  white_in_check: false,
  black_in_check: false,
  white_pieces: PieceSet::new(),
  black_pieces: PieceSet::new(),
  clock: std::time::Instant::now(),
  timer_increment: Duration::from_secs(30),
  mode: GameMode::Default,
  game_over: false,
};

let translation = (Coordinates {x: 1, y: 0}, Coordinates { x: 2, y: 2});
get_legal_move_list(&mut state);
take_turn(&mut state, translation);

// let test = ();
println!("{:?}", state.board);
// for translation in state.move_list.white {
//   let index = (usize::from(translation.0), usize::from(translation.1));
//   println!("{:?}", index);
// }
// println!("{:?}", (state.move_list));
}