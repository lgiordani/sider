use crate::{
    resp::RESP,
    server_result::{ServerError, ServerMessage, ServerValue},
};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Request {
    pub value: RESP,
    pub sender: mpsc::Sender<ServerMessage>,
}

impl Request {
    pub async fn error(&self, e: ServerError) {
        self.sender.send(ServerMessage::Error(e)).await.unwrap();
    }

    pub async fn data(&self, d: ServerValue) {
        self.sender.send(ServerMessage::Data(d)).await.unwrap();
    }
}
