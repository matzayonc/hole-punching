use bincode::serialize;
use log::{debug, info};
use std::{collections::HashMap, net::SocketAddr};
use tokio::net::UdpSocket;

use crate::{MessagesFromServer, ServerMessage};

pub struct RandevousServer {
    socket: UdpSocket,
}

impl RandevousServer {
    async fn new(listener: SocketAddr) -> RandevousServer {
        let socket = UdpSocket::bind(&listener)
            .await
            .expect("Failed to bind UDP socket");

        info!("Listening for UDP packets on {}", listener);

        RandevousServer { socket }
    }

    async fn run(self) {
        let mut buffer = [0u8; 1024];
        let mut waiting = HashMap::new();

        loop {
            let (n, peer_address) = self.socket.recv_from(&mut buffer).await.unwrap();

            let Ok(message) = bincode::deserialize::<ServerMessage>(&buffer[..n]) else {
                let message = String::from_utf8_lossy(&buffer[..n]);
                debug!("Received {} bytes from {}: {}", n, peer_address, message);
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
        info!("Received {:?} from {}", message, peer_address);
        match message {
            ServerMessage::Register { name } => {
                let address = peer_address.to_string();
                waiting.insert(name.clone(), address);
                let response = serialize(&MessagesFromServer::RegisterConfirmation { name })
                    .expect("Failed to serialize message");
                self.socket.send_to(&response, &peer_address).await.unwrap();
            }
            ServerMessage::Ping { name } => {
                let response = serialize(&MessagesFromServer::Pong { name })
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
                                debug!("Peer address for {} invalid: {}", to, target_address);
                                return;
                            }
                        };

                        let request = serialize(&MessagesFromServer::ConnectionRequest {
                            name: from.clone(),
                            address: peer_address.to_string(),
                        })
                        .expect("Failed to serialize message");
                        self.socket
                            .send_to(&request, &target_address)
                            .await
                            .expect("Failed to send UDP packet");

                        info!("Forwarded connection request to {}", target_address);

                        serialize(&MessagesFromServer::Confirm {
                            name: from.clone(),
                            address: target_address.to_string(),
                        })
                        .expect("Failed to serialize message")
                    }
                    None => serialize(&MessagesFromServer::Reject { name: from.clone() })
                        .expect("Failed to serialize message"),
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
