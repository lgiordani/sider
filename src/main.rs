use crate::request::Request;
use crate::resp::{bytes_to_resp, RESP};
use crate::storage::Storage;
use connection::ConnectionMessage;
use server_result::ServerMessage;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    select,
    sync::mpsc,
};

mod connection;
mod request;
mod resp;
mod resp_result;
mod server;
mod server_result;
mod set;
mod storage;
mod storage_result;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let storage = Arc::new(Mutex::new(Storage::new()));

    let mut interval_timer = tokio::time::interval(Duration::from_millis(10));

    let (server_sender, _) = mpsc::channel::<ConnectionMessage>(32);

    loop {
        tokio::select! {
            connection = listener.accept() => {
                match connection {
                    Ok((stream, _)) => {
                        tokio::spawn(handle_connection(stream, server_sender.clone()));
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }
                }
            }

            _ = interval_timer.tick() => {
                tokio::spawn(expire_keys(storage.clone()));
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream, server_sender: mpsc::Sender<ConnectionMessage>) {
    let mut buffer = [0; 512];

    let (connection_sender, _) = mpsc::channel::<ServerMessage>(32);

    loop {
        select! {
            result = stream.read(&mut buffer) => {
                match result {
                    Ok(size) if size != 0 => {
                        let mut index: usize = 0;

                        let resp = match bytes_to_resp(&buffer[..size].to_vec(), &mut index) {
                            Ok(v) => v,
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                return;
                            }
                        };

                        let request = Request {
                            value: resp,
                            sender: connection_sender.clone(),
                        };

                        match server_sender.send(ConnectionMessage::Request(request)).await {
                            Ok(()) => {},
                            Err(e) => {
                                eprintln!("Error sending request: {}", e);
                                return;
                            }
                        }
                    }
                    Ok(_) => {
                        println!("Connection closed");
                        break;
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        break;
                    }
                }
            }
        }
    }
}

async fn expire_keys(storage: Arc<Mutex<Storage>>) {
    let mut guard = storage.lock().unwrap();

    guard.expire_keys();
}
