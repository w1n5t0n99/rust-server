pub mod types;

use std::collections::HashMap;
use std::fmt::Debug;
use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock};
use std::io;
use std::io::prelude::*;

fn main() {
    println!("Test input");

    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        println!("Output: {}", line);

        if line.to_lowercase() == "exit" {
            break;
        }
    }
}
