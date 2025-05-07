use crate::resp::RESP;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ServerError {
    CommandInternalError(String),
    CommandNotAvailable(String),
    CommandSyntaxError(String),
    IncorrectData,
    StorageNotInitialised,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::CommandInternalError(string) => {
                write!(f, "Internal error while processing {}.", string)
            }

            ServerError::CommandNotAvailable(c) => {
                write!(f, "The requested command {} is not available.", c)
            }

            ServerError::CommandSyntaxError(string) => {
                write!(f, "Syntax error while processing {}.", string)
            }

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
    None,
    RESP(RESP),
}

pub type ServerResult = Result<ServerValue, ServerError>;

#[derive(Debug, PartialEq)]
pub enum ServerMessage {
    Data(ServerValue),
    Error(ServerError),
}
