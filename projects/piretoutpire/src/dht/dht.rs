use super::{peer_node::PeerNode, routing_table::RoutingTable};
use crate::network::protocol::Peer;
use errors::AnyResult;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    io::BufReader,
    net::SocketAddr,
    path::Path,
};

// The DHT is a way to handle a collaborative hash map. It allows to maintain a
// decentralized network.
#[derive(Debug)]
pub struct DistributedHashTable {
    id: u32,
    routing_table: RoutingTable,
    kv_store: HashMap<u32, String>,
    files_store: HashMap<u32, HashSet<Peer>>,
}

// Intermediary structure to serialize and deserialize dht peers.
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    id: u32,
    peers: Vec<PeerNode>,
    peers_lru: Vec<PeerNode>,
    kv_store: HashMap<u32, String>,
    files_store: HashMap<u32, Vec<Peer>>,
}

impl DistributedHashTable {
    // Initiate a new DHT for a given user.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            routing_table: RoutingTable::new(id),
            kv_store: HashMap::new(),
            files_store: HashMap::new(),
        }
    }

    // Get the owner id of this DHT.
    pub fn id(&self) -> u32 {
        self.id
    }

    // Enable the recent peer cache. On small network, with non uniform id
    /// distribution, caching peers could be hard. The "recent" peers cache is
    /// used on top of the routing table, to help finding peers. On big network,
    /// it's usually not needed and could be disactivated.
    pub fn set_recent_peers_cache_enable(&mut self, value: bool) {
        self.routing_table.set_recent_peers_cache_enable(value);
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

    // Search for the closest peer.
    pub async fn find_closest_peer(&self, target: u32) -> Option<PeerNode> {
        let mut res = self.find_closest_peers(target, 1).await.collect::<Vec<_>>();
        res.pop()
    }

    // Search for the N closest peers.
    pub async fn find_closest_peers(&self, target: u32, nb: usize) -> impl Iterator<Item = PeerNode> {
        let res = self
            .routing_table
            .get_closest_peers_from(target, nb)
            .await
            .collect::<Vec<_>>();
        res.into_iter()
    }

    // Add a new node for ease of purpose in test files.
    pub async fn add_node(&mut self, id: u32, addr: SocketAddr) {
        self.add_peer_node(PeerNode::new(id, addr)).await;
    }

    // Clean the whole dht.
    pub async fn clean(&mut self) {
        self.routing_table.clear().await;
    }

    // Dump this dht into a file.
    pub async fn dump_to_file(&self, path: &Path) -> AnyResult<()> {
        let config = Config {
            id: self.id,
            peers: self.routing_table.get_all_peers().await.collect(),
            peers_lru: self
                .routing_table
                .get_recent_peers_cache()
                .map(Clone::clone)
                .collect(),
            kv_store: self.kv_store.clone(),
            files_store: self
                .files_store
                .iter()
                .map(|(key, peers)| (*key, peers.into_iter().map(Clone::clone).collect::<Vec<_>>()))
                .collect(),
        };

        let file = OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(&path)?;
        serde_json::to_writer(file, &config)?;

        Ok(())
    }

    // Reload the dht from a given file.
    pub async fn load_from_file(&mut self, path: &Path) -> AnyResult<()> {
        let file = OpenOptions::new().read(true).open(&path)?;
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;

        self.clean().await;
        self.id = config.id;
        for peer in config.peers.into_iter().chain(config.peers_lru.into_iter()) {
            self.add_peer_node(peer).await;
        }
        self.kv_store = config.kv_store;
        self.files_store = config
            .files_store
            .into_iter()
            .map(|(key, peers)| (key, peers.into_iter().collect::<HashSet<_>>()))
            .collect();

        Ok(())
    }

    // Store a given value for a given key.
    // Value will be overwritten.
    pub fn store_value(&mut self, key: u32, message: String) {
        self.kv_store.insert(key, message);
    }

    // Get a stored value from its key.
    pub fn get_value(&self, key: u32) -> Option<&String> {
        self.kv_store.get(&key)
    }

    // Store a given peer file owner for a given key.
    // Value will be added to the list.
    pub fn store_file_peer(&mut self, key: u32, peer: Peer) {
        self.files_store
            .entry(key)
            .and_modify(|peers| {
                peers.insert(peer.clone());
            })
            .or_insert_with(|| {
                let mut set = HashSet::new();
                set.insert(peer);
                set
            });
    }

    // Get a list of peers who own a given file.
    pub fn get_file_peers(&self, key: u32) -> Option<impl Iterator<Item = &Peer>> {
        self.files_store.get(&key).and_then(|peers| Some(peers.iter()))
    }

    // Flag that we requested a peer. A peer which is requested a lot, but never
    // answer will be considred bad.
    pub async fn peer_was_requested(&mut self, target: u32) {
        self.routing_table.peer_was_requested(target).await;
    }

    // Flag that the peer correctly responded, hence is alive.
    pub async fn peer_has_responded(&mut self, target: u32) {
        self.routing_table.peer_has_responded(target).await;
    }
}

// Private method.
impl DistributedHashTable {
    // Add a new node for ease of purpose in test files.
    async fn add_peer_node(&mut self, peer: PeerNode) {
        self.routing_table.add_node(peer).await;
    }
}

// Only exists for testing purpose.
#[cfg(test)]
impl DistributedHashTable {
    // Get the number of peer in the dht.
    pub async fn len(&self) -> usize {
        self.routing_table.get_all_peers().await.count()
    }

    // Get all peer ids, sorted.
    pub async fn peer_ids(&self) -> Vec<u32> {
        let mut res = self
            .routing_table
            .get_all_peers()
            .await
            .map(|peer| peer.id())
            .collect::<Vec<u32>>();
        res.sort();
        res
    }
}

#[cfg(test)]
#[path = "dht_test.rs"]
mod dht_test;
