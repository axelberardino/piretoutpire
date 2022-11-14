use super::{
    bucket_tree::{BucketTree, InsertResult},
    peer_node::PeerNode,
};
use crate::utils::distance;
use std::collections::VecDeque;

// Holds information about other nodes.
// This routing table represents part of the global distributed nodes. Only the
// nodes "close" to id of the owner of this table, are maintained.
#[derive(Debug)]
pub struct RoutingTable {
    id: u32,
    bucket_tree: BucketTree,
    latest_too_far_nodes: VecDeque<PeerNode>,
}

impl RoutingTable {
    // Create a new routing table with a given identifier as a reference for
    // what to maintain.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            bucket_tree: BucketTree::new(),
            latest_too_far_nodes: VecDeque::new(),
        }
    }

    // Clean the routing table.
    pub async fn clean(&mut self) {
        self.bucket_tree = BucketTree::new();
    }

    // Add a new node inside the routing table, store as a distance.
    pub async fn add_node(&mut self, mut peer: PeerNode) {
        peer.set_id(distance(peer.id(), self.id));
        if let InsertResult::NoRoom = self.bucket_tree.add_peer_node(peer.clone()).await {
            self.latest_too_far_nodes.push_front(peer);
            if self.latest_too_far_nodes.len() > 100 {
                self.latest_too_far_nodes.pop_back();
            }
        }
    }

    // Get all peers in this routing table.
    // FIXME: collect() followed by into_iter, not great
    pub async fn get_all_peers(&self) -> impl Iterator<Item = PeerNode> {
        self.bucket_tree
            .get_all_peers()
            .await
            .map(|mut peer| {
                peer.set_id(distance(peer.id(), self.id));
                peer
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    // Get the closest peers from a given target.
    pub async fn get_closest_peers_from(&self, target: u32, nb: usize) -> impl Iterator<Item = PeerNode> {
        let mut peers = self.get_all_peers().await.collect::<Vec<_>>();
        peers.sort_by_key(|peer| distance(peer.id(), target));
        peers.into_iter().take(nb)
    }
}

#[cfg(test)]
#[path = "routing_table_test.rs"]
mod routing_table_test;
