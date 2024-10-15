#![allow(dead_code)]

use std::{collections::HashMap, sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex}, thread};
use crate::threadpool::ThreadPool;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Event {
  MoveInput,
  Surrender,
  UserInput,
}

pub type Payload = Vec<u8>;

pub trait Handler: Send + Sync {
  fn handle_mut(&mut self, event: Event, payload: Payload);
  fn handle(&self, event: Event, payload: Payload) {}
}

#[derive(Clone)]
struct Listener {
  event: Event,
  handler: Arc<Mutex<dyn Handler>>,
}

pub struct Dispatcher<'a> {
  tx: Sender<(Event, Payload)>,
  rx: Arc<Mutex<Receiver<(Event, Payload)>>>,
  handler_registry: Arc<Mutex<HashMap<Event, Vec<Arc<Mutex<dyn Handler>>>>>>,
  threadpool: Option<&'a ThreadPool>,
} 

impl<'a> Dispatcher<'a> {
  pub fn new(threadpool: &'a ThreadPool) -> Self {
    let (tx, rx) = channel();
    Dispatcher {tx, rx: Arc::new(Mutex::new(rx)), handler_registry: Arc::new(Mutex::new(HashMap::new())), threadpool: Some(threadpool)}
  }

  pub fn register_handler(&mut self, event: Event, handler: Arc<Mutex<dyn Handler>>) {
    let mut registry = self.handler_registry.lock().unwrap();
    registry.entry(event).or_insert_with(Vec::new).push(handler);
  }

  pub fn trigger_event(&self, event: Event, payload: Payload) {
    self.tx.send((event, payload)).unwrap();
  }

  pub fn start(&self) {
    if let Some(threadpool) = self.threadpool {
      let rx = self.rx.clone();
      let handler_registry = self.handler_registry.clone();
      
      //should check for new events 
      let f = move || loop {
        match rx.lock().unwrap().recv() {
          Ok((event, payload)) => {
            let handler_registry = handler_registry.lock().unwrap();
            if let Some(handler_list) = handler_registry.get(&event) {
              for handler in handler_list {
                // handler.lock().unwrap().handle(event.clone(), payload.clone());
                handler.lock().unwrap().handle_mut(event.clone(), payload.clone());
              }
            }
      
          },
          Err(e) => {
            break;
          }
        }
      };
      threadpool.execute(f);
    }
  }
}

impl<'a> Drop for Dispatcher<'a> {
  fn drop(&mut self) {
      self.threadpool = None;
      self.handler_registry.lock().unwrap().drain();
      
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  pub fn listen() {
    pub struct TestHandler;

    impl Handler for TestHandler {
      fn handle_mut(&mut self, event: Event, payload: Payload) {
          let data = String::from_utf8(payload).unwrap();
          println!("{:?} {:?}", event, data);
      }
    }
    let pool = ThreadPool::new(1).unwrap();

    let mut event_loop = Dispatcher::new(&pool);

    event_loop.register_handler(Event::MoveInput, Arc::new(Mutex::new(TestHandler)));

    event_loop.start();

    loop {
      println!("Gib input");

      let mut input = String::new();

      std::io::stdin()
        .read_line(&mut input)
        .expect("Input Error");

      let input = input.trim();

      if input == "exit" {
        break;
      }
      
      let event = Event::MoveInput;

      event_loop.trigger_event(event, input.as_bytes().to_vec());
    }
  }
}
