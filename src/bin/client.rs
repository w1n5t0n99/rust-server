use futures::{future, StreamExt, TryStreamExt, pin_mut};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

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

async fn exit_signal(tx: futures::channel::mpsc::UnboundedSender<Message>) {
    tokio::signal::ctrl_c().await.expect("signal error");
    tx.unbounded_send(Message::Close(None)).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
   
    let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx.clone()));

    let url = url::Url::parse("ws:////127.0.0.1:8080").unwrap();

    println!("server url: {}", url);

    let (ws_stream, _) = connect_async(url).await.expect("failed to connect");
    println!("websocket handshake successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(|m| Ok(m)).forward(write);

    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_text().unwrap();
            let output_msg = format!("server: {}", data);
            tokio::io::stdout().write_all(output_msg.as_bytes()).await.unwrap();
        })
    };

    //pin_mut!(stdin_to_ws, ws_to_stdout);
    //future::select(stdin_to_ws, ws_to_stdout).await;
    
    tokio::select! {
        _ = stdin_to_ws => {  println!("client input exiting"); }

        _ = ws_to_stdout => {  println!("client output exiting"); }

        _ = exit_signal(stdin_tx.clone()) => {
            println!("Got it... exiting");
        }
    }

    Ok(())
}