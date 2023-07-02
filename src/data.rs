use serde::{Deserialize, Serialize};

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
pub enum PeerType {
    Full,
    Passive,
}
