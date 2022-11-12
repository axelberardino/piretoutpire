use super::{bucket_tree::BucketTree, peer_node::PeerNode};
use crate::utils::distance;

// Holds information about other nodes.
// This routing table represents part of the global distributed nodes. Only the
// nodes "close" to id of the owner of this table, are maintained.
pub struct RoutingTable {
    id: u32,
    bucket_tree: BucketTree,
}

impl RoutingTable {
    // Create a new routing table with a given identifier as a reference for
    // what to maintain.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            bucket_tree: BucketTree::new(),
        }
    }

    // Add a new node inside the routing table, store as a distance.
    pub fn add_node(&mut self, mut peer: PeerNode) {
        peer.set_id(distance(peer.id(), self.id));
        self.bucket_tree.add_peer_node(peer);
    }
}

#[cfg(test)]
#[path = "routing_table_test.rs"]
mod routing_table_test;
