use bincode::serde::{decode_from_slice, encode_to_vec};
use std::{collections::HashMap, net::SocketAddr};
use tokio::net::UdpSocket;
use tracing::{debug, info, span, Level};

use crate::{MessagesFromServer, ServerMessage, ENCODING};

pub struct RandevousServer {
    socket: UdpSocket,
}

impl RandevousServer {
    async fn new(listener: SocketAddr) -> RandevousServer {
        let socket = UdpSocket::bind(&listener)
            .await
            .expect("Failed to bind UDP socket");

        info!(%listener, "Listening for UDP packets");
        RandevousServer { socket }
    }

    async fn run(self) {
        let mut buffer = [0u8; 1024];
        let mut waiting = HashMap::new();

        loop {
            let (n, peer_address) = self.socket.recv_from(&mut buffer).await.unwrap();

            let Ok((message, _)) = decode_from_slice(&buffer[..n], ENCODING) else {
                let message = String::from_utf8_lossy(&buffer[..n]);
                debug!(bytes = n, from = %peer_address, message = %message, "Received non-bincode packet");
                continue;
            };

            self.process(peer_address, message, &mut waiting).await;
        }
    }

    async fn process(
        &self,
        peer_address: SocketAddr,
        message: ServerMessage,
        waiting: &mut HashMap<String, String>,
    ) {
        let span = span!(Level::INFO, "process_server_message", peer = %peer_address);
        let _enter = span.enter();
        info!(message = ?message, "Processing server message");
        match message {
            ServerMessage::Register { name } => {
                let address = peer_address.to_string();
                waiting.insert(name.clone(), address);
                let response = encode_to_vec(
                    &MessagesFromServer::RegisterConfirmation { name },
                    bincode::config::standard(),
                )
                .expect("Failed to serialize message");
                self.socket.send_to(&response, &peer_address).await.unwrap();
            }
            ServerMessage::Ping { name } => {
                let response = encode_to_vec(&MessagesFromServer::Pong { name }, ENCODING)
                    .expect("Failed to serialize message");
                self.socket
                    .send_to(&response, &peer_address)
                    .await
                    .expect("Failed to send UDP packet");
            }
            ServerMessage::ConnectionRequest { from, to } => {
                let control_message = match waiting.get(&to) {
                    Some(target_address) => {
                        let target_address: SocketAddr = match target_address.parse() {
                            Ok(s) => s,
                            Err(_) => {
                                debug!(%to, raw_address = %target_address, "Peer address invalid");
                                return;
                            }
                        };

                        let request = encode_to_vec(
                            &MessagesFromServer::ConnectionRequest {
                                name: from.clone(),
                                address: peer_address.to_string(),
                            },
                            ENCODING,
                        )
                        .expect("Failed to serialize message");
                        self.socket
                            .send_to(&request, &target_address)
                            .await
                            .expect("Failed to send UDP packet");

                        info!(to = %to, target = %target_address, "Forwarded connection request");

                        encode_to_vec(
                            &MessagesFromServer::Confirm {
                                name: from.clone(),
                                address: target_address.to_string(),
                            },
                            ENCODING,
                        )
                        .expect("Failed to serialize message")
                    }
                    None => {
                        encode_to_vec(&MessagesFromServer::Reject { name: from.clone() }, ENCODING)
                            .expect("Failed to serialize message")
                    }
                };

                if let Some(from_control_address) = waiting.get(&from) {
                    self.socket
                        .send_to(&control_message, &from_control_address)
                        .await
                        .expect("Failed to send UDP packet");
                };
            }
        };
    }

    pub async fn serve(listener: SocketAddr) {
        let server = Self::new(listener).await;
        server.run().await;
    }
}
