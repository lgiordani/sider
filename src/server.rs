use crate::commands::{echo, get, info, ping, set};
use crate::connection::{stream_send_receive_resp, ConnectionMessage};
use crate::replication::ReplicationConfig;
use crate::request::Request;
use crate::resp::bytes_to_resp;
use crate::server_result::{ServerError, ServerResult, ServerValue};
use crate::storage::Storage;
use crate::RESP;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct ServerInfo {
    pub host: String,
    pub port: u16,
}

pub struct Server {
    pub info: ServerInfo,
    pub storage: Option<Storage>,
    pub replication: ReplicationConfig,
}

impl Server {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            info: ServerInfo {
                host: host,
                port: port,
            },
            storage: None,
            replication: ReplicationConfig::new_master(),
        }
    }

    pub fn set_storage(&mut self, storage: Storage) {
        self.storage = Some(storage);
    }

    pub fn set_replication(&mut self, config: ReplicationConfig) {
        self.replication = config;
    }

    pub fn expire_keys(&mut self) {
        let storage = match self.storage.as_mut() {
            Some(storage) => storage,
            None => return,
        };

        storage.expire_keys();
    }
}

pub async fn run_server(mut server: Server, mut crx: mpsc::Receiver<ConnectionMessage>) {
    let mut interval_timer = tokio::time::interval(Duration::from_millis(10));

    loop {
        tokio::select! {
            Some(message) = crx.recv() => {
                match message {
                    ConnectionMessage::Request(request) => {
                        process_request(request, &mut server).await;
                    }
                }
            }

            _ = interval_timer.tick() => {
                server.expire_keys();
            }
        }
    }
}

pub async fn process_request(request: Request, server: &mut Server) {
    let elements = match &request.value {
        RESP::Array(v) => v,
        _ => {
            request.error(ServerError::IncorrectData).await;
            return;
        }
    };

    let mut command = Vec::new();
    for elem in elements.iter() {
        match elem {
            RESP::BulkString(v) => command.push(v.clone()),
            _ => {
                request.error(ServerError::IncorrectData).await;
                return;
            }
        }
    }

    let command_name = command[0].to_lowercase();

    match command_name.as_str() {
        "echo" => {
            echo::command(server, &request, &command).await;
        }
        "get" => {
            get::command(server, &request, &command).await;
        }
        "info" => {
            info::command(server, &request, &command).await;
        }
        "ping" => {
            ping::command(server, &request, &command).await;
        }
        "set" => {
            set::command(server, &request, &command).await;
        }
        _ => {
            request
                .error(ServerError::CommandNotAvailable(command[0].clone()))
                .await;
        }
    }
}

pub async fn handshake(stream: &mut TcpStream, info: &ServerInfo) -> ServerResult {
    let mut buffer = [0; 512];

    /////////////////////////////////////
    // Send PING

    let ping = RESP::Array(vec![RESP::SimpleString(String::from("PING"))]);

    let resp = stream_send_receive_resp(stream, &ping, &mut buffer)
        .await
        .map_err(|e| ServerError::HandshakeFailed(e.to_string()))?;

    if resp != RESP::SimpleString(String::from("PONG")) {
        return Err(ServerError::HandshakeFailed(String::from("PING failed")));
    };

    /////////////////////////////////////
    // Send REPLCONF listening-port xxx

    let replconf = RESP::Array(vec![
        RESP::SimpleString(String::from("REPLCONF")),
        RESP::SimpleString(String::from("listening-port")),
        RESP::SimpleString(info.port.to_string()),
    ]);

    let resp = stream_send_receive_resp(stream, &replconf, &mut buffer)
        .await
        .map_err(|e| ServerError::HandshakeFailed(e.to_string()))?;

    if resp != RESP::SimpleString(String::from("OK")) {
        return Err(ServerError::HandshakeFailed(format!(
            "Sending {} - Wrong server answer: {}",
            replconf.to_string(),
            resp.to_string()
        )));
    };

    /////////////////////////////////////
    // Send REPLCONF capa psync2

    let replconf = RESP::Array(vec![
        RESP::SimpleString(String::from("REPLCONF")),
        RESP::SimpleString(String::from("capa")),
        RESP::SimpleString(String::from("psync2")),
    ]);

    let resp = stream_send_receive_resp(stream, &replconf, &mut buffer)
        .await
        .map_err(|e| ServerError::HandshakeFailed(e.to_string()))?;

    if resp != RESP::SimpleString(String::from("OK")) {
        return Err(ServerError::HandshakeFailed(format!(
            "Sending {} - Wrong server answer: {}",
            replconf.to_string(),
            resp.to_string()
        )));
    };

    /////////////////////////////////////
    // Send PSYNC ? -1

    let psync = RESP::Array(vec![
        RESP::SimpleString(String::from("PSYNC")),
        RESP::SimpleString(String::from("?")),
        RESP::SimpleString(String::from("-1")),
    ]);

    stream
        .write_all(psync.to_string().as_bytes())
        .await
        .map_err(|e| {
            ServerError::HandshakeFailed(format!(
                "Sending {} - Cannot write to stream: {}",
                replconf.to_string(),
                e.to_string()
            ))
        })?;

    Ok(ServerValue::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_result::{ServerMessage, ServerValue};

    #[test]
    fn test_create_new() {
        let server: Server = Server::new("localhost".to_string(), 6379);

        match server.storage {
            Some(_) => panic!(),
            None => (),
        };
    }

    #[test]
    fn test_set_storage() {
        let storage = Storage::new();

        let mut server: Server = Server::new("localhost".to_string(), 6379);

        server.set_storage(storage);

        match server.storage {
            Some(_) => (),
            None => panic!(),
        };
    }

    #[tokio::test]
    async fn test_process_request_ping() {
        let (connection_sender, mut connection_receiver) = mpsc::channel::<ServerMessage>(32);

        let request = Request {
            value: RESP::Array(vec![RESP::BulkString(String::from("PING"))]),
            sender: connection_sender,
        };

        let storage = Storage::new();

        let mut server: Server = Server::new("localhost".to_string(), 6379);
        server.set_storage(storage);

        process_request(request, &mut server).await;

        assert_eq!(
            connection_receiver.try_recv().unwrap(),
            ServerMessage::Data(ServerValue::RESP(RESP::SimpleString(String::from("PONG"))))
        );
    }

    #[tokio::test]
    async fn test_process_request_echo() {
        let (connection_sender, mut connection_receiver) = mpsc::channel::<ServerMessage>(32);

        let request = Request {
            value: RESP::Array(vec![
                RESP::BulkString(String::from("ECHO")),
                RESP::BulkString(String::from("42")),
            ]),
            sender: connection_sender,
        };

        let storage = Storage::new();

        let mut server: Server = Server::new("localhost".to_string(), 6379);
        server.set_storage(storage);

        process_request(request, &mut server).await;

        assert_eq!(
            connection_receiver.try_recv().unwrap(),
            ServerMessage::Data(ServerValue::RESP(RESP::BulkString(String::from("42"))))
        );
    }

    #[tokio::test]
    async fn test_process_request_not_array() {
        let (connection_sender, mut connection_receiver) = mpsc::channel::<ServerMessage>(32);

        let request = Request {
            value: RESP::BulkString(String::from("PING")),
            sender: connection_sender,
        };

        let storage = Storage::new();

        let mut server: Server = Server::new("localhost".to_string(), 6379);
        server.set_storage(storage);

        process_request(request, &mut server).await;

        assert_eq!(
            connection_receiver.try_recv().unwrap(),
            ServerMessage::Error(ServerError::IncorrectData)
        );
    }

    #[tokio::test]
    async fn test_process_request_not_bulkstrings() {
        let (connection_sender, mut connection_receiver) = mpsc::channel::<ServerMessage>(32);

        let request = Request {
            value: RESP::Array(vec![RESP::SimpleString(String::from("PING"))]),
            sender: connection_sender,
        };

        let storage = Storage::new();

        let mut server: Server = Server::new("localhost".to_string(), 6379);
        server.set_storage(storage);

        process_request(request, &mut server).await;

        assert_eq!(
            connection_receiver.try_recv().unwrap(),
            ServerMessage::Error(ServerError::IncorrectData)
        );
    }
}
