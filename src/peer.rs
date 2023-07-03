use crate::{data::Peer, PeerMessage};
use bincode::{deserialize, serialize};
use log::{debug, info};
use std::net::SocketAddr;
use tokio::time::Duration;
use tokio::{net::UdpSocket, sync::mpsc::Receiver};

pub async fn connect_peers(peer: Peer, mut rx: Receiver<()>, punched: Option<UdpSocket>) {
    info!("Connecting to peer {}", peer.name);
    let interval = if punched.is_some() { 5 } else { 7 };
    let mut interval = tokio::time::interval(Duration::from_secs(interval));

    let client_address: SocketAddr = "0.0.0.0:0".parse().expect("Local address invalid"); // Let the OS assign a random available port for the client
    let socket = if let Some(s) = punched {
        s
    } else {
        UdpSocket::bind(&client_address)
            .await
            .expect("Failed to bind UDP socket")
    };

    let mut buffer = [0u8; 1024];

    let message = serialize(&PeerMessage::Discover {
        name: "Hans".to_string(),
    })
    .expect("Failed to serialize message");
    socket
        .send_to(&message, &peer.address)
        .await
        .expect("Failed to send UDP packet");

    loop {
        tokio::select! {
            _ = interval.tick() => {
                debug!("Sending ping to peer {}", peer.name);
                let message = serialize(&PeerMessage::Ping).expect("Failed to serialize message");
                socket
                .send_to(&message, &peer.address)
                .await
                .expect("Failed to send UDP packet");
            },
            v = rx.recv() => {
                if let Some(_) = v {
                    info!("Received message from system {}", peer.name);
                } else {
                    info!("Connection with peer {} closed by force", peer.name);
                }
                println!("Received message from system {}", peer.name);
            },
            v = socket.recv_from(&mut buffer) => {
                    let (n, peer_address) = v.expect("Failed to receive UDP packet");

                    if peer_address != peer.address {
                        debug!("Received message from unknown peer {}", peer_address);
                        continue;
                    }

                    let message = if let Ok(message) = deserialize::<PeerMessage>(&buffer[..n]) {
                        message
                    } else {
                        let message = String::from_utf8_lossy(&buffer[..n]);
                        debug!("Received invalid message from peer {}: {}", peer.name, message);
                        continue;
                    };

                    match message {
                        PeerMessage::Ping  => {
                            debug!("Received ping from peer {}", peer.name);
                            let response = serialize(&PeerMessage::Pong).expect("Failed to serialize message");
                            socket
                            .send_to(&response, &peer.address)
                            .await
                            .expect("Failed to send UDP packet");

                        }
                        PeerMessage::Pong => {
                            debug!("Received pong from peer {}", peer.name);
                            interval.reset();
                        }
                        _ => {
                            let message = String::from_utf8_lossy(&buffer[..n]);
                            debug!("Received invalid message from peer {}: {}", peer.name, message);
                        }
                }
            }
        }
    }
}
