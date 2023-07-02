use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Register {
        name: String,
        address: String,
    },
    Ping {
        name: String,
    },
    Pong {},
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
