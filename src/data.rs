use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Register { name: String, address: String },
    Ping { name: String },
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerType {
    Full,
    Passive,
}
