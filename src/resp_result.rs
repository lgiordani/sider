#[derive(Debug)]
pub enum RESPError {}

pub type RESPResult<T> = Result<T, RESPError>;
