use std::time::{Duration, Instant};
use std::mem;
use futures::channel::mpsc::{UnboundedReceiver};
use futures::{future, StreamExt, TryStreamExt, pin_mut};
use futures::stream::{SplitSink, SplitStream};
use std::sync::mpsc::{channel, Sender, Receiver};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
//use tokio::sync::{broadcast};
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use anyhow::Result;
use minifb::{Key, Window, WindowOptions};
use rust_server::types::*;

//==================================================================

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

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

fn window_loop(wbuffer_rx: Receiver<Vec<u32>>, input_tx: Sender<Key>) {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {        
        // update frame buffer if rendered
        while let Ok(b) = wbuffer_rx.try_recv() {
            let _  = mem::replace(&mut buffer, b);
        }

        window.get_keys().iter().for_each(|key| match key {
            Key::W => { let _ = input_tx.send(*key); },
            Key::S => { let _ = input_tx.send(*key); },
            Key::A => { let _ = input_tx.send(*key); },
            Key::D => { let _ = input_tx.send(*key); },
            _ => { },
        });
        
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

fn render_loop(wbuffer_tx: Sender<Vec<u32>>, input_rx: Receiver<Key>) {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut now = Instant::now();
    let dur = Duration::from_millis(32);

    let mut x_pos: usize = 275;
    let mut y_pos: usize = 75;

    loop {
        // if time has passed render to buffer
        if  now.elapsed() >= dur {
            
            buffer.iter_mut().map(|x| *x = 0).count();
            
            while let Ok(key) = input_rx.try_recv() {
                match key {
                    Key::W => { y_pos += 2; if y_pos >= HEIGHT { y_pos = 0; } },
                    Key::S => { y_pos = y_pos.wrapping_sub(2); if y_pos >= HEIGHT { y_pos = 0; } },
                    Key::A => { x_pos += 2;  if x_pos >= WIDTH { x_pos = 0; }},
                    Key::D => { x_pos = x_pos.wrapping_sub(2);  if x_pos >= WIDTH { x_pos = 0; }},
                    _ => { }
                }
            }

            let x_pos_end = if (x_pos + 25) >= WIDTH { WIDTH - 1 } else { x_pos + 25 };
            let y_pos_end = if (y_pos + 25) >= HEIGHT { HEIGHT - 1 } else { y_pos + 25 };
            
            for y in 0..(HEIGHT-1) {
                for x in 0..(WIDTH-1) {

                    if x >= x_pos && x <= x_pos_end && y >= y_pos && y <= y_pos_end {
                        buffer[(y * WIDTH) + x] = 0x00FF00;
                    }
                }
            }

            let _ = wbuffer_tx.send(buffer.clone());
            now = Instant::now();
        }
    }

}

#[tokio::main]
async fn main() -> Result<()> {
   
    let (fb_sender, fb_receiver) = channel();
    let (input_sender, input_reciever) = channel();

    // initialize window thread
    let window_handle = std::thread::spawn(move || window_loop(fb_receiver, input_sender));
    let render_handle = std::thread::spawn(move || render_loop(fb_sender, input_reciever));

    let url = url::Url::parse("ws:////127.0.0.1:8080").unwrap();
    println!("server url: {}", url);

    if let Ok((ws_stream, _)) = connect_async(url).await {
        println!("websocket handshake successfully completed");
        let (write, read) = ws_stream.split();
        
        let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();

        tokio::select! {
            _ = read_stdin(stdin_tx) => { }
            _ = stdin_to_ws(write, stdin_rx) => { }
            _ = ws_to_stdout(read) => { }
            _ = exit_signal() => { println!("exiting");  }
        }
    }
    else {
        println!("failed to connect");
    }

    window_handle.join().unwrap();
    
    println!("programm ended");

    Ok(())
}
