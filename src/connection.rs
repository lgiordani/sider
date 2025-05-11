use crate::resp::{bytes_to_resp, RESP};
use crate::server::{handshake, ServerInfo};
use crate::server_result::{ServerMessage, ServerValue};
use crate::{request::Request, server_result::ServerError};
use std::fmt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    sync::mpsc,
};

#[derive(Debug)]
pub enum ConnectionError {
    CannotReadFromStream(String),
    CannotWriteToStream(String),
    MalformedRESP(String),
    ServerError(ServerError),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::CannotReadFromStream(string) => {
                write!(f, "Cannot read from stream: {}.", string)
            }
            ConnectionError::CannotWriteToStream(string) => {
                write!(f, "Cannot write to stream: {}.", string)
            }
            ConnectionError::MalformedRESP(string) => {
                write!(f, "Cannot convert bytes to RESP: {}.", string)
            }
            ConnectionError::ServerError(e) => {
                write!(f, "{}", format!("Server error: {}", e))
            }
        }
    }
}

type ConnectionResult<T> = Result<T, ConnectionError>;

#[derive(Debug)]
pub enum ConnectionMessage {
    Request(Request),
}

pub async fn run_listener(host: String, port: u16, server_sender: mpsc::Sender<ConnectionMessage>) {
    let listener = TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();

    loop {
        tokio::select! {
            connection = listener.accept() => {
                match connection {
                    Ok((stream, _)) => {
                        tokio::spawn(handle_connection(stream, server_sender.clone()));
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        continue;
                    }
                }
            }
        }
    }
}

pub async fn run_master_listener(
    host: String,
    port: u16,
    info: &ServerInfo,
    server_sender: mpsc::Sender<ConnectionMessage>,
) {
    // Actively connect to the master
    let mut stream = TcpStream::connect(format!("{}:{}", host, port))
        .await
        .unwrap();

    // Run the handshake protocol
    if let Err(e) = handshake(&mut stream, info).await {
        eprintln!("Handshake failed: {}", e.to_string());
        std::process::exit(1);
    }

    tokio::spawn(async move { handle_connection(stream, server_sender.clone()).await });
}

pub async fn handle_connection(
    mut stream: TcpStream,
    server_sender: mpsc::Sender<ConnectionMessage>,
) {
    let mut buffer = [0; 512];

    let (connection_sender, mut connection_receiver) = mpsc::channel::<ServerMessage>(32);

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

            Some(response) = connection_receiver.recv() => {
                let _ = match response {
                    ServerMessage::Data(ServerValue::RESP(v)) => stream.write_all(v.to_string().as_bytes()).await,
                    ServerMessage::Data(ServerValue::None) => Ok(()),
                    ServerMessage::Error(e) => {
                        eprintln!("Error: {}", ConnectionError::ServerError(e));
                        return;
                    }
                };
            }

        }
    }
}

// Write RESP data to the stream
pub async fn stream_write_resp(stream: &mut TcpStream, data: &RESP) -> ConnectionResult<usize> {
    let string_data = data.to_string();
    let bytes = string_data.as_bytes();

    match stream.write_all(bytes).await {
        Ok(_) => Ok(bytes.len()),
        Err(e) => Err(ConnectionError::CannotWriteToStream(e.to_string())),
    }
}

// Read RESP data from the stream
async fn stream_read_resp(stream: &mut TcpStream, buffer: &mut [u8]) -> ConnectionResult<RESP> {
    match stream.read(buffer).await {
        Ok(size) => Ok(size),
        Err(e) => Err(ConnectionError::CannotReadFromStream(e.to_string())),
    }?;

    let mut index: usize = 0;

    bytes_to_resp(&buffer, &mut index).map_err(|e| ConnectionError::MalformedRESP(e.to_string()))
}
