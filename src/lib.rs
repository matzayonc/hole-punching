use dashmap::DashMap;
use std::sync::Arc;
use tokio::{net::UdpSocket, sync::mpsc};

use crate::peer::{Peer, PeerNode};

mod data;
pub mod demo;
mod peer;
pub mod randevous;

pub use data::*;

#[derive(Clone)]
pub struct PeerListeners(Arc<DashMap<String, PeerConnection>>);

impl PeerListeners {
    pub fn new() -> Self {
        PeerListeners(Arc::new(DashMap::new()))
    }

    fn add_peer_task(self, own_name: String, peer: Peer, punched: Option<UdpSocket>) {
        let (tx, rx) = mpsc::channel::<()>(32);

        let peer_clone = peer.clone();
        let handle = tokio::spawn(async move {
            let peer_node = PeerNode::new(own_name, peer_clone, punched).await;
            peer_node.listen(rx).await;
        });
        let peer_connection = PeerConnection {
            peer,
            handle,
            sender: tx,
        };
        self.0
            .insert(peer_connection.peer.name.clone(), peer_connection);
    }
}
