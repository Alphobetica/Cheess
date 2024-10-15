use crate::gameloop::{Event, Handler, Payload};


pub struct UserInput {
  pub input: String,
}

impl Handler for UserInput {
  fn handle_mut(&mut self, _event: Event, _payload: Payload) {
    println!("Make move");
    
    let mut input = String::new();
    std::io::stdin()
      .read_line(&mut input)
      .expect("Error reading input");

    self.input = input.to_lowercase().trim().to_owned();
  }
}


// pub fn special_commands(input: &str) {}