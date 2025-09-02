use crate::PeerMessage;
use bincode::{deserialize, serialize};
use log::{debug, info};
use std::net::SocketAddr;
use tokio::time::{Duration, Interval};
use tokio::{net::UdpSocket, sync::mpsc::Receiver};

#[derive(Debug, Clone)]
pub struct Peer {
    pub name: String,
    pub address: SocketAddr,
}

pub struct PeerNode {
    socket: UdpSocket,
    interval: Interval,
    peer: Peer,
    own_name: String,
}

impl PeerNode {
    pub async fn new(own_name: String, peer: Peer, punched: Option<UdpSocket>) -> PeerNode {
        info!("Connecting to peer {}", peer.name);
        let interval = if punched.is_some() { 5 } else { 7 };
        let interval = tokio::time::interval(Duration::from_secs(interval));

        assert_ne!(own_name, peer.name, "Cannot connect to self");

        let socket_to_peer = if let Some(s) = punched {
            s
        } else {
            Self::initialize_connection(own_name.clone(), peer.clone()).await
        };

        PeerNode {
            socket: socket_to_peer,
            interval,
            peer,
            own_name,
        }
    }

    async fn initialize_connection(own_name: String, peer: Peer) -> UdpSocket {
        let client_address: SocketAddr = "0.0.0.0:0".parse().expect("Local address invalid"); // Let the OS assign a random available port for the client

        let socket = UdpSocket::bind(&client_address)
            .await
            .expect("Failed to bind UDP socket");

        let message = serialize(&PeerMessage::Introduce {
            source: own_name,
            expected: peer.name,
        })
        .expect("Serializing introduce message expected to succeed.");

        socket
            .send_to(&message, &peer.address)
            .await
            .expect("Failed to send UDP packet");

        socket
    }

    pub async fn listen(mut self, mut rx: Receiver<()>) {
        let mut buffer = [0u8; 1024];

        let message = serialize(&PeerMessage::Introduce {
            source: self.own_name.clone(),
            expected: self.peer.name.clone(),
        })
        .expect("Failed to serialize message");
        self.socket
            .send_to(&message, &self.peer.address)
            .await
            .expect("Failed to send UDP packet");

        loop {
            tokio::select! {
                _ = self.interval.tick() => {
                    debug!("Sending ping to peer {}", self.peer.name);
                    let message = serialize(&PeerMessage::Ping).expect("Failed to serialize message");
                    self.socket
                    .send_to(&message, &self.peer.address)
                    .await
                    .expect("Failed to send UDP packet");
                },
                v = rx.recv() => {
                    if let Some(_) = v {
                        info!("Received message from system {}", self.peer.name);
                    } else {
                        info!("Connection with peer {} closed by force", self.peer.name);
                    }
                    println!("Received message from system {}", self.peer.name);
                },
                v = self.socket.recv_from(&mut buffer) => {
                    let (n, peer_address) = v.expect("Failed to receive UDP packet");

                    if &peer_address != &self.peer.address {
                        debug!("Received message from unknown peer {}", peer_address);
                        continue;
                    }

                    let message = if let Ok(message) = deserialize::<PeerMessage>(&buffer[..n]) {
                        message
                    } else {
                        let message = String::from_utf8_lossy(&buffer[..n]);
                        debug!("Received invalid message from peer {}: {}", self.peer.name, message);
                        continue;
                    };

                    self.process(message).await;
                }
            }
        }
    }

    async fn process(&mut self, message: PeerMessage) {
        match message {
            PeerMessage::Ping => {
                debug!("Received ping from peer {}", self.peer.name);
                let response = serialize(&PeerMessage::Pong).expect("Failed to serialize message");
                self.socket
                    .send_to(&response, &self.peer.address)
                    .await
                    .expect("Failed to send UDP packet");
            }
            PeerMessage::Pong => {
                debug!("Received pong from peer {}", self.peer.name);
                self.interval.reset();
            }
            unexpected_message => {
                debug!(
                    "Received invalid message from peer {}: {unexpected_message:?}",
                    self.peer.name,
                );
            }
        }
    }
}
