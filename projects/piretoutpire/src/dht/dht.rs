use super::{peer_node::PeerNode, routing_table::RoutingTable};
use std::net::SocketAddr;

// The DHT is a way to handle a collaborative hash map. It allows to maintain a
// decentralized network.
pub struct DistributedHashTable {
    routing_table: RoutingTable,
}

impl DistributedHashTable {
    // Initiate a new DHT for a given user.
    pub fn new(id: u32) -> Self {
        Self {
            routing_table: RoutingTable::new(id),
        }
    }

    // Try to find a given node. Either return it, or return the closest known
    // node. When trying to find a node, also add the sender inside the routing
    // table.
    pub async fn find_node(&mut self, sender: PeerNode, target: u32) -> impl Iterator<Item = PeerNode> {
        let res = self
            .routing_table
            .get_closest_peers_from(target, 4)
            .await
            .collect::<Vec<_>>();
        self.routing_table.add_node(sender).await;
        res.into_iter()
    }

    // Add a new node for ease of purpose in test files.
    pub async fn add_node(&mut self, id: u32, addr: SocketAddr) {
        self.add_peer_node(PeerNode::new(id, addr)).await;
    }

    // Add a new node for ease of purpose in test files.
    async fn add_peer_node(&mut self, peer: PeerNode) {
        self.routing_table.add_node(peer).await;
    }
}

// Only exists for testing purpose.
#[cfg(test)]
impl DistributedHashTable {
    // Get the number of peer in the dht.
    async fn len(&self) -> usize {
        self.routing_table.get_all_peers().await.count()
    }
}

#[cfg(test)]
#[path = "dht_test.rs"]
mod dht_test;
