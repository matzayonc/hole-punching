use std::{net::SocketAddr, time::Duration};
use tokio::{net::UdpSocket, task, time::sleep};

pub async fn demo(client: SocketAddr, server: SocketAddr) {
    let receive_task: task::JoinHandle<()> = task::spawn(async move {
        let socket = match UdpSocket::bind(&client).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to bind UDP socket to {}: {}", client, e);
                return;
            }
        };

        let message = format!("Hello, UDP packet {}", 1).into_bytes();
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

pub async fn serve(local_address: SocketAddr) {
    let receive_task = task::spawn(async move {
        let socket = match UdpSocket::bind(&local_address).await {
            Ok(s) => s,
            Err(e) => {
                println!("Failed to bind UDP socket to {}: {}", local_address, e);
                return;
            }
        };

        println!("Listening for UDP packets on {}", local_address);

        let mut buffer = [0u8; 1024];
        loop {
            let (n, peer_address) = socket.recv_from(&mut buffer).await.unwrap();

            let received_data = &buffer[..n];
            let message = String::from_utf8_lossy(received_data);
            println!("Received UDP packet from {}: {}", peer_address, message);
            let response = format!("Hello, UDP packet {}", 42).into_bytes();
            let socket = UdpSocket::bind(&peer_address).await.unwrap();
            socket.send_to(&response, &peer_address).await.unwrap();
            sleep(Duration::from_millis(500)).await;
        }
    });

    receive_task.await.unwrap();
}
