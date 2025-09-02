use bincode::{deserialize, serialize};
use log::{debug, info};
use std::{collections::HashMap, net::SocketAddr};
use tokio::{net::UdpSocket, task};

use crate::{peer::Peer, MessagesFromServer, PeerListeners, PeerMessage, ServerMessage};

pub async fn demo(
    client: SocketAddr,
    server: SocketAddr,
    peers: PeerListeners,
    name: String,
    target: String,
) {
    let receive_task: task::JoinHandle<()> = task::spawn(async move {
        let socket = UdpSocket::bind(&client)
            .await
            .expect("Failed to bind UDP socket");

        let message = serialize(&ServerMessage::Register { name: name.clone() })
            .expect("Failed to serialize message");

        match socket.send_to(&message, &server).await {
            Ok(n) => println!("Sent {} bytes to {}", n, server),
            Err(e) => println!("Failed to send UDP packet: {}", e),
        };
        info!("Listening for UDP packets on {}", client);

        let other_socket: UdpSocket = UdpSocket::bind(&client)
            .await
            .expect("Failed to bind UDP socket");
        tokio::spawn(connect(
            other_socket,
            server,
            peers.clone(),
            name.clone(),
            target.clone(),
        ));

        let mut buffer: [u8; 1024] = [0u8; 1024];
        loop {
            let (n, peer_address) = socket.recv_from(&mut buffer).await.unwrap();
            let message = deserialize::<MessagesFromServer>(&buffer[..n])
                .expect("Failed to deserialize message");

            match message {
                MessagesFromServer::ConnectionRequest {
                    name: peer_name,
                    address,
                } => {
                    info!("Peer {} at {} wants to connect", peer_name, address);
                    let peer = Peer {
                        name: peer_name.clone(),
                        address: address.parse().expect("Peer address invalid"),
                    };
                    peers.clone().add_peer_task(name.clone(), peer, None);
                }
                _ => {
                    let received_data = &buffer[..n];
                    let message = String::from_utf8_lossy(received_data);
                    debug!("Received {} bytes from {}: {}", n, peer_address, message);
                }
            }
        }
    });

    receive_task.await.unwrap();
}

async fn connect(
    socket: UdpSocket,
    server: SocketAddr,
    peers: PeerListeners,
    own_name: String,
    target: String,
) -> Result<(), ()> {
    let message = serialize(&ServerMessage::ConnectionRequest {
        from: own_name.clone(),
        to: target.clone(),
    });

    socket
        .send_to(&message.unwrap(), &server)
        .await
        .expect("Failed to send UDP packet");

    let mut buffer: [u8; 1024] = [0u8; 1024];
    loop {
        let (n, peer_address) = socket.recv_from(&mut buffer).await.unwrap();
        let message =
            deserialize::<PeerMessage>(&buffer[..n]).expect("Failed to deserialize message");

        match message {
            PeerMessage::Introduce { source, expected } => {
                info!("Peer {} at {} opened connection", source, peer_address);
                assert_eq!(own_name, expected);
                let peer = Peer {
                    name: source.clone(),
                    address: peer_address,
                };
                peers.clone().add_peer_task(own_name, peer, Some(socket));
                return Ok(());
            }
            _ => {
                let received_data = &buffer[..n];
                let message = String::from_utf8_lossy(received_data);
                debug!(
                    "Received {} bytes from unknown peer {}: {}",
                    n, peer_address, message
                );
            }
        }
    }
}

pub async fn serve(listener: SocketAddr) {
    let receive_task = task::spawn(async move {
        let socket = UdpSocket::bind(&listener)
            .await
            .expect("Failed to bind UDP socket");

        info!("Listening for UDP packets on {}", listener);
        let mut waiting = HashMap::new();

        let mut buffer = [0u8; 1024];
        loop {
            let (n, peer_address) = socket.recv_from(&mut buffer).await.unwrap();

            let message = if let Ok(m) = bincode::deserialize::<ServerMessage>(&buffer[..n]) {
                m
            } else {
                let message = String::from_utf8_lossy(&buffer[..n]);
                debug!("Received {} bytes from {}: {}", n, peer_address, message);
                continue;
            };

            match message {
                ServerMessage::Register { name } => {
                    let address = peer_address.to_string();
                    waiting.insert(name.clone(), address);
                    let response = serialize(&MessagesFromServer::RegisterConfirmation { name })
                        .expect("Failed to serialize message");
                    socket.send_to(&response, &peer_address).await.unwrap();
                }
                ServerMessage::Ping { name } => {
                    let response = serialize(&MessagesFromServer::Pong { name })
                        .expect("Failed to serialize message");
                    socket
                        .send_to(&response, &peer_address)
                        .await
                        .expect("Failed to send UDP packet");
                }
                ServerMessage::ConnectionRequest { from, to } => {
                    let control_message = match waiting.get(&to) {
                        Some(target_address) => {
                            let target_address: SocketAddr = match target_address.parse() {
                                Ok(address) => address,
                                Err(_) => {
                                    debug!("Peer address for {} invalid: {}", to, target_address);
                                    continue;
                                }
                            };

                            let request = serialize(&MessagesFromServer::ConnectionRequest {
                                name: from.clone(),
                                address: peer_address.to_string(),
                            })
                            .expect("Failed to serialize message");
                            socket
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
                        socket
                            .send_to(&control_message, &from_control_address)
                            .await
                            .expect("Failed to send UDP packet");
                    };
                }
            };
        }
    });

    receive_task.await.unwrap();
}
