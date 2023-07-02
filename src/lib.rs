use bincode::serialize;
use std::{collections::HashMap, net::SocketAddr, time::Duration};
use tokio::{net::UdpSocket, task, time::sleep};

use crate::data::{MessagesFromServer, PeerType, ServerMessage};

mod data;

pub async fn demo(client: SocketAddr, server: SocketAddr) {
    let receive_task: task::JoinHandle<()> = task::spawn(async move {
        let socket = UdpSocket::bind(&client)
            .await
            .expect("Failed to bind UDP socket");

        let message = serialize(&ServerMessage::Register {
            name: "Alice".to_string(),
            address: client.ip().to_string(),
        })
        .expect("Failed to serialize message");

        match socket.send_to(&message, &server).await {
            Ok(n) => println!("Sent {} bytes to {}", n, server),
            Err(e) => println!("Failed to send UDP packet: {}", e),
        };
        println!("Listening for UDP packets on {}", client);

        let mut buffer = [0u8; 1024];
        loop {
            let (n, peer_address) = socket.recv_from(&mut buffer).await.unwrap();
            let received_data = &buffer[..n];
            let message = String::from_utf8_lossy(received_data);
            println!("Received UDP packet from {}: {}", peer_address, message);
        }
    });

    receive_task.await.unwrap();
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
                ServerMessage::Register { name, address } => {
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

                        serialize(&MessagesFromServer::Confirm { name: from.clone() })
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
