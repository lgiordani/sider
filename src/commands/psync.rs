use crate::request::Request;
use crate::resp::RESP;
use crate::server::Server;
use crate::server_result::ServerValue;

pub async fn command(server: &mut Server, request: &Request, _command: &Vec<String>) {
    server.replication.master_repl_offset = 0;

    let resp = ServerValue::RESP(RESP::SimpleString(format!(
        "FULLRESYNC {} {}",
        server.replication.master_replid.clone(),
        server.replication.master_repl_offset.to_string()
    )));

    request.data(resp).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_result::{ServerMessage, ServerValue};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_command() {
        let mut server = Server::new("localhost".to_string(), 6379);

        let cmd = vec![String::from("psync"), String::from("?"), String::from("-1")];

        let (request_channel_tx, mut request_channel_rx) = mpsc::channel::<ServerMessage>(32);

        let request = Request {
            value: RESP::Null,
            sender: request_channel_tx.clone(),
        };

        server.replication.master_replid = String::from("some_repl_id");
        server.replication.master_repl_offset = 1234;

        command(&mut server, &request, &cmd).await;

        assert_eq!(
            request_channel_rx.try_recv().unwrap(),
            ServerMessage::Data(ServerValue::RESP(RESP::SimpleString(String::from(
                "FULLRESYNC some_repl_id 0"
            ))))
        );
    }
}
