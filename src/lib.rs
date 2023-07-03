use bincode::{deserialize, serialize};
use dashmap::DashMap;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    net::UdpSocket,
    sync::mpsc::{self, Receiver},
    task,
    time::sleep,
};

use crate::peer::connect_peers;

mod data;
mod peer;

pub use data::*;
pub type Peers = Arc<DashMap<String, PeerConnection>>;

fn add_peer_task(peer: Peer, peers: Peers) {
    let (tx, rx) = mpsc::channel::<()>(32);

    let handle = tokio::spawn(connect_peers(peer.clone(), rx));
    let peer_connection = PeerConnection {
        peer,
        handle,
        sender: tx,
    };
    peers.insert(peer_connection.peer.name.clone(), peer_connection);
}

pub async fn demo(client: SocketAddr, server: SocketAddr, peers: Peers) {
    let receive_task: task::JoinHandle<()> = task::spawn(async move {
        let socket = UdpSocket::bind(&client)
            .await
            .expect("Failed to bind UDP socket");

        let message = serialize(&ServerMessage::Register {
            name: "Alice".to_string(),
        })
        .expect("Failed to serialize message");

        match socket.send_to(&message, &server).await {
            Ok(n) => println!("Sent {} bytes to {}", n, server),
            Err(e) => println!("Failed to send UDP packet: {}", e),
        };
        println!("Listening for UDP packets on {}", client);

        let other_socket: UdpSocket = UdpSocket::bind(&client)
            .await
            .expect("Failed to bind UDP socket");
        tokio::spawn(connect(other_socket, server, peers.clone()));

        let mut buffer: [u8; 1024] = [0u8; 1024];
        loop {
            let (n, peer_address) = socket.recv_from(&mut buffer).await.unwrap();
            let message = deserialize::<MessagesFromServer>(&buffer[..n])
                .expect("Failed to deserialize message");

            match message {
                MessagesFromServer::ConnectionRequest {
                    name,
                    address,
                    peer_type,
                } => {
                    let peer = Peer {
                        name: name.clone(),
                        address: address.parse().expect("Peer address invalid"),
                        peer_type,
                    };
                    add_peer_task(peer, peers.clone());
                }
                _ => {
                    let received_data = &buffer[..n];
                    let message = String::from_utf8_lossy(received_data);
                    println!("Received UDP packet from {}: {}", peer_address, message);
                }
            }
        }
    });

    receive_task.await.unwrap();
}

pub async fn connect(socket: UdpSocket, server: SocketAddr, peers: Peers) -> Result<(), ()> {
    let peer_type = PeerType::Full;
    let message = serialize(&ServerMessage::ConnectionRequest {
        from: "Bob".to_string(),
        to: "Alice".to_string(),
        peer_type: peer_type.clone(),
    });

    match socket.send_to(&message.unwrap(), &server).await {
        Ok(n) => println!("Sent {} bytes to {}", n, server),
        Err(e) => eprintln!("Failed to send UDP packet: {}", e),
    };

    let mut buffer: [u8; 1024] = [0u8; 1024];
    loop {
        let (n, peer_address) = socket.recv_from(&mut buffer).await.unwrap();
        let message =
            deserialize::<MessagesFromServer>(&buffer[..n]).expect("Failed to deserialize message");

        match message {
            MessagesFromServer::Confirm { name, address } => {
                println!("Received confirmation from {} at {}", name, address);
                let peer = Peer {
                    name: name.clone(),
                    address: address.parse().expect("Peer address invalid"),
                    peer_type: peer_type.clone(),
                };
                add_peer_task(peer, peers);
                return Ok(());
            }
            MessagesFromServer::Reject { name } => {
                println!("Connection request rejected by {}", name);
                return Err(());
            }
            _ => {
                let received_data = &buffer[..n];
                let message = String::from_utf8_lossy(received_data);
                println!("Received UDP packet from {}: {}", peer_address, message);
            }
        }
    }
}

pub async fn serve(listener: SocketAddr) {
    let receive_task = task::spawn(async move {
        let socket = UdpSocket::bind(&listener)
            .await
            .expect("Failed to bind UDP socket");

        println!("Listening for UDP packets on {}", listener);
        let mut waiting = HashMap::new();

        let mut buffer = [0u8; 1024];
        loop {
            let (n, peer_address) = socket.recv_from(&mut buffer).await.unwrap();

            let message = bincode::deserialize::<ServerMessage>(&buffer[..n])
                .expect("Failed to deserialize message");

            let response = match message {
                ServerMessage::Register { name } => {
                    let address = peer_address.to_string();
                    waiting.insert(name.clone(), address);
                    serialize(&MessagesFromServer::RegisterConfirmation { name })
                        .expect("Failed to serialize message")
                }
                ServerMessage::Ping { name } => serialize(&MessagesFromServer::Pong { name })
                    .expect("Failed to serialize message"),
                ServerMessage::ConnectionRequest {
                    from,
                    to,
                    peer_type,
                } => match waiting.get(&to) {
                    Some(peer_address) => {
                        let peer_address: SocketAddr =
                            peer_address.parse().expect("Peer address invalid");

                        let request = serialize(&MessagesFromServer::ConnectionRequest {
                            name: from.clone(),
                            address: peer_address.to_string(),
                            peer_type,
                        })
                        .expect("Failed to serialize message");
                        socket
                            .send_to(&request, &peer_address)
                            .await
                            .expect("Failed to send UDP packet");

                        serialize(&MessagesFromServer::Confirm {
                            name: from.clone(),
                            address: peer_address.to_string(),
                        })
                        .expect("Failed to serialize message")
                    }
                    None => serialize(&MessagesFromServer::Reject { name: from })
                        .expect("Failed to serialize message"),
                },
            };

            let message = String::from_utf8_lossy(&buffer[..n]);
            println!("Received UDP packet from {}: {}", peer_address, message);
            socket.send_to(&response, &peer_address).await.unwrap();
            sleep(Duration::from_millis(500)).await;
        }
    });

    receive_task.await.unwrap();
}
