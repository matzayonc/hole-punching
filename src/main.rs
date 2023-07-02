use dotenv::dotenv;
use hole::{demo, serve};
use std::env;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio_websockets::Error;

#[cfg(not(any(feature = "peer", feature = "server")))]
compile_error!("Either feature \"peer\" or \"server\" must be enabled for this crate.");

#[cfg(all(feature = "peer", feature = "server"))]
compile_error!("Features \"peer\" and \"server\" can't be enabled at the same time.");

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let client_address: SocketAddr = "0.0.0.0:0".parse().expect("Local address invalid"); // Let the OS assign a random available port for the client
    let server_address: SocketAddr = env::var("RENDEZVOUS_SERVER")
        .expect("RENDEZVOUS_SERVER environment variable not set")
        .to_socket_addrs()
        .expect("DNS resolution failed")
        .next()
        .expect("No DNS records found for hostname");

    // As a peer contact rendezvous server and wait for a peer to connect
    #[cfg(feature = "peer")]
    demo(client_address, server_address).await;

    // As a server wait for a peer to connect
    #[cfg(feature = "server")]
    serve(server_address, client_address).await;

    Ok(())
}
