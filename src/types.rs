
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Entity {
    pub id: u32,
    pub pos: (i32, i32),
}

pub type Entities = HashMap<u32, Entity>;
