
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Counter {
    counter: u32,
}

impl Counter {
    pub fn new() -> Self { Counter{ counter: 0 } }

    pub fn next(&mut self) -> u32 {
        let cur = self.counter;
        self.counter += 1;
        cur
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Entity {
    pub id: u32,
    pub pos: (i32, i32),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Event {
    ClientRegistered(u32),
    UpdateEntities,
    Chat(String),
}

pub type Entities = Arc<Mutex<HashMap<u32, Entity>>>;
pub type EntityCounter = Arc<Mutex<Counter>>;