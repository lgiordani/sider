use crate::resp::RESP;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ServerError {
    IncorrectData,
    StorageNotInitialised,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::IncorrectData => {
                write!(f, "Data received from stream is incorrect.")
            }
            ServerError::StorageNotInitialised => {
                write!(f, "Storage has not been initialised.")
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ServerValue {
    RESP(RESP),
}

pub type _ServerResult = Result<ServerValue, ServerError>;

#[derive(Debug, PartialEq)]
pub enum ServerMessage {
    Data(ServerValue),
    Error(ServerError),
}
