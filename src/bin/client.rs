use std::net::Shutdown;
use futures::{future, StreamExt, TryStreamExt, pin_mut};
use futures::stream::{SplitSink, SplitStream};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, mpsc};
//use tokio::sync::mpsc::{channel, Sender};
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

async fn rs(tx: futures::channel::mpsc::UnboundedSender<Message>, mut shutdown: shutdown::Shutdown, _sender: Sender<()>) {
    tokio::select! {
        _ = read_stdin(tx) => { }

        _ = shutdown.recv() => { println!("read_stdin shutting down");  }
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

async fn stw(write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, stdin_rx: UnboundedReceiver<Message>, mut shutdown: shutdown::Shutdown, _sender: Sender<()>) {
    tokio::select! {
        _ = stdin_to_ws(write, stdin_rx) => { }

        _ = shutdown.recv() => { println!("stdin_to_ws shutting down");  }
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

async fn write_stdout(mut rx: mpsc::Receiver<String>) {
    while let Some(msg) = rx.recv().await {
        let msg = format!("output: {}", msg);

        match tokio::io::stdout().write_all(msg.as_bytes()).await {
            Ok(_) => { },
            Err(_) => { println!("ERROR: could not write message to stdout") },
        }
    }
}

/*
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
*/



/*
async fn wts(read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, mut shutdown: shutdown::Shutdown, _sender: Sender<()>) {
    tokio::select! {
        _ = ws_to_stdout(read) => { }
        
        _ = shutdown.recv() => { println!("ws_to_stdout shutting down");  }
    }
}

async fn exit_signal(tx: broadcast::Sender<()>, _sender: Sender<()>) {
    tokio::signal::ctrl_c().await.expect("signal error");
    tx.send(()).unwrap();
    println!("exit signal ended");
}
*/

#[tokio::main]
async fn main() -> Result<()> {
   
    /*
    let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();

    let url = url::Url::parse("ws:////127.0.0.1:8080").unwrap();
    println!("server url: {}", url);

    let (ws_stream, _) = connect_async(url).await.expect("failed to connect");
    println!("websocket handshake successfully completed");

    let (write, read) = ws_stream.split();
    let (tx, _) = broadcast::channel(16);
    let (send_gaurd, mut rec_gaurd) = channel(1);

    tokio::spawn(stw(write, stdin_rx, shutdown::Shutdown::new(tx.subscribe()), send_gaurd.clone()));
    tokio::spawn(wts(read, shutdown::Shutdown::new(tx.subscribe()), send_gaurd.clone()));
    tokio::spawn(rs(stdin_tx.clone(), shutdown::Shutdown::new(tx.subscribe()), send_gaurd.clone()));
    tokio::spawn(exit_signal(tx, send_gaurd.clone()));


    
    drop(send_gaurd);

    let _ = rec_gaurd.recv().await;
    println!("programm ended");

<<<<<<< HEAD
        _ = shutdown2.recv() => { println!("read_stdin shutting down");  }
    }

    */

    println!("Ctrl+C to exit");

    //let (tx, rx) = mpsc::channel(64);
    let (tx, rx) = std::sync::mpsc::channel();

    tokio::select! {
        _ = tokio::signal::ctrl_c() => { println!("shutting down"); },
        //_ = read_stdin(tx) => { println!("read_stdin shutting down"); },
        //_ = write_stdout(rx) => { println!("write_stdout shutting down"); },
    }
    
    Ok(())
}

/*
    let stdin_channel = spawn_stdin_channel();
    loop {
        match stdin_channel.try_recv() {
            Ok(key) => println!("Received: {}", key),
            Err(TryRecvError::Empty) => println!("Channel empty"),
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
        sleep(1000);
    }
}

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    rx
}
 */