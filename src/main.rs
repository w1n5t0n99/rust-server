pub mod types;

use std::collections::HashMap;
use std::fmt::Debug;
use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock};
use std::io;
use std::io::prelude::*;

use futures::{future, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};

use anyhow::Result;

async fn accept_connection(stream: TcpStream) {
    let addr = stream.peer_addr().expect("connected streams should have a peer address");
    println!("{}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("error during websocket handshake occurred");

    let (write, read) = ws_stream.split();
    // we should not forward messages other than text or binary
    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("failed to forward message");
}

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
     let listener = TcpListener::bind("127.0.0.1:8080").await.expect("can't listen");

     while let Ok((stream, addr)) = listener.accept().await {
         println!("socket address: {}", addr);
        tokio::spawn(accept_connection(stream));
     }

    Ok(())
}
