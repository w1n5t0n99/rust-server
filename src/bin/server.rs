use std::collections::HashMap;
use std::os::windows::process;
use std::str::EncodeUtf16;
use std::sync::{Arc, RwLock, Mutex};
use futures::sink::SinkExt;
use futures::{future, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use serde::{Serialize, Deserialize};

use anyhow::Result;

use rust_server::types::{EntityCounter, Entities, Entity, Counter, Event};

//===============================================================

fn process_event(msg: Message) -> Result<Event> {
    match msg {
        Message::Text(txt) => {
            let event: Event = serde_json::from_str(&txt)?;
            Ok(event)
        }
        _ => Err(tokio_tungstenite::tungstenite::Error::Utf8.into())
    }
}

async fn process_socket(socket: TcpStream, counter: EntityCounter) -> Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(socket)
        .await?;

    let (mut write, read) = ws_stream.split();

    // register client
    let reg_event = Event::ClientRegistered(counter.lock().unwrap().next());
    let reg_event_json = serde_json::to_string(&reg_event).unwrap();
    
    write.send(Message::Text(reg_event_json)).await?;

    // loop update client
    let res = read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .then(|msg| {
            future::ready(msg.and_then(|m| { println!("message: {}", m); Ok(m) }))
        })
        .forward(write)
        .await;
    
    match res {
        Ok(_) => {
            println!("connection closed: no error");
            Ok(())
        }
        Err(err) => {
            println!("connection closed: ERROR - {}", err);
            Err(err.into())
        }
    }
}

async fn exit_signal() {
    tokio::signal::ctrl_c().await.expect("signal error");
}

async fn connection_handler(listener: TcpListener, counter: EntityCounter) {
    loop {
        match listener.accept().await {
            Ok((socket, _addr)) => { tokio::spawn( process_socket(socket, counter.clone())); },
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
    let entities: Arc<Mutex<HashMap<u32, Entity>>> = Arc::new(Mutex::new(HashMap::new()));
    // Used to assign a unique id to each new player
    let counter = Arc::new(Mutex::new(Counter::new()));


    tokio::select! {
        _ = connection_handler(server_listener, counter.clone()) => {
            println!("Server exiting");
        }

        _ = exit_signal() => {
            println!("Got it... exiting");
        }
    }
    
    Ok(())
}

