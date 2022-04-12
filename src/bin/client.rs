use std::{time::{Duration, Instant}};
use futures::stream::SplitStream;
//use std::sync::mpsc::{channel, Sender, Receiver};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

//use tokio::sync::{broadcast};
//use tokio::sync::mpsc;
use futures_util::{future, pin_mut, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use anyhow::Result;
use tracing::{info_span, Instrument};
use std::env;

type MsgUnboundedReciever = futures::channel::mpsc::UnboundedReceiver<Message>;
type MsgUnboundedSender = futures::channel::mpsc::UnboundedSender<Message>;

#[tokio::main]
async fn main() {
    let connect_addr =
        env::args().nth(1).unwrap_or_else(|| panic!("this program requires at least one argument"));

    let url = url::Url::parse(&connect_addr).unwrap();

    let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();

    tokio::spawn(read_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            tokio::io::stdout().write_all(&data).await.unwrap();
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}


async fn handle_incoming_messages(read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
    read.for_each(|message| async {
        let data = message.unwrap().into_data();
        tokio::io::stdout().write_all(&data).await.unwrap();
    });
}


#[tracing::instrument(
    name = "Read stdin task",
    skip(tx),
)]
async fn read_stdin(tx: MsgUnboundedSender) {

    let mut stdin = tokio::io::stdin();
    loop {
        let span = info_span!("stdin read");

        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf)
            .instrument(span)
            .await {
            Err(e) => { 
                tracing::error!("Failed to read stdin: {:?}",e);
                break;
            },
            Ok(0) => break,
            Ok(n) => n,
        };

        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}