use crate::replication::ReplicationConfig;
use crate::resp::RESP;
use crate::storage::Storage;
use clap::Parser;
use connection::{run_listener, ConnectionMessage};
use server::{run_server, Server};
use tokio::sync::mpsc;

mod commands;

mod connection;
mod replication;
mod request;
mod resp;
mod resp_result;
mod server;
mod server_result;
mod set;
mod storage;
mod storage_result;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "The TCP port to use for the server",
        default_value_t = 6379
    )]
    port: u16,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let replication_config = ReplicationConfig::new_master();

    let mut storage = Storage::new();
    storage.set_active_expiry(true);

    let mut server = Server::new();
    server.set_storage(storage);
    server.set_replication(replication_config);

    let (server_sender, server_receiver) = mpsc::channel::<ConnectionMessage>(32);

    tokio::spawn(run_server(server, server_receiver));

    run_listener("127.0.0.1".to_string(), args.port, server_sender).await;

    Ok(())
}
