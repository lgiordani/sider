use crate::resp::RESP;
use crate::storage::Storage;
use connection::{run_listener, ConnectionMessage};
use server::{run_server, Server};
use tokio::sync::mpsc;

mod connection;
mod request;
mod resp;
mod resp_result;
mod server;
mod server_result;
mod set;
mod storage;
mod storage_result;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let storage = Storage::new();
    let mut server = Server::new();
    server = server.set_storage(storage);

    let (server_sender, server_receiver) = mpsc::channel::<ConnectionMessage>(32);

    tokio::spawn(run_server(server, server_receiver));

    run_listener("127.0.0.1".to_string(), 6379, server_sender).await;

    Ok(())
}
