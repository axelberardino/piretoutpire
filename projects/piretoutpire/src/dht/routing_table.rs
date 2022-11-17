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
    recent_peers_cache_enabled: bool,
    latest_too_far_peers: VecDeque<PeerNode>,
}

impl RoutingTable {
    // Create a new routing table with a given identifier as a reference for
    // what to maintain.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            bucket_tree: BucketTree::new(),
            recent_peers_cache_enabled: true,
            latest_too_far_peers: VecDeque::new(),
        }
    }

    // Enable the recent peer cache. On small network, with non uniform id
    /// distribution, caching peers could be hard. The "recent" peers cache is
    /// used on top of the routing table, to help finding peers. On big network,
    /// it's usually not needed and could be disactivated.
    pub fn set_recent_peers_cache_enable(&mut self, value: bool) {
        self.recent_peers_cache_enabled = value;
    }

    // Get the peers lru cache
    pub fn get_recent_peers_cache(&self) -> impl Iterator<Item = &PeerNode> {
        self.latest_too_far_peers.iter()
    }

    // Clear the routing table.
    pub async fn clear(&mut self) {
        self.bucket_tree = BucketTree::new();
    }

    // Add a new node inside the routing table, store as a distance.
    pub async fn add_node(&mut self, mut peer: PeerNode) {
        peer.set_id(distance(peer.id(), self.id));
        if let InsertResult::NoRoom = self.bucket_tree.add_peer_node(peer.clone()).await {
            if self.recent_peers_cache_enabled {
                // Push an existing node to the front, or add it.
                if let Some(idx) = self
                    .latest_too_far_peers
                    .iter()
                    .position(|lru| lru.id() == peer.id())
                {
                    self.latest_too_far_peers.remove(idx);
                }
                self.latest_too_far_peers.push_front(peer);

                // Prevent lru to grow too much.
                if self.latest_too_far_peers.len() > 100 {
                    self.latest_too_far_peers.pop_back();
                }
            }
        }
    }

    // Get all peers in this routing table.
    pub async fn get_all_peers(&self) -> impl Iterator<Item = PeerNode> + '_ {
        self.bucket_tree
            .get_all_peers()
            .await
            .map(|mut peer| {
                peer.set_id(distance(peer.id(), self.id));
                peer
            })
            .chain(self.latest_too_far_peers.iter().map(Clone::clone))
    }

    // Get the closest peers from a given target.
    pub async fn get_closest_peers_from(&self, target: u32, nb: usize) -> impl Iterator<Item = PeerNode> {
        let mut peers = self.get_all_peers().await.collect::<Vec<_>>();
        peers.sort_by_key(|peer| distance(peer.id(), target));
        peers.into_iter().take(nb)
    }

    // Flag that we requested a peer. A peer which is requested a lot, but never
    // answer will be considred bad.
    pub async fn peer_was_requested(&mut self, target: u32) {
        self.bucket_tree.peer_was_requested(target).await;
    }

    // Flag that the peer correctly responded, hence is alive.
    pub async fn peer_has_responded(&mut self, target: u32) {
        self.bucket_tree.peer_has_responded(target).await;
    }
}

#[cfg(test)]
#[path = "routing_table_test.rs"]
mod routing_table_test;
