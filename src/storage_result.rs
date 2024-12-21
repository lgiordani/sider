use std::fmt;

#[derive(Debug, PartialEq)]
pub enum StorageError {
    CommandSyntaxError(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::CommandSyntaxError(string) => {
                write!(f, "Syntax error while processing {}!", string)
            }
        }
    }
}

pub type StorageResult<T> = Result<T, StorageError>;
