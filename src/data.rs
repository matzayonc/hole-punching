use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use crate::peer::Peer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Register { name: String },
    Ping { name: String },
    ConnectionRequest { from: String, to: String },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagesFromServer {
    Pong { name: String },
    RegisterConfirmation { name: String },
    ConnectionRequest { name: String, address: String },
    Confirm { name: String, address: String },
    Reject { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerMessage {
    Introduce { source: String, expected: String },
    Ping,
    Pong,
    Init,
}

pub struct PeerConnection {
    pub peer: Peer,
    pub handle: JoinHandle<()>,
    pub sender: Sender<()>,
}
