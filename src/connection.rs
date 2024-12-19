use crate::{request::Request, server_result::ServerError};
use std::fmt;

#[derive(Debug)]
pub enum ConnectionError {
    ServerError(ServerError),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::ServerError(e) => {
                write!(f, "{}", format!("Server error: {}", e))
            }
        }
    }
}

#[derive(Debug)]
pub enum ConnectionMessage {
    Request(Request),
}
