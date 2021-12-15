use std::net::Shutdown;
use futures::{future, StreamExt, TryStreamExt, pin_mut};
use futures::stream::{SplitSink, SplitStream};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, mpsc};
use tokio::net::TcpStream;

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};

use anyhow::Result;

use rust_server::shutdown;

/*
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
}
*/

async fn read_stdin(tx: mpsc::Sender<String>) {
    let mut stdin = tokio::io::stdin();

    loop {
        let mut buf = vec![0; 1024];

        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };

        buf.truncate(n);
        //tx.send(Message::binary(buf)).await.expect("send message error");
        let msg = match String::from_utf8(buf) {
            Ok(msg) => msg,
            Err(_) => "ERROR: message could not be parsed".to_string(),
        };

        tx.send(msg).await.expect("send message error");
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

async fn wts(read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, mut shutdown: shutdown::Shutdown) {
    tokio::select! {
        _ = ws_to_stdout(read) => { }

        _ = shutdown.recv() => { println!("ws_to_stdout shutting down");  }
    }
}

async fn exit_signal(tx: broadcast::Sender<()>) {
    tokio::signal::ctrl_c().await.expect("signal error");
    tx.send(()).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
   
    /*
    let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();

    let (tx, _rx1) = broadcast::channel(16);
    let mut shutdown0 = shutdown::Shutdown::new(tx.subscribe()); 
    let mut shutdown1 = shutdown::Shutdown::new(tx.subscribe()); 
    let mut shutdown2 = shutdown::Shutdown::new(tx.subscribe()); 

    let url = url::Url::parse("ws:////127.0.0.1:8080").unwrap();
    println!("server url: {}", url);

    let (ws_stream, _) = connect_async(url).await.expect("failed to connect");
    println!("websocket handshake successfully completed");

    let (write, read) = ws_stream.split();

    tokio::spawn(exit_signal(tx));
    let stw_handle = tokio::spawn(stdin_to_ws(write, stdin_rx));
    tokio::spawn(wts(read, shutdown1));
    let rts_handle = tokio::spawn(read_stdin(stdin_tx.clone()));

    tokio::select! {
        _ = stw_handle => {  println!("client input exiting"); }

        _ = shutdown0.recv() => { println!("stdin_to_ws shutting down");  }
    }
    
    tokio::select! {
        _ = rts_handle => { }

        _ = shutdown2.recv() => { println!("read_stdin shutting down");  }
    }

    */
    
    Ok(())
}