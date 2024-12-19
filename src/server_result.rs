use crate::resp::RESP;
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

#[derive(Debug)]
pub enum ServerValue {
    RESP(RESP),
}

pub type ServerResult = Result<ServerValue, ServerError>;

#[derive(Debug)]
pub enum ServerMessage {
    Data(ServerValue),
    Error(ServerError),
}
