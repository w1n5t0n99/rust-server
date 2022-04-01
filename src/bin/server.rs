use std::env;
use futures::sink::SinkExt;
use futures::{future, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use serde::{Serialize, Deserialize};
use log::info;
use anyhow::Result;

//===============================================================
/*
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
*/

//===================================================================
//===================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let _ = env_logger::try_init();
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream.peer_addr().expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let (write, read) = ws_stream.split();
    // We should not forward messages other than text or binary.
    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("Failed to forward messages")
}