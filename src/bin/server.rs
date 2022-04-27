use std::env;
use std::net::SocketAddr;
use futures::sink::SinkExt;
use futures::{future, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use serde::{Serialize, Deserialize};

use tracing::instrument::WithSubscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing::{instrument, Instrument};
use tracing::{info, info_span};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;

type MsgUnboundedReciever = futures::channel::mpsc::UnboundedReceiver<Message>;
type MsgUnboundedSender = futures::channel::mpsc::UnboundedSender<Message>;


//===================================================================
//===================================================================

#[tokio::main]
async fn main() {
    // initialize tracing
    let formatting_layer = BunyanFormattingLayer::new("tracing_demo".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(JsonStorageLayer)
        .with(formatting_layer);

    /*
    let file_appender = tracing_appender::rolling::hourly("/logs", "prefix.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
       .with_writer(non_blocking)
       .init();
    */

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());
    // Create the event loop and TCP listener we'll accept connections on.
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    info!("Listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(accept_connection(stream, addr));
    }

}

#[instrument]
async fn accept_connection(stream: TcpStream, addr: SocketAddr) {
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let (write, read) = ws_stream.split();
    // We should not forward messages other than text or binary.
    read.try_filter(|msg| {
            match msg.to_text() {
                Ok(msg) => info!("message: {}", msg),
                Err(_) => info!("non text message"),
            }

            future::ready(msg.is_text() || msg.is_binary())
        })
        .forward(write)
        .instrument(info_span!("forward message"))
        .await
        .expect("Failed to forward messages")
}