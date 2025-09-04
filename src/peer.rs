use crate::{PeerMessage, ENCODING};
use bincode::serde::{decode_from_slice, encode_to_vec};
use std::net::SocketAddr;
use tokio::time::{Duration, Interval};
use tokio::{net::UdpSocket, sync::mpsc::Receiver};
use tracing::{debug, info, span, Level};

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
        info!(peer = %peer.name, "Connecting to peer");
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

        let message = encode_to_vec(
            &PeerMessage::Introduce {
                source: own_name,
                expected: peer.name,
            },
            ENCODING,
        )
        .expect("Serializing introduce message expected to succeed.");

        socket
            .send_to(&message, &peer.address)
            .await
            .expect("Failed to send UDP packet");

        socket
    }

    pub async fn listen(mut self, mut rx: Receiver<()>) {
        let mut buffer = [0u8; 1024];

        let message = encode_to_vec(
            &PeerMessage::Introduce {
                source: self.own_name.clone(),
                expected: self.peer.name.clone(),
            },
            ENCODING,
        )
        .expect("Failed to serialize message");
        self.socket
            .send_to(&message, &self.peer.address)
            .await
            .expect("Failed to send UDP packet");

        loop {
            tokio::select! {
                _ = self.interval.tick() => {
                    debug!(peer = %self.peer.name, "Sending ping to peer");
                    let message = encode_to_vec(&PeerMessage::Ping, ENCODING).expect("Failed to serialize message");
                    self.socket
                    .send_to(&message, &self.peer.address)
                    .await
                    .expect("Failed to send UDP packet");
                },
                v = rx.recv() => {
                    if let Some(_) = v {
                        info!(peer = %self.peer.name, "Received message from system");
                    } else {
                        info!(peer = %self.peer.name, "Connection with peer closed by force");
                    }
                    debug!(peer = %self.peer.name, "rx.recv returned");
                },
                v = self.socket.recv_from(&mut buffer) => {
                    let (n, peer_address) = v.expect("Failed to receive UDP packet");

                    if &peer_address != &self.peer.address {
                        debug!(from = %peer_address, "Received message from unknown peer");
                        continue;
                    }

                    let (message, _) = if let Ok(message) = decode_from_slice(&buffer[..n], ENCODING) {
                        message
                    } else {
                        let message = String::from_utf8_lossy(&buffer[..n]);
                        debug!(peer = %self.peer.name, message = %message, "Received invalid message from peer");
                        continue;
                    };

                    let s = span!(Level::INFO, "process_peer_message", peer = %self.peer.name);
                    let _ent = s.enter();
                    self.process(message).await;
                }
            }
        }
    }

    async fn process(&mut self, message: PeerMessage) {
        match message {
            PeerMessage::Ping => {
                debug!(peer = %self.peer.name, "Received ping from peer");
                let response = encode_to_vec(&PeerMessage::Pong, ENCODING)
                    .expect("Failed to serialize message");
                self.socket
                    .send_to(&response, &self.peer.address)
                    .await
                    .expect("Failed to send UDP packet");
            }
            PeerMessage::Pong => {
                debug!(peer = %self.peer.name, "Received pong from peer");
                self.interval.reset();
            }
            unexpected_message => {
                debug!(peer = %self.peer.name, message = ?unexpected_message, "Received invalid message from peer");
            }
        }
    }
}
