use crate::connection::ConnectionMessage;
use crate::request::Request;
use crate::storage::Storage;
use crate::RESP;
use tokio::sync::mpsc;

pub struct Server {
    pub storage: Option<Storage>,
}

impl Server {
    pub fn new() -> Self {
        Self { storage: None }
    }

    pub fn set_storage(mut self, storage: Storage) -> Self {
        self.storage = Some(storage);
        self
    }
}

pub async fn run_server(mut server: Server, mut crx: mpsc::Receiver<ConnectionMessage>) {
    loop {
        tokio::select! {
            Some(message) = crx.recv() => {
                match message {
                    ConnectionMessage::Request(request) => {
                        process_request(request, &mut server).await;
                    }
                }
            }
        }
    }
}

pub async fn process_request(request: Request, server: &mut Server) {
    let elements = match &request.value {
        RESP::Array(v) => v,
        _ => {
            panic!()
        }
    };

    let mut command = Vec::new();
    for elem in elements.iter() {
        match elem {
            RESP::BulkString(v) => command.push(v.clone()),
            _ => {
                panic!()
            }
        }
    }

    let storage = match server.storage.as_mut() {
        Some(storage) => storage,
        None => panic!(),
    };

    let _response = storage.process_command(&command);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new() {
        let server: Server = Server::new();

        match server.storage {
            Some(_) => panic!(),
            None => (),
        };
    }

    #[test]
    fn test_set_storage() {
        let storage = Storage::new();

        let server: Server = Server::new().set_storage(storage);

        match server.storage {
            Some(_) => (),
            None => panic!(),
        };
    }

    // #[test]
    // fn test_process_request_ping() {
    //     let (connection_sender, _) = mpsc::channel::<ServerMessage>(32);

    //     let request = Request {
    //         value: RESP::Array(vec![RESP::BulkString(String::from("PING"))]),
    //         sender: connection_sender,
    //     };

    //     let storage = Arc::new(Mutex::new(Storage::new()));

    //     let output = process_request(request, storage).unwrap();

    //     assert_eq!(output, RESP::SimpleString(String::from("PONG")));
    // }

    // #[test]
    // fn test_process_request_echo() {
    //     let (connection_sender, _) = mpsc::channel::<ServerMessage>(32);

    //     let request = Request {
    //         value: RESP::Array(vec![
    //             RESP::BulkString(String::from("ECHO")),
    //             RESP::BulkString(String::from("42")),
    //         ]),
    //         sender: connection_sender,
    //     };

    //     let storage = Arc::new(Mutex::new(Storage::new()));

    //     let output = process_request(request, storage).unwrap();

    //     assert_eq!(output, RESP::BulkString(String::from("42")));
    // }

    // #[test]
    // fn test_process_request_not_array() {
    //     let (connection_sender, _) = mpsc::channel::<ServerMessage>(32);

    //     let request = Request {
    //         value: RESP::BulkString(String::from("PING")),
    //         sender: connection_sender,
    //     };

    //     let storage = Arc::new(Mutex::new(Storage::new()));

    //     let error = process_request(request, storage).unwrap_err();

    //     assert_eq!(error, StorageError::IncorrectRequest);
    // }

    // #[test]
    // fn test_process_request_not_bulkstrings() {
    //     let (connection_sender, _) = mpsc::channel::<ServerMessage>(32);

    //     let request = Request {
    //         value: RESP::Array(vec![RESP::SimpleString(String::from("PING"))]),
    //         sender: connection_sender,
    //     };

    //     let storage = Arc::new(Mutex::new(Storage::new()));

    //     let error = process_request(request, storage).unwrap_err();

    //     assert_eq!(error, StorageError::IncorrectRequest);
    // }
}
