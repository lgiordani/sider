use crate::RESP;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ServerError {
    CommandError,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::CommandError => write!(f, "Error while processing!"),
        }
    }
}

pub type ServerResult<T> = Result<T, ServerError>;

pub fn process_request(request: RESP) -> ServerResult<RESP> {
    let elements = match request {
        RESP::Array(v) => v,
        _ => {
            return Err(ServerError::CommandError);
        }
    };

    let mut command = Vec::new();
    for elem in elements.iter() {
        match elem {
            RESP::BulkString(v) => command.push(v),
            _ => {
                return Err(ServerError::CommandError);
            }
        }
    }

    match command[0].to_lowercase().as_str() {
        "ping" => Ok(RESP::SimpleString(String::from("PONG"))),
        _ => {
            return Err(ServerError::CommandError);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_request_ping() {
        let request = RESP::Array(vec![RESP::BulkString(String::from("PING"))]);

        let output = process_request(request).unwrap();

        assert_eq!(output, RESP::SimpleString(String::from("PONG")));
    }

    #[test]
    fn test_process_request_not_array() {
        let request = RESP::BulkString(String::from("PING"));

        let error = process_request(request).unwrap_err();

        assert_eq!(error, ServerError::CommandError);
    }

    #[test]
    fn test_process_request_not_bulkstrings() {
        let request = RESP::Array(vec![RESP::SimpleString(String::from("PING"))]);

        let error = process_request(request).unwrap_err();

        assert_eq!(error, ServerError::CommandError);
    }
}
