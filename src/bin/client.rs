use futures::channel::mpsc::UnboundedReceiver;
use futures::{future, StreamExt, TryStreamExt, pin_mut};
use futures::stream::{SplitSink, SplitStream};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, mpsc};
use tokio::net::TcpStream;

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};

use anyhow::Result;

async fn read_stdin(tx: futures::channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}

async fn stdin_to_ws(write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, stdin_rx: UnboundedReceiver<Message>) {
    let res = stdin_rx
        .map(|m| Ok(m))
        .forward(write)
        .await;

    match res {
        Ok(_) => { }
        Err(_) => { println!("error sending input to server"); }
    }
}

async fn ws_to_stdout(read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
    read.for_each(|message| async {
        match message {
            Ok(message) => {
                let output_msg = format!("server: {}", message.into_text().unwrap());
                tokio::io::stdout().write_all(output_msg.as_bytes()).await.unwrap();
            }
            Err(_) => {
                println!("error reading message");
            }
        }
    }).await;
}

async fn exit_signal(tx: broadcast::Sender<u8>) {
    tokio::signal::ctrl_c().await.expect("signal error");
    tx.send(0).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
   
    let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();
    let (tx, mut rx1) = broadcast::channel::<u8>(16);
    //let mut rx2 = tx.subscribe();    

    let url = url::Url::parse("ws:////127.0.0.1:8080").unwrap();
    println!("server url: {}", url);

    let (ws_stream, _) = connect_async(url).await.expect("failed to connect");
    println!("websocket handshake successfully completed");

    let (write, read) = ws_stream.split();

    //pin_mut!(stdin_to_ws, ws_to_stdout);
    //future::select(stdin_to_ws, ws_to_stdout).await;
    
    tokio::select! {
        _ = stdin_to_ws(write, stdin_rx) => {  println!("client input exiting"); }

        _ = ws_to_stdout(read) => {  println!("client output exiting"); }

        _ = read_stdin(stdin_tx.clone()) => { }

        _ = exit_signal(tx) => { println!("Got it... exiting"); }
    }

    Ok(())
}