mod gameloop;
mod input;
mod threadpool;
mod bitboard;

use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::ops::{Index, IndexMut};

pub use gameloop::*;
pub use input::*;
pub use threadpool::*;
pub use bitboard::*;

use crate::PieceColour::*;

pub const EMPTY: u8 = 0;
pub const PAWN: u8 = 10;
pub const BISHOP: u8 = 13;
pub const KNIGHT: u8 = 12;
pub const ROOK: u8 = 11;
pub const QUEEN: u8 = 14;
pub const KING: u8 = 15;


pub fn run() {
    let mut game = GameState::new();

    println!("Select Mode");
    
    
    let mut input = String::new();
    loop {
        std::io::stdin()
          .read_line(&mut input)
          .expect("Error reading input");
        input = input.trim().to_owned();

         if input == "default" {
            println!("Default game mode selected");
            break;
        } else if input == "blitz" {
            println!("Blitz game mode selected");
            game.blitz_mode();
            break;
        } else {
            println!("Invalid game mode: {}", input);
            input.clear();
        }
    }
   
    
    game.white_timer = game.white_timer + game.timer_increment;

    get_legal_move_list(&mut game);

    let threadpool = ThreadPool::new(2).expect("Error creating threads");
    let input_struct = Arc::new(Mutex::new(UserInput { input: String::new() }));
    

    
    let mut event_loop = gameloop::Dispatcher::new(&threadpool);
    let game_state_pointer = Arc::new(Mutex::new(game));
    
    
    event_loop.register_handler(Event::UserInput, input_struct.clone());
    event_loop.register_handler(Event::MoveInput, game_state_pointer.clone());


    event_loop.start();    
    
    'main_loop: loop {
        std::thread::sleep(std::time::Duration::from_millis(40)); 
        if game_state_pointer.lock().unwrap().game_over {
            println!("Game Over");
            break 'main_loop
        }
        
        

        // probably should be async
        event_loop.trigger_event(Event::UserInput, Vec::new());
        std::thread::sleep(std::time::Duration::from_millis(40));
        // or else main locks inputstruct before input
        // should be .join() with threadpool
        let input = input_struct.lock().unwrap().input.clone();

        println!("Line: {input}");

        //Commands list
        if input == "exit" {
            break 'main_loop;
        }
        
        if input == "reset" {
            game_state_pointer.lock().unwrap().reset();
        }

        if input == "resign" {
            let mut state = game_state_pointer.lock().unwrap();
            state.game_over = true;
            if state.player_turn == 1 {
                println!("White Resigns")
            } else if state.player_turn == 2 {
                println!("Black Resigns")
            } else {
                println!("Invalid player turn")
            }
        }
        
        
        let event = gameloop::Event::MoveInput;

        if let Ok(payload) = parse_payload_from_index(&input) {
            event_loop.trigger_event(event, payload);
        } else if let Some(payload) = parse_payload_from_coordinates(&input) {
            event_loop.trigger_event(event, payload);
        } /* else {
            std::thread::sleep(std::time::Duration::from_secs(1));
        } */
        // println!("{} :  ", game.board, );

        
    }
    
    //drop(event_loop);
    
    
}

type BoardRep = (Vec<u8>, Vec<PieceColour>); //1 array of piece type in space and 2nd array of colour of piece type
type PastBoardRep = Vec<BoardRep>; // 3 move draw rule
#[derive(Debug, Clone)]
pub struct GameState {
    pub board: BoardRep, //look into bitboards in the future instead of vec array
    pub move_list: PlayerValidMoves,
    pub last_move: Option<Move>,
    // pub board: Rc<BoardRep>, //look into bitboards in the future instead of vec array
    pub player_turn: u8,
    pub white_can_castle_queenside: bool,
    pub black_can_castle_queenside: bool,
    pub white_can_castle_kingside: bool,
    pub black_can_castle_kingside: bool,    
    pub last_capture_or_pawn_move: u8, // 50 move no fun thing happen boring game rule
    pub table_states_since_last_capture_or_pawn_move: Vec<BitBoard>,
    pub en_passant_possible: bool,
    pub white_timer: Duration,
    pub black_timer: Duration,
    pub turn_counter: u16,
    pub white_in_check: bool,
    pub black_in_check: bool,
    pub white_pieces: PieceSet,
    pub black_pieces: PieceSet,
    pub clock : std::time::Instant,
    pub timer_increment : std::time::Duration,
    pub mode: GameMode,
    pub game_over: bool,
    //fide rules set time to 50minutes after 40 moves etc... pub move_count_time_added: ((u8, Duration), (u8, Duration))
    //reversable table state check
}

#[derive(Copy, Clone, Debug)]
pub enum GameMode {
    Default,
    Blitz,
    Rapid,
    Daily,
}


impl gameloop::Handler for GameState {
    // fn handle(&self, event: gameloop::Event, payload: gameloop::Payload) {}

    fn handle_mut(&mut self, event: gameloop::Event, payload: gameloop::Payload) {
        let translation = parse_coordinates_from_payload(payload);
        let valid_move = match self.player_turn {
            1 => {
                &self.move_list.white.iter().any(|elem| *elem == translation)
            },
            2 => {
                &self.move_list.black.iter().any(|elem| *elem == translation)
            },
            _ => panic!("Player_turn wrong"),
        };
        if *valid_move {
            self.update_chess_clock();
            take_turn(self, translation);
        } else {
            println!("Not in move list")
        }
        
    }
}
impl GameState {
    pub fn new() -> Self {
        GameState {
            board: generate_start_board(),
            move_list: PlayerValidMoves{ black: MoveList::new(), white: MoveList::new()},
            last_move: None,
            player_turn: 1,     //when white takes turn add 1 when black takes turn -1
            white_can_castle_queenside: true,
            black_can_castle_queenside: true,
            white_can_castle_kingside: true, 
            black_can_castle_kingside: true,
            last_capture_or_pawn_move: 0,
            table_states_since_last_capture_or_pawn_move: vec![boardrep_to_bitboard(&generate_start_board())],
            en_passant_possible: false, //detects if en_passant_possible from last move
            white_timer: Duration::from_secs(1800), 
            black_timer: Duration::from_secs(1800),
            turn_counter: 0,
            white_in_check: false,
            black_in_check: false,
            white_pieces: PieceSet::new(),
            black_pieces: PieceSet::new(),
            clock: std::time::Instant::now(),
            timer_increment: Duration::from_secs(30),
            mode: GameMode::Default,
            game_over: false,
        }       
    }
    // allow people to choose mode, blitz/default, can add more later.
    pub fn blitz_mode(&mut self) {
        self.white_timer = Duration::from_secs(300);
        self.black_timer = Duration::from_secs(300);
        self.timer_increment = Duration:: from_secs(1);
        self.mode = GameMode::Blitz;
    }

    pub fn reset (&mut self) {
        let mode = self.mode;
        *self = GameState::new();
        if let GameMode::Blitz = mode {
            self.blitz_mode();
        } 
        self.white_timer = self.white_timer + self.timer_increment;
        get_legal_move_list(self);
    }

    pub fn update_chess_clock(&mut self) {
        match self.player_turn {
            1 => {
                self.white_timer = self.white_timer.saturating_sub(self.clock.elapsed());
            },
            2 => {
                self.black_timer = self.black_timer.saturating_sub(self.clock.elapsed()); 
            },
            _=> panic!("Clock updating detected no player turn")
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      
        }
        
        self.clock = std::time::Instant::now();
        
    }
    
    pub fn take_turn(&mut self) {
        if self.player_turn == 1 {
            self.player_turn += 1;
        } else { 
            self.player_turn -= 1;
        }
    }

}

#[derive(Debug, Clone, Copy)]
pub struct PieceSet {
    pawn: u8,
    rook: u8,
    knight: u8,
    bishop: u8,
    queen: u8,
}

impl PieceSet {
    pub fn new() -> Self {
        PieceSet { pawn: 8, rook: 2, knight: 2, bishop: 2, queen: 1}
    }
}

impl Index<usize> for PieceSet {
    type Output = u8;
    
    fn index (&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.pawn,
            1 => &self.rook,
            2 => &self.knight,
            3 => &self.bishop,
            4 => &self.queen,
            n => panic!("Invalid index: {} for PieceSet", n)
        }
    }
}

impl IndexMut<usize> for PieceSet {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.pawn,
            1 => &mut self.rook,
            2 => &mut self.knight,
            3 => &mut self.bishop,
            4 => &mut self.queen,
            n => panic!("Invalid index: {} for PieceSet", n)
        }
    }
}


fn generate_start_board() -> BoardRep {
    let piece_type = vec![
        ROOK,  KNIGHT, BISHOP, QUEEN, KING,  BISHOP, KNIGHT, ROOK,
        PAWN,  PAWN,   PAWN,   PAWN,  PAWN,  PAWN,   PAWN,   PAWN,
        EMPTY, EMPTY,  EMPTY,  EMPTY, EMPTY, EMPTY,  EMPTY,  EMPTY,
        EMPTY, EMPTY,  EMPTY,  EMPTY, EMPTY, EMPTY,  EMPTY,  EMPTY,
        EMPTY, EMPTY,  EMPTY,  EMPTY, EMPTY, EMPTY,  EMPTY,  EMPTY,
        EMPTY, EMPTY,  EMPTY,  EMPTY, EMPTY, EMPTY,  EMPTY,  EMPTY,
        PAWN,  PAWN,   PAWN,   PAWN,  PAWN,  PAWN,   PAWN,   PAWN,
        ROOK,  KNIGHT, BISHOP, QUEEN, KING,  BISHOP, KNIGHT, ROOK,
    ];
    let piece_colour = vec![
        White, White, White, White, White, White, White, White,
        White, White, White, White, White, White, White, White,
        Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
        Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
        Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
        Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
        Black, Black, Black, Black, Black, Black, Black, Black,
        Black, Black, Black, Black, Black, Black, Black, Black];

    (piece_type, piece_colour)    
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coordinates {
    pub x: usize,
    pub y: usize,
}

impl Coordinates {
    pub fn does_move_run_off_side(self, coord_2: Self) -> Result<Self, RunOffError> {
        if self.y == coord_2.y {
            return Ok(self);
        } else {
            Err(RunOffError)
        }
    }
}
#[derive(Debug, Clone)]
pub struct RunOffError;
impl std::fmt::Display for RunOffError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Ran off side of board")
    }
}

pub struct CoordDiff {
    pub x: i8,
    pub y: i8,
}


impl std::ops::Sub for Coordinates {
    type Output = CoordDiff;

    fn sub(self, other: Self) -> Self::Output {
        CoordDiff {
                x: self.x as i8 - other.x as i8,
                y: self.y as i8 - other.y as i8,
        }
    }
}

impl From<usize> for Coordinates {
    fn from(index: usize) -> Self {
        let column: usize = index % 8;
        let row: usize = index / 8;
        return Coordinates { x: column, y: row };
    }
}

impl From<Coordinates> for usize {
    fn from(coords: Coordinates) -> Self {
        return coords.y * 8 + coords.x;
    }
}


#[derive(Debug, Clone)]
pub struct PlayerValidMoves {
    pub white: MoveList,
    pub black: MoveList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceColour {
    Black,
    White,
    Empty,
}

#[derive(Debug, Clone)]
pub enum PieceType {
    King(King),
    Queen(Queen),
    Pawn(Pawn),
    Bishop(Bishop),
    Knight(Knight),
    Rook(Rook),
    Empty,
}

#[derive(Debug, Clone)]
pub enum MoveDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl MoveDirection {
    fn get_filter_closure(&self) -> impl FnMut(&Move) -> bool {
        match self {
            MoveDirection::North => {
                |element: &Move| {
                    let difference = element.1 - element.0;
                    difference.x == 0 && difference.y < 0
                }
            },
            MoveDirection::NorthEast => {
                |element: &Move| {
                    let difference = element.1 - element.0;
                    difference.x > 0  && difference.y < 0
                 }
            },
            MoveDirection::East => 
                |element: &Move| {
                    //let difference = get_index(element.1) as i8 - get_index(element.0) as i8;
                    let difference = element.1 - element.0;
                    difference.x > 0 && difference.y == 0
            },
            MoveDirection::SouthEast => { 
                |element: &Move| {
                    let difference = element.1 - element.0;
                    difference.x  > 0 && difference.y > 0
                }
            },
            MoveDirection::South => {
                |element: &Move| {
                    let difference = element.1 - element.0;
                    difference.x == 0 && difference.y > 0
                }
                
            },
            MoveDirection::SouthWest => {
                |element: &Move| {
                    let difference = element.1 - element.0;
                    difference.x < 0 && difference.y > 0
                }
            },
            MoveDirection::West => {
                |element: &Move| {
                    let difference = element.1 - element.0;
                    difference.x <0 && difference.y == 0
                 }
            },
            MoveDirection::NorthWest => {
                |element: &Move| {
                    let difference = element.1 - element.0;
                    difference.x < 0 && difference.y < 0
                }
            },
        }
    }
    fn get_sort_closure(&self) -> impl FnMut(&Move, &Move) -> std::cmp::Ordering {
        match self {
            MoveDirection::North => {
                |a: &Move, b: &Move| {
                    b.1.y.cmp(&a.1.y)
                }
            },
            MoveDirection::NorthEast => {
                |a: &Move, b: &Move| {
                    a.1.x.cmp(&b.1.x)
                }
            },
            MoveDirection::East => {
                |a: &Move, b: &Move| {
                    a.1.x.cmp(&b.1.x)
                }
            },
            MoveDirection::SouthEast => {
                |a: &Move, b: &Move| {
                    a.1.x.cmp(&b.1.x)
                }
            },
            MoveDirection::South => {
                |a: &Move, b: &Move| {
                    a.1.y.cmp(&b.1.y)
                }
            },
            MoveDirection::SouthWest => {
                |a: &Move ,b: &Move| {
                    b.1.x.cmp(&a.1.x)
              }
            },
            MoveDirection::West => {
                |a: &Move, b: &Move| {
                    b.1.x.cmp(&a.1.x)
                }
        },
            MoveDirection::NorthWest => {
                |a: &Move,b: &Move| {
                    b.1.x.cmp(&a.1.x)
                }
            },
        }
    }
}


#[derive(Debug, Clone)]
pub struct Pawn {
    //can move either -9, -8, -7, 7, 8, 9 and only 1 step, also pawn can move -17, -16, -15, 15, 16, 17 for first move
    
}

impl Pawn { 
    //based on colour either moves all south or north flavoured directions, based on blocking only, can move digonally 
    // generate all lateral and diagonal moves, filter out north/south based on color, filter out east/west
    // truncate list to 1 space in North/South
    // allow diagonal moves only if collision
    // check for double move by y coordinate based on color
    //
    fn get_valid_moves(board: &BoardRep, origin: Coordinates) -> MoveList {
        return Pawn::pawn_specific_moves(origin, board);
    }
     
    // if pawn is black and origin.y is 6 truncate vertical move to 2
    //else if pawn is white and origin.y is 1 truncate vertical move to 2
    // if diagonally blocked by other colour allow diagonal move of 1
    pub fn pawn_specific_moves(origin: Coordinates, board: &BoardRep) -> MoveList {
    
        let mut move_list: MoveList = Vec::new();
        let mut ultimate_move_list: MoveList = Vec::new();
        let mut lateral_moves = generate_lateral_moves(origin, board);
        let mut diagonal_moves = generate_diagonal_moves(origin, board);
        
        move_list.append(&mut lateral_moves);
        move_list.append(&mut diagonal_moves);
        
        match board.1[usize::from(origin)] {
            PieceColour::White => {
                let directions: Vec<MoveDirection> = vec![MoveDirection::South, MoveDirection::SouthEast, MoveDirection::SouthWest];
                for direction in directions {
                    let mut directional_move_list = separate_direction_from_movelist(&move_list, direction.clone());
                    directional_move_list = match direction {
                        MoveDirection::SouthEast | MoveDirection::SouthWest => {
                            directional_move_list
                            .into_iter()
                            .enumerate()
                            .filter(|(i,_)| *i == 0 as usize)
                            .map(|element| element.1)
                            .collect::<MoveList>()
                        }, 
                        MoveDirection::South => {
                            if origin.y == 1 {
                                directional_move_list
                                .into_iter()
                                .enumerate()
                                .filter(|(i,_)| *i <= 1 as usize)
                                .map(|element| element.1)
                                .collect::<MoveList>()
                            } else {
                                directional_move_list
                                .into_iter()
                                .enumerate()
                                .filter(|(i,_)| *i == 0 as usize)
                                .map(|element| element.1)
                                .collect::<MoveList>()
                        }
                    },
                    _ => panic!("White pawn moving North"),
                };
                
                let mut trunc = Pawn::pawn_collision(directional_move_list, direction, board);
                ultimate_move_list.append(&mut trunc);
                }
            },
            PieceColour::Black => {
                let directions: Vec<MoveDirection> = vec![MoveDirection::North, MoveDirection::NorthEast, MoveDirection::NorthWest];
                for direction in directions {
                    let mut directional_move_list = separate_direction_from_movelist(&move_list, direction.clone());
                    directional_move_list = match direction {
                        MoveDirection::NorthEast | MoveDirection::NorthWest => {
                            directional_move_list
                            .into_iter()
                            .enumerate()
                            .filter(|(i,_)| *i == 0 as usize)
                            .map(|element| element.1)
                            .collect::<MoveList>()
                        }, 
                        MoveDirection::North => {
                            if origin.y == 6 {
                                directional_move_list
                                .into_iter()
                                .enumerate()
                                .filter(|(i,_)| *i <= 1 as usize)
                                .map(|element| element.1)
                                .collect::<MoveList>()
                            } else {
                                directional_move_list
                                .into_iter()
                                .enumerate()
                                .filter(|(i,_)| *i == 0 as usize)
                                .map(|element| element.1)
                                .collect::<MoveList>()
                            }
                        },
                        _ => panic!("Black pawn moving South"),
                    };
                    
                    let mut trunc = Pawn::pawn_collision(directional_move_list, direction, board);
                    ultimate_move_list.append(&mut trunc);
                }
            },
            _ => panic!("Pawn doesn't have colour"),
        }
        return ultimate_move_list;
    }
    
    pub fn pawn_collision(moves: MoveList, direction: MoveDirection, board: &BoardRep) -> MoveList {
        let mut move_list: MoveList = Vec::new();
        let color_board = &board.1;
        //if vertical blocks on all collision
        //if diagonal blocks
        for element in moves {
            let destination = element.1;
            match direction {
                MoveDirection::North | MoveDirection::South => {
                    let(occupied, _) = is_square_occupied(element, board);
                    if occupied {
                        break;
                    }
                    move_list.push(element);
                },
                MoveDirection::NorthEast
                | MoveDirection::NorthWest
                | MoveDirection::SouthEast
                | MoveDirection::SouthWest => {
                    let(occupied, is_same_colour) = is_square_occupied(element, board);
                    if (color_board[usize::from(destination)] == PieceColour::Empty) || (occupied && is_same_colour) {
                        break;
                    }
                    move_list.push(element);
                },
                _ => panic!("Invalid move direction"),
            }
        }
        return move_list;
    }
        
    //pawn en passant
    pub fn en_passant(state: &mut GameState, translation: Move) -> (Option<usize>, Option<usize>, Option<usize>) {
        //because the last move was a pawn move and the vector of table states has been cleared as a result

        let premove_board = bitboard_to_boardrep(&state.table_states_since_last_capture_or_pawn_move[0]);
        let destination = translation.1;
        let left_destination = Coordinates::from(usize::from(translation.1) - 1);
        let right_destination = Coordinates::from(usize::from(translation.1) + 1);
        let origin_index = usize::from(translation.0);
        let destination_index = usize::from(translation.1);
        let origin_piece_type = premove_board.0[origin_index] as u8;
        let moving_colour = premove_board.1[origin_index];
        let runs_off_left = destination.does_move_run_off_side(left_destination);
        let runs_off_right = destination.does_move_run_off_side(right_destination);
        let left_piece_type = (state.board.0[destination_index - 1]) as u8;
        let right_piece_type = (state.board.0[destination_index + 1]) as u8;
        let left_pawn_colour = state.board.1[destination_index - 1];
        let right_pawn_colour = state.board.1[destination_index + 1];
        

        if origin_piece_type != PAWN
        || ((translation.0.y != 1 && translation.1.y != 3) && (translation.0.y != 6 && translation.1.y != 4))
        {
            state.en_passant_possible = false;
            return (None, None, None);
        }

        let mut left_is_opposite_colour_pawn: Option<usize> = None;
        let mut right_is_opposite_colour_pawn: Option<usize> = None;


        if runs_off_left.is_ok() && left_piece_type == PAWN && moving_colour != left_pawn_colour {
            state.en_passant_possible = true;
            left_is_opposite_colour_pawn = Some(destination_index - 1);
        }

        if runs_off_right.is_ok() && right_piece_type == PAWN && moving_colour != right_pawn_colour {
            state.en_passant_possible = true;
            right_is_opposite_colour_pawn = Some(destination_index + 1);
        }

        let capturable_square = match moving_colour {
            PieceColour::Black => destination_index + 8,
            PieceColour::White => destination_index - 8,
            _=> panic!("En passant pawn colour Empty"),
        };
        return (left_is_opposite_colour_pawn, right_is_opposite_colour_pawn, Some(capturable_square))
        
        //actually write the maneuver
    }

    pub fn append_en_passant_moves(state: &mut GameState, move_list: PlayerValidMoves, translation: Move) -> PlayerValidMoves {
        let mut output_move_list = PlayerValidMoves {
            black: move_list.black,
            white: move_list.white,
        };

        // let premove_board = &state.table_states_since_last_capture_or_pawn_move[0];
        let (option_left_pawn_index, option_right_pawn_index, destination) = Pawn::en_passant(state, translation);
        
        if state.en_passant_possible == true && option_left_pawn_index.is_some() {
            let left_pawn_index = option_left_pawn_index.unwrap();
            match state.board.1[left_pawn_index] {
                White => {
                    output_move_list.white.push((Coordinates::from(destination.unwrap()), Coordinates::from(left_pawn_index)));
                },
                Black => {
                    output_move_list.black.push((Coordinates::from(destination.unwrap()), Coordinates::from(left_pawn_index)));
                },
                _ => panic!("En passant returning empty colour"),
            }
        }
        if state.en_passant_possible == true && option_right_pawn_index.is_some() {
            let right_pawn_index = option_right_pawn_index.unwrap();
            match state.board.1[right_pawn_index] {
                White => {
                    output_move_list.white.push((Coordinates::from(destination.unwrap()), Coordinates::from(right_pawn_index)));
                },
                Black => {
                    output_move_list.black.push((Coordinates::from(destination.unwrap()), Coordinates::from(right_pawn_index)));
                },
                _ => panic!("En passant returning empty colour")
            }
        }
        return output_move_list;
    }

    pub fn pawn_promotion (destination: Coordinates, state: &mut GameState) {
        //if pawn is on y of 0 or y of 7 after it moves, it promotes to one of the options
        let promotion_choice = Pawn::get_promotion_choice();
        if state.player_turn == 1 {
            match promotion_choice {
                KNIGHT => state.white_pieces.knight += 1,
                ROOK => state.white_pieces.rook += 1,
                BISHOP => state.white_pieces.bishop += 1,
                QUEEN => state.white_pieces.queen += 1,
                _=> panic!("Invalid promotion piece choice")
            }
            
        } else {
            match promotion_choice {
                KNIGHT => state.black_pieces.knight += 1,
                ROOK => state.black_pieces.rook += 1,
                BISHOP => state.black_pieces.bishop += 1,
                QUEEN => state.black_pieces.queen += 1,
                _ => panic!("Invalid promotion piece choice")
            }
        }
        state.board.0[usize::from(destination)] = promotion_choice;
    }
    
    pub fn get_promotion_choice() -> u8 {
        let promotion_possibilities = [ROOK, KNIGHT, BISHOP, QUEEN];
        let mut input = String::new();
        println!("Choose piece to promote to");
        loop {
            std::io::stdin()
                .read_line(&mut input)
                .expect("Error reading input");

            let choice: u8 = match input.to_lowercase().trim() {
                "rook" => ROOK,
                "knight" => KNIGHT,
                "bishop" => BISHOP,
                "queen" => QUEEN,
                _ => {
                    println!("Invalid piecetype");
                    0
                },
            };
            if promotion_possibilities.contains(&choice) {
                break choice;
            } else {
                println!("Not a valid pawn promotion choice");
                input.clear();
            }
        }
        
    }
}


#[derive(Debug, Clone)]
pub struct Rook {
    //can move -8, -1, 1 8
  } 

impl Rook {
    pub fn get_valid_moves (board: &BoardRep, origin: Coordinates) -> MoveList{
        let directions: Vec<MoveDirection> = vec![MoveDirection::North, MoveDirection::East, MoveDirection::South, MoveDirection::West];
        return piece_specific_moves(directions, origin, board);
    }
}

#[derive(Debug, Clone)]
pub struct Knight {
    // -17, -15, -10 -6, 6, 10, 15, 17
    //column 7 cant go -6 or +10
    //column 8 cant go -15, -6
    //index cant go below 0 or above 63
    //if adding -17, -10, 6, 15 index cant be divisible by 7
    //if adding -15, -6 , 10, 17 index cant be divisible by 8  
}

impl Knight {
    pub fn get_valid_moves(board: &BoardRep, origin: Coordinates) -> MoveList {
        let mut move_list: MoveList = Vec::new();
        let square = usize::from(origin);
                //check above rules 
        let math_moves = [-17, -15, -10, -6, 6, 10, 15, 17];
        let potential_moves = math_moves.map(|element| square as i8 + element );
                //indices even cant be divisible by 7
        let valid_moves = potential_moves.iter().enumerate().filter(|(index, element)| {
        if index % 2 == 0 && **element as i8 % 8 == 7{
            false
        } else if index % 2 == 1 && **element as i8 % 8 == 0 {
            false
        } else if **element > 63 || **element < 0 {
            false
        } else {
            true
        }
        });
                //yay a list of acceptable moves for the knight
        for (_, element) in valid_moves {
            let destination = Coordinates::from(*element as usize);
                    //if there is a piece there same colour continue loop, otherwise push to movelist
            if is_square_occupied((origin, destination), board).1 {
                continue;
            } else{
                move_list.push((origin, destination));
            }
        }
            
     return move_list;
    }
}

#[derive(Debug, Clone)]
pub struct Bishop {
    //Can move -9 - 7 +7 +9
}

impl Bishop {
    pub fn get_valid_moves(board: &BoardRep, origin: Coordinates) -> MoveList {
    let directions: Vec<MoveDirection> = vec![MoveDirection::NorthEast, MoveDirection::NorthWest, MoveDirection::SouthEast, MoveDirection::SouthWest];
    return piece_specific_moves(directions, origin, board);
            
    //sort list into difference in index order, then check collisions, if there is a collision in a multiple of 7 or 9, -7 or -9 excise the list beyond that point
    //break list into 4 directions of travel
    // if difference b/w origin and destination is greater than 0 and multiple of 7 its SE   
    //if collision detected with same colour stop before if detected with different colour stop at
    }
}

#[derive(Debug, Clone)]
pub struct Queen {
    // queen is just a rook and bishop
}

impl Queen {
    pub fn get_valid_moves (board: &BoardRep, origin: Coordinates) -> MoveList{
       let directions: Vec<MoveDirection> = vec![MoveDirection::North, MoveDirection::South, MoveDirection::East, MoveDirection::West, MoveDirection::NorthEast, MoveDirection::NorthWest, MoveDirection::SouthEast, MoveDirection::SouthWest];
       return piece_specific_moves(directions, origin, board);
    }
}

#[derive(Debug, Clone)]
pub struct King {
    //can move -9, -8, -7, -1, 1, 7, 8, 9 can also castle and some bullshit, also has to chekc if check/mate 
}
//generate movelist every turn to detect Check/Checkmate status, and check if reqeusted move is possible

impl King {
    //redo everything
    pub fn get_valid_moves(board: &BoardRep, origin: Coordinates) -> MoveList {
        let mut move_list: MoveList = Vec::new();
        let mut ultimate_move_list: MoveList = Vec::new();
        let mut lateral_moves = generate_lateral_moves(origin, board);
        let mut diagonal_moves = generate_diagonal_moves(origin, board);
        
        move_list.append(&mut lateral_moves);
        move_list.append(&mut diagonal_moves);

        
        let directions: Vec<MoveDirection> = vec![MoveDirection::North, MoveDirection::South, MoveDirection::East, MoveDirection::West, MoveDirection::NorthEast, MoveDirection::NorthWest, MoveDirection::SouthEast, MoveDirection::SouthWest];
            for direction in directions {
                let mut directional_move_list = separate_direction_from_movelist(&move_list, direction.clone());

                directional_move_list = directional_move_list
                    .into_iter()
                    .enumerate()
                    .filter(|(i,_)| *i == 0 as usize)
                    .map(|element| element.1)
                    .collect::<MoveList>();

                directional_move_list = slice_valid_moves_at_collision(directional_move_list, board);
                ultimate_move_list.append(&mut directional_move_list);
              
            }
        return ultimate_move_list;
    }
    //needs to check if its in the opposing colour movelist or will be for a next turn based on piece movement

    pub fn check_checker(state: &mut GameState, move_list: MoveList) -> bool {
        // if king position is in movelist, player of king colour is in check.s
        let first_move = move_list[0];
        let first_move_origin = first_move.0;
        let colour_of_moves = state.board.1[usize::from(first_move_origin)];
        

        // loop through board look for king and check if check
        for i in 0..state.board.0.len() {
            let piece = state.board.0[i];
            let king_colour = state.board.1[i];
            if piece == KING && king_colour != colour_of_moves {
                let origin = Coordinates::from(i);
                let mut valid_move = move_list.iter();
                let is_in_check = valid_move.any(|element| element.1 == origin);
                if is_in_check {
                    match king_colour {
                        Black => state.black_in_check = true,
                        White => state.white_in_check = true,
                        _ => panic!("Empty colour king in check")
                    }
                } else {
                    match king_colour {
                        Black => state.black_in_check = false,
                        White => state.white_in_check = false,
                        _ => panic!("Empty colour king in check")
                    }
                }
                return is_in_check;
            } else{
                continue;
            }
        }
        panic!("Missing King");
    }

    pub fn can_castle_kingside(move_list: &PlayerValidMoves, state: &GameState) -> (bool, bool) {
        let uncheckable_squares_white = [Coordinates{x:4,y:0}, Coordinates{x:5,y:0}, Coordinates{x:6,y:0}];
        let uncheckable_squares_black = [Coordinates{x:4,y:7}, Coordinates{x:5,y:7}, Coordinates{x:6,y:7}];
        let empty_squares_white = [Coordinates{x:5,y:0}, Coordinates{x:6,y:0}]; 
        let empty_squares_black = [Coordinates{x:5,y:7}, Coordinates{x:6,y:7}]; 

        let white_moves = &move_list.white;
        let black_moves = &move_list.black;
        let mut white_castle = true;
        let mut black_castle = true;
        //if uncheckable squares are in end position of opposite colour movelist, you cannot castle
        //if coordinates of x1 x2 or x3 are occupied, cannot castle
        for translation in white_moves {
            if uncheckable_squares_black.into_iter().any(|square| square == translation.1)
            || empty_squares_black.into_iter().any(|square| state.board.1[usize::from(square)] != PieceColour::Empty)
            || state.black_can_castle_queenside == false
            || (state.board.0[63] != ROOK || state.board.1[63] != PieceColour::Black)
            {
                black_castle = false;
            }
        }

        for translation in black_moves {
            if uncheckable_squares_white.into_iter().any(|square| square == translation.1)
            || empty_squares_white.into_iter().any(|square| state.board.1[usize::from(square)] != PieceColour::Empty)
            || state.white_can_castle_queenside == false
            || (state.board.0[7] != ROOK || state.board.1[7] != PieceColour::White)
            {
                white_castle = false;
            }
        }
        return (white_castle, black_castle)
    }
    
    pub fn can_castle_queenside(move_list: &PlayerValidMoves, state: &GameState) -> (bool, bool) {
        let uncheckable_squares_white = [Coordinates{x:2,y:0}, Coordinates{x:3,y:0}, Coordinates{x:4,y:0}];
        let uncheckable_squares_black = [Coordinates{x:2,y:7}, Coordinates{x:3,y:7}, Coordinates{x:4,y:7}];
        let empty_squares_white = [Coordinates{x:2,y:0}, Coordinates{x:3,y:0}]; 
        let empty_squares_black = [Coordinates{x:2,y:7}, Coordinates{x:3,y:7}]; 
        
        
        let white_moves = &move_list.white;
        let black_moves = &move_list.black;
        let mut white_castle = true;
        let mut black_castle = true;
        //if uncheckable squares are in end position of opposite colour movelist, you cannot castle
        //if coordinates of x1 x2 or x3 are occupied, cannot castle
        for translation in white_moves {
            if uncheckable_squares_black.into_iter().any(|square| square == translation.1)
            || empty_squares_black.into_iter().any(|square| state.board.1[usize::from(square)] != PieceColour::Empty)
            || state.black_can_castle_queenside == false
            ||(state.board.0[56] != ROOK || state.board.1[56] != PieceColour::Black)
            {
                black_castle = false;
            }
        }

        for translation in black_moves {
            if uncheckable_squares_white.into_iter().any(|square| square == translation.1)
            || empty_squares_white.into_iter().any(|square| state.board.1[usize::from(square)] != PieceColour::Empty)
            || state.white_can_castle_queenside == false
            || (state.board.0[0] != ROOK || state.board.1[0] != PieceColour::White)
            {
                white_castle = false;
            }
        }

        return (white_castle, black_castle)
    }

    pub fn append_castle_moves(move_list: PlayerValidMoves, state: &GameState) -> PlayerValidMoves {
        let mut output_move_list = PlayerValidMoves {
            white: move_list.white,
            black: move_list.black,
        };
        //if white can queenside  castle append that move
        let white_queenside_castle_move = (Coordinates {x:4, y: 0}, Coordinates{x: 2,y: 0});
        let black_queenside_castle_move = (Coordinates {x:4, y: 7}, Coordinates{x: 2,y: 7});
        let white_kingside_castle_move = (Coordinates {x:4, y: 0}, Coordinates{x: 6, y: 0});
        let black_kingside_castle_move = (Coordinates {x:4, y: 7}, Coordinates{x: 6, y: 7});
        
        let kingside = King::can_castle_kingside(&output_move_list, state);
        let queenside = King::can_castle_queenside(&output_move_list, state);
        
        if state.white_can_castle_kingside && kingside.0 {
            output_move_list.white.push(white_kingside_castle_move)
        }
        if state.black_can_castle_kingside && kingside.1 {
            output_move_list.black.push(black_kingside_castle_move)
        }
        if state.white_can_castle_queenside && queenside.0{
            output_move_list.white.push(white_queenside_castle_move)
        }
        if state.black_can_castle_queenside && queenside.1{
            output_move_list.black.push(black_queenside_castle_move)
        }

        return output_move_list;
    }
    
    // call this function when move selected and board changed
    pub fn is_move_a_castle(translation: Move, board: &BoardRep) -> bool {
        return board.0[usize::from(translation.0)] == KING
        && (translation.1.x as i8).abs_diff(translation.0.x as i8) > 1;
    }
    pub fn check_to_disable_castling(state: &mut GameState) {
        if state.board.0[4] != ROOK || state.board.1[7] != White {
            state.white_can_castle_kingside = false
        }
        if state.board.0[60] != ROOK || state.board.1[63] != Black {
            state.black_can_castle_kingside = false
        }
        if state.board.0[4] != ROOK || state.board.1[0] != White {
            state.white_can_castle_queenside = false
        }
        if state.board.0[60] != ROOK || state.board.1[56] != Black {
            state.black_can_castle_queenside = false
        }
    }
    // pub fn its_1000_years_too_early_for_you_to_fight_me_kid(state: &GameState) {
    //     //king teleports behind a pawn and kills it
    // }
}
    
pub type MoveList = Vec<(Coordinates, Coordinates)>;

pub fn is_square_occupied(movement: Move, board: &BoardRep) -> (bool, bool) {
    let board = &board.1;
    let target = &board[usize::from(movement.1)];
    let is_occupied = match target {
        PieceColour::Empty => false,
        _ => true,
    };
    let is_same_colour = board[usize::from(movement.0)] == board[usize::from(movement.1)];
    return (is_occupied, is_same_colour);
}

pub fn slice_valid_moves_at_collision(list: MoveList, board: &BoardRep) -> MoveList {
    let mut move_list: MoveList = Vec::new();
    for index in 0..list.len() {
        let square = list[index];
        let (occupied, same_colour) = is_square_occupied(square, board);
        if occupied && same_colour {
            //index -1 is last element in list
            break;
        } else if occupied {
            //index is last element in list
            move_list.push(list[index]);
            break;
        } else {
            move_list.push(list[index])
        }
    } 
    return move_list;
}

pub fn generate_diagonal_moves(origin: Coordinates, board: &BoardRep) -> MoveList {
    let mut move_list: MoveList = Vec::new();
    let piece_board = &board.0;
    for square_index in 0..piece_board.len() {
        
        let destination = Coordinates::from(square_index);
        if destination.x.abs_diff(origin.x) == destination.y.abs_diff(origin.y) {
            move_list.push((origin, destination));
        }                            
    }
    return move_list;
}

pub fn generate_lateral_moves(origin: Coordinates, board: &BoardRep) -> MoveList {
    let mut move_list: MoveList = Vec::new();
    let piece_board = &board.0;
    for square_index in 0..piece_board.len() {
        let destination = Coordinates::from(square_index);
        //change un either x or y allowed change in both not allowed
        if (origin.x == destination.x && origin.y != destination.y)
        || (origin.x != destination.x && origin.y == destination.y) {
            move_list.push((origin, destination))
        } 
    }
    return move_list;
}

type Move = (Coordinates, Coordinates);

pub fn separate_direction_from_movelist(list: &MoveList, direction: MoveDirection) -> MoveList {
    let mut moves = list.clone()
    .into_iter()
    .filter(direction.get_filter_closure())
    .collect::<Vec<_>>();
    moves.sort_unstable_by(direction.get_sort_closure());
    return moves;
}

pub fn piece_specific_moves(directions: Vec<MoveDirection>, origin: Coordinates, board: &BoardRep) -> MoveList {
    let mut move_list: MoveList = Vec::new();
    let mut ultimate_move_list: MoveList = Vec::new();
    let is_diagonal = directions.iter().any(|element| {
        match element {
            MoveDirection::NorthEast => true,
            MoveDirection::NorthWest => true,
            MoveDirection::SouthEast => true,
            MoveDirection::SouthWest => true,
            _ => false,
        }
    });
    let is_lateral = directions.iter().any(|element| {
        match element {
            MoveDirection::North => true,
            MoveDirection::East => true,
            MoveDirection::South => true, 
            MoveDirection::West=> true,
            _ => false,
        }
    });
    
    if is_diagonal {
        move_list.append(&mut generate_diagonal_moves(origin, board))
    } 
    if is_lateral {
        move_list.append(&mut generate_lateral_moves(origin, board))
    }

    for direction in directions {
        let mut directional_moves = separate_direction_from_movelist(&move_list, direction);
        directional_moves = slice_valid_moves_at_collision(directional_moves, board);
        ultimate_move_list.append(&mut directional_moves);
    }
    
    return ultimate_move_list;
}

pub fn get_valid_moves_for_piece(board: &BoardRep) -> PlayerValidMoves {
    let mut white_move_list: MoveList = Vec::new();
    let mut black_move_list: MoveList = Vec::new();
    let piece_board = &board.0;
    let colour_board = &board.1;
    

    for i in 0..piece_board.len() {
        let piece = &piece_board[i];
        let square_color = &colour_board[i];

        match square_color {
            PieceColour::White => {
                match *piece {
                    KING => white_move_list.append(&mut King::get_valid_moves(board, Coordinates::from(i))),
                    QUEEN => white_move_list.append(&mut Queen::get_valid_moves(board, Coordinates::from(i))),
                    ROOK => white_move_list.append(&mut Rook::get_valid_moves(board, Coordinates::from(i))),
                    BISHOP => white_move_list.append(&mut Bishop::get_valid_moves(board, Coordinates::from(i))),
                    KNIGHT => white_move_list.append(&mut Knight::get_valid_moves(board, Coordinates::from(i))),
                    PAWN => white_move_list.append(&mut Pawn::get_valid_moves(board, Coordinates::from(i))),
                    _ => panic!("Invalid piece type"),
                }
            },
            PieceColour::Black => {
                match *piece {
                    KING => black_move_list.append(&mut King::get_valid_moves(board, Coordinates::from(i))),
                    QUEEN => black_move_list.append(&mut Queen::get_valid_moves(board, Coordinates::from(i))),
                    ROOK => black_move_list.append(&mut Rook::get_valid_moves(board, Coordinates::from(i))),
                    BISHOP => black_move_list.append(&mut Bishop::get_valid_moves(board, Coordinates::from(i))),
                    KNIGHT => black_move_list.append(&mut Knight::get_valid_moves(board, Coordinates::from(i))),
                    PAWN => black_move_list.append(&mut Pawn::get_valid_moves(board, Coordinates::from(i))),
                    _ => panic!("Invalid piece type"),
                }
            },
            PieceColour::Empty => continue
        }
        
    }

    let total_moves_for_piece = PlayerValidMoves {
        white: white_move_list,
        black: black_move_list,
    };

    return total_moves_for_piece;                                                                                                                                                                                                                                                                                                                                                                                                                                   
}

pub fn make_move(board: &BoardRep, translation: Move) -> BoardRep {
    let mut piece_board = board.0.clone();
    let mut colour_board = board.1.clone();
    let origin_index = usize::from(translation.0);                                                                              
    let destination_index= usize::from(translation.1);
    let piece_type = board.0[origin_index];
    let piece_colour = board.1[origin_index];
    piece_board[origin_index] = EMPTY;
    piece_board[destination_index] = piece_type; // of piectype at move origin
    colour_board[origin_index] = Empty;
    colour_board[destination_index] = piece_colour;

    if King::is_move_a_castle(translation, board) {
        //if king's X increases kingside rook of same Y value moves -2 X
        let king_origin = usize::from(translation.0);
        let king_destination = usize::from(translation.1);
        let king_colour = board.1[king_origin];

        if king_destination as i8 - king_origin as i8 == 2 {
            piece_board[king_destination + 1] = EMPTY;
            colour_board[king_destination + 1] = Empty;
            
            piece_board[king_origin + 1] = ROOK;
            colour_board[king_origin + 1] = king_colour;
        }
        // if king's X decreases, queenside rook of same Y moves +3 X
        else {
            piece_board[king_destination - 2] = EMPTY;
            colour_board[king_destination - 2] = Empty;
            
            piece_board[king_origin - 1] = ROOK;
            colour_board[king_origin - 1] = king_colour;
        }
    } 

    return (piece_board, colour_board)
} 
//need to check if prospective moves put yourself in check, its okay to put the opponent in check but not yourself

pub fn remove_check_positions(list: MoveList, state: &mut GameState) -> MoveList {
    //takes in movelist makes move for every move of its colour,
    // checks to see if its king is in check in any of the boards that were generated
    // it outputs a movelist without the moves that put its king in check
    let first_move = list[0];
    let origin = first_move.0;
    let move_color = state.board.1[usize::from(origin)];
    
    return list
    .into_iter()
    .filter(|translation| {
        let simulated_board = make_move(&state.board, *translation);
        let simulated_player_moves: PlayerValidMoves = get_valid_moves_for_piece(&simulated_board);
        let old_board = state.board.clone();
        state.board = simulated_board;

        match move_color  {
            PieceColour::White => {
                let out = !King::check_checker(state, simulated_player_moves.black);
                state.board = old_board;
                out
            },
            PieceColour::Black =>{
                let out = !King::check_checker(state, simulated_player_moves.white);
                state.board = old_board;
                out
            },
            _ => panic!("Move does not match a square with a coloured piece on it"),
        }
    })
    .collect::<MoveList>();
}

pub fn get_legal_move_list(state: &mut GameState) {
    let move_list = get_valid_moves_for_piece(&state.board);
    let white_move_list = remove_check_positions(move_list.white, state);
    let black_move_list = remove_check_positions(move_list.black, state);

    let mut output_move_list = PlayerValidMoves {white: white_move_list, black: black_move_list};
    
    output_move_list = King::append_castle_moves(output_move_list, state);

    if state.last_move.is_some(){
        output_move_list = Pawn::append_en_passant_moves(state, output_move_list, state.last_move.unwrap());
    }

    state.move_list = output_move_list;
}

pub fn take_turn(state: &mut GameState, translation: Move) {
    let premove_board = state.board.clone();
    // let move_colour = state.board.1[usize::from(translation.0)];
    state.board = make_move(&state.board, translation);

    let captured_piece = premove_board.0[usize::from(translation.1)];
    let captured_colour = premove_board.1[usize::from(translation.1)];
    if captured_colour == White {
        match captured_piece {
            PAWN => state.white_pieces.pawn -= 1,
            ROOK => state.white_pieces.rook -= 1,
            KNIGHT => state.white_pieces.knight -= 1,
            BISHOP =>state.white_pieces.bishop -= 1,
            QUEEN =>state.white_pieces.queen -= 1,
            EMPTY => {},
            _=> panic!("No Piecetype for given square"),
                
        }    
    } else if captured_colour == Black {
        match captured_piece {
            PAWN => state.black_pieces.pawn -=1,
            ROOK => state.black_pieces.rook -= 1,
            KNIGHT => state.black_pieces.knight -= 1,
            BISHOP => state.black_pieces.bishop -= 1,
            QUEEN => state.black_pieces.queen -= 1,           
            EMPTY => {},
            _=> panic!("No Piecetype for given square"),
        }
    }
    


    // pawn promotion
    let is_pawn = premove_board.0[usize::from(translation.0)] == PAWN;
    if is_pawn && (translation.1.y == 0 || translation.1.y == 7) {
        Pawn::pawn_promotion(translation.1, state);
    }

    state.last_move = Some(translation);
    
    //table states updates
    if premove_board.0[usize::from(translation.0)] == PAWN || premove_board.0[usize::from(translation.1)] != EMPTY {
        state.last_capture_or_pawn_move = 0;
        
        state.table_states_since_last_capture_or_pawn_move = vec![boardrep_to_bitboard(&state.board.clone())];
    }else {
        state.last_capture_or_pawn_move += 1;
        state.table_states_since_last_capture_or_pawn_move.push(boardrep_to_bitboard(&state.board.clone()));
    }
    

    //if king or kingside rook moves, state.colour.can kinside castle = false
    King::check_to_disable_castling(state);
    //en passant logic: black pawn on y= 6 moving to y=4, white pawn on y=4 takes y = 5 where x is +1 or -1 not between
    // white pawn y=1 moving to y=3, black pawn y=3 takes y=2 where x is either -1 or +1
    // call function

    Pawn::en_passant(state, translation);

    //check checking
    match state.player_turn {
        1 => state.white_in_check = false,
        2 => state.black_in_check = false,
        _ => panic!("Invalid player turn number"),
    }
    
    
    get_legal_move_list(state);
    
    //game over check
    if let Some(ending) = game_end(state) {
        println!("{:?}", ending);
        state.game_over = true;
    }
    
    //update turn counter
    state.turn_counter += 1;

    // add timer increment to next player
    match state.player_turn {
        1 => state.black_timer += state.timer_increment,
        2 => state.white_timer += state.timer_increment,
        _ => panic!("Player turn is not 1(white) or 2(black)"),
    }
    //change player turn
    if state.player_turn == 1 {
        state.player_turn = 2
    } else {
        state.player_turn = 1
    }
    //fantastic GUI
    println!("{:?}, player turn {:?}, White clock {:?}, Black clock {:?}, Is white in check {:?}, Is black in check {:?}", state.board, state.player_turn, state.white_timer, state.black_timer, state.white_in_check, state.black_in_check )
}

#[derive(Debug)]
pub enum GameEnd {
    Stalemate, // good
    InsufficientMaterials, // good
    FiftyMoveRuleDraw, // good 
    RepetitionDraw, 
    Checkmate(bool), // good 
    TimeOut(bool), // good
    Resignation(bool),
}

impl GameEnd {
    pub fn insufficient_materials(state: &GameState) -> (bool, bool)  {
        //pawn, rook, and queen must be 0
        //knight alone must be less than 3
        // bishop must be less than 2 or of same square colour
        let mut black_insufficient = false;
        let mut white_insufficient = false;

        if state.white_pieces.pawn > 0 || state.white_pieces.rook > 0 || state.white_pieces.queen > 0 {
            ()
        } else if state.white_pieces.bishop == 0 && state.white_pieces.knight < 3 {
            white_insufficient = true;
        } else if state.white_pieces.knight == 0 && (state.white_pieces.bishop < 2 || !bishop_can_checkmate(state, White)){
            white_insufficient = true;
        }

        if state.black_pieces.pawn > 0 || state.black_pieces.rook > 0 || state.black_pieces.queen > 0 {
            ()
        } else if state.black_pieces.bishop == 0 && state.black_pieces.knight < 3 {
            black_insufficient = true;
        } else if state.black_pieces.knight == 0 && (state.black_pieces.bishop < 2 || !bishop_can_checkmate(state, Black)){
            black_insufficient = true;
        }
        
        return (white_insufficient, black_insufficient);
    }
}
pub fn bishop_can_checkmate (state: &GameState, colour: PieceColour) -> bool {
    let mut bishop_list = vec![];

    for i in 0..state.board.0.len() {
        if state.board.0[i] == BISHOP && state.board.1[i] == colour {
            bishop_list.push(Coordinates::from(i));
        }
    }
    !bishop_list.iter().all(|bishop| checker_board(*bishop) == true ) || !bishop_list.into_iter().all(|bishop| checker_board(bishop) == false)

}

pub fn checker_board(square: Coordinates) -> bool {
    if (square.x as u8 + square.y as u8) % 2 != 0 {
        true
        //meaning white
    } else {
        false
        //meaning black
    }
}

pub fn game_end(state: &mut GameState) -> Option<GameEnd> {   
    //given player movelist is empty, game ends and given player loses.
    return if state.move_list.white.len() == 0 && state.white_in_check {
        println!("Black Wins by Checkmate");
        Some(GameEnd::Checkmate(true))
    } else if state.move_list.white.len() == 0 && !state.white_in_check {
        println!("White Draws Stalemate");
        Some(GameEnd::Stalemate)
    } else if state.move_list.black.len() == 0 && state.black_in_check {
        println!("White Wins by Checkmate");
        Some(GameEnd::Checkmate(false))
    } else if state.move_list.black.len() == 0 && !state.black_in_check {
        println!("White Draws Stalemate");
        Some(GameEnd::Stalemate)
    } else if state.last_capture_or_pawn_move >= 100 {
        println!("Draw by 50 move rule");
        Some(GameEnd::FiftyMoveRuleDraw)
    } else if game_end_by_repetition(state) {
        println!("Draw by threefold repetition");
        Some(GameEnd::RepetitionDraw)
    } else if GameEnd::insufficient_materials(state) == (true, true) {
        println!("Draw by insufficient Material");
        Some(GameEnd::InsufficientMaterials)
    } else if state.white_timer <= Duration::from_secs(0) && GameEnd::insufficient_materials(state) == (false, true) {
        println!("Draw by White Time Out and Black Insufficient Materials");
        Some(GameEnd::InsufficientMaterials)
    } else if state.black_timer <= Duration::from_secs(0) && GameEnd::insufficient_materials(state) == (true, false) {
        println!("Draw by Black Time Out and White Insufficient Materials");
        Some(GameEnd::InsufficientMaterials)
    } else if state.white_timer <= Duration::from_secs(0) {
        println!("Draw by White Time Out");
        Some(GameEnd::TimeOut(true))
    } else if state.black_timer <= Duration::from_secs(0) {
        println!("Draw by Black Time Out");
        Some(GameEnd::TimeOut(false))
    } // Resignation  
    else {
        None
    };

    // insufficient material, King, king bishop, king knight, kingknight knight, 
    // king and any number of bishops on same colour square 
    // king v king, king simple v king, king simple v king simple, king knight knight v king
    
    //if draw conditions met game end in draw
    // check and empty movelist win, if empty move list no check, win.
    // checkmate, time-out, resignation are all win lose scenarios
    //stalemate, insufficient material, 50 move rule, repitition, and agreement are all draws.
}

fn game_end_by_repetition(state: &mut GameState) -> bool {
    let board_list = &state.table_states_since_last_capture_or_pawn_move;
    if board_list.len() <= 2 {
        return false;
    }

    let last_board = &board_list[board_list.len() - 1];
    let rest_boards = &board_list[0..board_list.len() - 2];

    
    let mut counter = 0;
    for board in rest_boards {
        if board == last_board {
            counter += 1;
        }
        if counter >= 2 {
            break;
        }
    }
    return counter >= 2;
   

}

fn parse_payload_from_index(index_string: &str) -> Result<gameloop::Payload, std::num::ParseIntError> {
    let mut split = index_string.trim().split_whitespace();
    let indices = (
        split.next().unwrap_or_default().to_string(),
        split.next().unwrap_or_default().to_string()
    );

    let origin_index = indices.0.trim().parse::<u8>()?;
    let destination_index = indices.1.trim().parse::<u8>()?;

    Ok(vec![origin_index, destination_index])

}


fn parse_payload_from_coordinates(simple_coords: &str) -> Option<gameloop::Payload> {
    //input format origin to destination : (a,b) (x,y) 
    let mut split = simple_coords.split_whitespace();
    let indices = (
        split.next().unwrap_or_default().to_string(),
        split.next().unwrap_or_default().to_string()
    );

    let x_digit = indices.0.trim().chars().nth(1)?;
    let y_digit =  indices.0.trim().chars().nth(3)?;
    let origin = Coordinates { x: x_digit.to_digit(10)? as usize, y: y_digit.to_digit(10)? as usize }; 
                                             

    let x_digit = indices.1.trim().chars().nth(1)?;
    let y_digit =  indices.1.trim().chars().nth(3)?;
    let destination = Coordinates { x: x_digit.to_digit(10)? as usize, y: y_digit.to_digit(10)? as usize };

    // println!("{:?}, {:?}", origin, destination);

    Some(vec![usize::from(origin) as u8, usize::from(destination) as u8])
}

fn parse_coordinates_from_payload(payload: Payload) -> Move {
    let origin = Coordinates::from(payload[0] as usize);
    let destination = Coordinates::from(payload[1] as usize);

    (origin, destination)
}