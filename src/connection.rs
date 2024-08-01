use crate::request::Request;

#[derive(Debug)]
pub enum ConnectionMessage {
    Request(Request),
}
