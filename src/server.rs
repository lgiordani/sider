use crate::storage::Storage;
use crate::storage_result::{StorageError, StorageResult};
use crate::RESP;
use std::sync::{Arc, Mutex};

pub fn process_request(request: RESP, storage: Arc<Mutex<Storage>>) -> StorageResult<RESP> {
    let elements = match request {
        RESP::Array(v) => v,
        _ => {
            return Err(StorageError::IncorrectRequest);
        }
    };

    let mut command = Vec::new();
    for elem in elements.iter() {
        match elem {
            RESP::BulkString(v) => command.push(v.clone()),
            _ => {
                return Err(StorageError::IncorrectRequest);
            }
        }
    }

    let mut guard = storage.lock().unwrap();
    let response = guard.process_command(&command);
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_request_ping() {
        let request = RESP::Array(vec![RESP::BulkString(String::from("PING"))]);
        let storage = Arc::new(Mutex::new(Storage::new()));

        let output = process_request(request, storage).unwrap();

        assert_eq!(output, RESP::SimpleString(String::from("PONG")));
    }

    #[test]
    fn test_process_request_echo() {
        let request = RESP::Array(vec![
            RESP::BulkString(String::from("ECHO")),
            RESP::BulkString(String::from("42")),
        ]);
        let storage = Arc::new(Mutex::new(Storage::new()));

        let output = process_request(request, storage).unwrap();

        assert_eq!(output, RESP::BulkString(String::from("42")));
    }

    #[test]
    fn test_process_request_not_array() {
        let request = RESP::BulkString(String::from("PING"));
        let storage = Arc::new(Mutex::new(Storage::new()));

        let error = process_request(request, storage).unwrap_err();

        assert_eq!(error, StorageError::IncorrectRequest);
    }

    #[test]
    fn test_process_request_not_bulkstrings() {
        let request = RESP::Array(vec![RESP::SimpleString(String::from("PING"))]);
        let storage = Arc::new(Mutex::new(Storage::new()));

        let error = process_request(request, storage).unwrap_err();

        assert_eq!(error, StorageError::IncorrectRequest);
    }
}
