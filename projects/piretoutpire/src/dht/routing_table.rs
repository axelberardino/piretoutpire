// Holds information about other nodes.
// This routing table represents part of the global distributed nodes. Only the
// nodes "close" to id of the owner of this table, are maintained.
pub struct RoutingTable {
    id: u32,
}

impl RoutingTable {
    // Create a new routing table with a given identifier as a reference for
    // what to maintain.
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

#[cfg(test)]
#[path = "routing_table_test.rs"]
mod routing_table_test;
