use super::routing_table::RoutingTable;

// The DHT is a way to handle a collaborative hash map. It allows to maintain a
// decentralized network.
pub struct DistributedHashTable {
    id: u32,
    routing_table: RoutingTable,
}

impl DistributedHashTable {
    // Initiate a new DHT for a given user.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            routing_table: RoutingTable::new(id),
        }
    }
}
