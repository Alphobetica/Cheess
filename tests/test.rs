// use cheess::{run, get_index, get_coordinates, Knight, GameState};
// use std::time::Duration;

// const EMPTY: u8 = 0;
// const PAWN: u8 = 10;
// const BISHOP: u8 = 13;
// const KNIGHT: u8 = 12;
// const ROOK: u8 = 11;
// const QUEEN: u8 = 14;
// const KING: u8 = 15;

// const WHITE: u8 = 1;
// const BLACK: u8 = 2;
// // #[test]
// fn main() {


  
//   let knight_board = vec![
//     EMPTY, EMPTY, EMPTY, EMPTY,  EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, EMPTY,  EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, EMPTY,  EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, EMPTY,  EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, KNIGHT, EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, EMPTY,  EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, EMPTY,  EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, EMPTY,  EMPTY, EMPTY, EMPTY, EMPTY];
 
//   let piece_color = vec![
//     EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, BLACK, EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY,
//     EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY];



// let mut game: GameState = GameState { 
//   board: (knight_board, piece_color),
//   player_turn: 2,     //when white takes turn add 1 when black takes turn -1
//   white_can_castle_queenside: true,
//   black_can_castle_queenside: true,
//   white_can_castle_kingside: true, 
//   black_can_castle_kingside: true,
//   last_capture_or_pawn_move: 0,
//   table_state_since_last_capture_or_pawn_move: Vec::new(), 
//   en_passant_possible: None, //placeholder dont forget
//   white_timer: Duration::from_secs(300), 
//   black_timer: Duration::from_secs(300),    
// };
// let test_list = Knight::get_valid_moves(game);
// let test_test: Vec<_> = test_list.iter().map(|element| (get_index(element.0), get_index(element.1))).collect();
// println!("{:?}", test_test);
// }


