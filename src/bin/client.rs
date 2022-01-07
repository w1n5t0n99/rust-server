use futures::channel::mpsc::UnboundedReceiver;
use futures::{future, StreamExt, TryStreamExt, pin_mut};
use futures::stream::{SplitSink, SplitStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
//use tokio::sync::{broadcast};
//use tokio::sync::mpsc::{channel, Sender};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use anyhow::Result;

//==================================================================

async fn read_stdin(tx: futures::channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };

        buf.truncate(n);
        let res = tx.unbounded_send(Message::binary(buf));
        if res.is_err() { break; }
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

    println!("stdin_to_ws ended");
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

    println!("ws_to_stdout ended");
}

async fn exit_signal() {
    tokio::signal::ctrl_c().await.expect("signal error");
}


#[tokio::main]
async fn main() -> Result<()> {
   
    let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();

    let url = url::Url::parse("ws:////127.0.0.1:8080").unwrap();
    println!("server url: {}", url);

    let (ws_stream, _) = connect_async(url).await.expect("failed to connect");
    println!("websocket handshake successfully completed");
    let (write, read) = ws_stream.split();
  
    tokio::select! {
        _ = read_stdin(stdin_tx) => { }
        _ = stdin_to_ws(write, stdin_rx) => { }
        _ = ws_to_stdout(read) => { }
        _ = exit_signal() => { println!("exiting");  }
    }
    
    println!("programm ended");

    Ok(())
}
