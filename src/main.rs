pub mod types;

use std::collections::HashMap;
use std::fmt::Debug;
use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock};
use std::io;
use std::io::prelude::*;

use futures::{Future, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    
    /*
    println!("Test input");

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        println!("Output: {}", line);



        if line.to_lowercase() == "exit" {
            break;
        }
    }
    */

     // Create the event loop and TCP listener we'll accept connections on.
     let try_socket = TcpListener::bind("127.0.0.1:8080").await;

    Ok(())
}
