use clap::Parser;
use log::info;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio_websockets::Error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Rendezvous server address
    #[arg(short, long)]
    server: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let args = Args::parse();

    let server_address: SocketAddr = args
        .server
        .to_socket_addrs()
        .expect("DNS resolution failed")
        .next()
        .expect("No DNS records found for hostname");

    info!("Starting rendezvous server on {}", server_address);

    // As a server wait for a peer to connect
    hole::randevous::RandevousServer::serve(server_address).await;

    Ok(())
}
