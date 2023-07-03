use bincode::serialize;
use std::net::SocketAddr;
use tokio::{net::UdpSocket, sync::mpsc::Receiver};

use crate::{data::Peer, PeerMessage};

pub async fn connect_peers(peer: Peer, mut rx: Receiver<()>) {
    println!("Connecting to peer {}", peer.name);
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    let client_address: SocketAddr = "0.0.0.0:0".parse().expect("Local address invalid"); // Let the OS assign a random available port for the client
    let socket = UdpSocket::bind(&client_address)
        .await
        .expect("Failed to bind UDP socket");

    let mut buffer = [0u8; 1024];

    let message = serialize(&PeerMessage::Init).expect("Failed to serialize message");
    socket
        .send_to(&message, &peer.address)
        .await
        .expect("Failed to send UDP packet");

    loop {
        tokio::select! {
            // _ = interval.tick() => {
            //     println!("Sending ping to peer {}", peer.name);
            //     socket.send_to(b"Ping", &peer.address).await.expect("Failed to send UDP packet");
            // },
            // _ = rx.recv() => {
            //     println!("Received message from system {}", peer.name);
            // },
            _ = socket.recv_from(&mut buffer) => {
                println!("Received UDP packet from peer {}", peer.name);
            },
        }
    }
}
