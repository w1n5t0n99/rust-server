pub mod types;

use std::collections::HashMap;
use std::fmt::Debug;
use std::time::{Duration, Instant};

use websocket::message::OwnedMessage;
use websocket::server::InvalidConnection;
use websocket::server::r#async::Server;

use std::sync::{Arc, RwLock};

fn main() {
    println!("Hello, world!");
}
