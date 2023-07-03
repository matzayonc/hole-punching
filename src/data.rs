use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc::Sender, task::JoinHandle};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Register {
        name: String,
    },
    Ping {
        name: String,
    },
    ConnectionRequest {
        from: String,
        to: String,
        peer_type: PeerType,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagesFromServer {
    Pong {
        name: String,
    },
    RegisterConfirmation {
        name: String,
    },
    ConnectionRequest {
        name: String,
        address: String,
        peer_type: PeerType,
    },
    Confirm {
        name: String,
    },
    Reject {
        name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerMessage {
    Ping,
    Pong,
    Init,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerType {
    Full,
    Passive,
}

#[derive(Debug, Clone)]
pub struct Peer {
    pub name: String,
    pub address: SocketAddr,
    pub peer_type: PeerType,
}

pub struct PeerConnection {
    pub peer: Peer,
    pub handle: JoinHandle<()>,
    pub sender: Sender<()>,
}
