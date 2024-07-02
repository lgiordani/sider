use crate::resp::RESP;
use crate::storage_result::{StorageError, StorageResult};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum StorageValue {
    String(String),
}

pub struct Storage {
    store: HashMap<String, StorageValue>,
}

impl Storage {
    pub fn new() -> Self {
        let store: HashMap<String, StorageValue> = HashMap::new();

        Self { store: store }
    }

    pub fn process_command(&mut self, command: &Vec<String>) -> StorageResult<RESP> {
        match command[0].to_lowercase().as_str() {
            "ping" => self.command_ping(&command),
            "echo" => self.command_echo(&command),
            _ => Err(StorageError::CommandNotAvailable(command[0].clone())),
        }
    }

    fn command_ping(&self, _command: &Vec<String>) -> StorageResult<RESP> {
        Ok(RESP::SimpleString("PONG".to_string()))
    }

    fn command_echo(&self, command: &Vec<String>) -> StorageResult<RESP> {
        Ok(RESP::BulkString(command[1].clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new() {
        let storage: Storage = Storage::new();

        assert_eq!(storage.store.len(), 0);
    }

    #[test]
    fn test_command_ping() {
        let command = vec![String::from("ping")];
        let storage: Storage = Storage::new();

        let output = storage.command_ping(&command).unwrap();

        assert_eq!(output, RESP::SimpleString(String::from("PONG")));
    }

    #[test]
    fn test_command_ping_uppercase() {
        let command = vec![String::from("PING")];
        let storage: Storage = Storage::new();

        let output = storage.command_ping(&command).unwrap();

        assert_eq!(output, RESP::SimpleString(String::from("PONG")));
    }

    #[test]
    fn test_command_echo() {
        let command = vec![String::from("echo"), String::from("42")];
        let storage: Storage = Storage::new();

        let output = storage.command_echo(&command).unwrap();

        assert_eq!(output, RESP::BulkString(String::from("42")));
    }
}
