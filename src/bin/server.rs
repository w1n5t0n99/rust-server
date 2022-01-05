use std::collections::HashMap;
use std::os::windows::process;
use std::sync::{Arc, RwLock};

use futures::{future, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};

use anyhow::Result;


/*
async fn accept_connection(stream: TcpStream) {
    let addr = stream.peer_addr().expect("connected streams should have a peer address");
    println!("{}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("error during websocket handshake occurred");

    let (write, read) = ws_stream.split();
    // we should not forward messages other than text or binary
    let res = read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .then(|msg| {
            future::ready(msg.and_then(|m| { println!("message from {}: {}", addr, m); Ok(m) }))
        })
        .forward(write)
        .await;

    match res {
        Ok(_) => {
            println!("connection closed {}: no error", addr);
        }
        Err(err) => {
            println!("connection closed {}: ERROR - {}", addr, err);
        }
    }
}

async fn main_loop(listener: TcpListener) {
    println!("Ctrl-C to exit");
    while let Ok((stream, addr)) = listener.accept().await {
       println!("socket address: {}", addr);
       tokio::spawn(accept_connection(stream));

    }
}
*/

async fn process_socket(socket: TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(socket)
        .await
        .expect("error during websocket handshake occurred"); // TODO perform better error handling

    let (write, read) = ws_stream.split();

    
    
}

async fn exit_signal() {
    tokio::signal::ctrl_c().await.expect("signal error");
}

async fn connection_handler(listener: TcpListener) {
    loop {
        match listener.accept().await {
            Ok((socket, _addr)) => { tokio::spawn( process_socket(socket)); },
            Err(e) => println!("couldn't get client: {:?}", e),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {

     // Create the event loop and TCP listener we'll accept connections on.
    let server_listener = TcpListener::bind("127.0.0.1:8080").await?;

    // Hashmap to store a sink value with an id key
    // A sink is used to send data to an open client connection
    //let connections = Arc::new(RwLock::new(HashMap::new()));
    // Hashmap of id:entity pairs. This is basically the game state
    //let entities = Arc::new(RwLock::new(HashMap::new()));
    // Used to assign a unique id to each new player
    let counter = Arc::new(RwLock::new(0));

    
    tokio::select! {
        _ = connection_handler(server_listener) => {
            println!("Server exiting");
        }

        _ = exit_signal() => {
            println!("Got it... exiting");
        }
    }
    



    Ok(())
}


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
