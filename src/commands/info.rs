use crate::replication::Role;
use crate::request::Request;
use crate::resp::bulk_string_from_vec;
use crate::server::Server;
use crate::server_result::ServerValue;

pub async fn command(server: &mut Server, request: &Request, _command: &Vec<String>) {
    let replication_info = server.replication.info();

    let mut output = vec![String::from("# Replication")];

    match replication_info.role {
        Role::Replica => output.push(String::from("role:slave")),
        Role::Master => output.push(String::from("role:master")),
    };

    output.push(format!("master_replid:{}", replication_info.master_replid));
    output.push(format!(
        "master_repl_offset:{}",
        replication_info.master_repl_offset
    ));

    request
        .data(ServerValue::RESP(bulk_string_from_vec(output)))
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replication::ReplicationConfig;
    use crate::resp::RESP;
    use crate::server_result::{ServerMessage, ServerValue};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_command_master() {
        let mut server: Server = Server::new();

        let cmd = vec![String::from("info")];

        let (request_channel_tx, mut request_channel_rx) = mpsc::channel::<ServerMessage>(32);

        let request = Request {
            value: RESP::Null,
            sender: request_channel_tx.clone(),
        };

        command(&mut server, &request, &cmd).await;

        let response = match request_channel_rx.try_recv().unwrap() {
            ServerMessage::Data(ServerValue::RESP(RESP::BulkString(value))) => value,
            _ => panic!(),
        };

        assert!(response.contains("# Replication"));
        assert!(response.contains("role:master"));
        assert!(response.contains("master_replid:"));
        assert!(response.contains("master_repl_offset:0"));
    }

    #[tokio::test]
    async fn test_command_replica() {
        let config = ReplicationConfig::new_replica(String::from("someserver"), 4242);
        let mut server: Server = Server::new();
        server.set_replication(config);

        let cmd = vec![String::from("info")];

        let (request_channel_tx, mut request_channel_rx) = mpsc::channel::<ServerMessage>(32);

        let request = Request {
            value: RESP::Null,
            sender: request_channel_tx.clone(),
        };

        command(&mut server, &request, &cmd).await;

        let response = match request_channel_rx.try_recv().unwrap() {
            ServerMessage::Data(ServerValue::RESP(RESP::BulkString(value))) => value,
            _ => panic!(),
        };

        assert!(response.contains("# Replication"));
        assert!(response.contains("role:slave"));
        assert!(response.contains("master_repl_offset:0"));
    }
}
