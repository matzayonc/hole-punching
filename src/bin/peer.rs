use clap::Parser;
use hole::PeerListeners;
use log::info;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio_websockets::Error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the peer
    #[arg(short, long)]
    name: String,

    /// Name of the target peer
    #[arg(short, long)]
    target: String,

    /// Rendezvous server address
    #[arg(short, long)]
    server: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let args = Args::parse();

    let client_address: SocketAddr = "0.0.0.0:0".parse().expect("Local address invalid"); // Let the OS assign a random available port for the client
    let server_address: SocketAddr = args
        .server
        .to_socket_addrs()
        .expect("DNS resolution failed")
        .next()
        .expect("No DNS records found for hostname");

    info!("Starting peer application with server {}", server_address);

    let peers = PeerListeners::new();
    // As a peer contact rendezvous server and wait for a peer to connect
    hole::demo::demo(
        client_address,
        server_address,
        peers,
        args.name,
        args.target,
    )
    .await;

    Ok(())
}
