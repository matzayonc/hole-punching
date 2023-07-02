use dotenv::dotenv;
use futures_util::SinkExt;
use http::Uri;
use std::env;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use tokio::net::{TcpListener, UdpSocket};
use tokio::task;
use tokio::time::{sleep, Duration};
use tokio_websockets::{ClientBuilder, Error, Message, ServerBuilder};

const SERVER: &str = "matzayonc.pl:1234";

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

    let receive_task = task::spawn(async move {
        let socket = match UdpSocket::bind(&client_address).await {
            Ok(s) => s,
            Err(e) => {
                println!("Failed to bind UDP socket to {}: {}", client_address, e);
                return;
            }
        };

        let message = format!("Hello, UDP packet {}", 1).into_bytes();
        match socket.send_to(&message, &server_address).await {
            Ok(n) => println!("Sent {} bytes to {}", n, server_address),
            Err(e) => println!("Failed to send UDP packet: {}", e),
        };
        println!("Listening for UDP packets on {}", client_address);

        let mut buffer = [0u8; 1024];
        loop {
            let (n, peer_address) = socket.recv_from(&mut buffer).await.unwrap();
            let received_data = &buffer[..n];
            let message = String::from_utf8_lossy(received_data);
            println!("Received UDP packet from {}: {}", peer_address, message);
        }
    });

    receive_task.await.unwrap();

    Ok(())
}
