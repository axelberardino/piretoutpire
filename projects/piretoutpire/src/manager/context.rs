use crate::{
    dht::dht::DistributedHashTable,
    file::{file_chunk::FileChunk, torrent_file::TorrentFile},
};
use std::{collections::HashMap, net::SocketAddr};

// Context handle everything about shared context.
pub struct Context {
    // Realtime list of peers
    pub peers: HashMap<u32, SocketAddr>, // TODO: DHT

    // Contains a trackerless list of local peers.
    pub dht: DistributedHashTable,

    // List of all available torrents currently owned, or currently downloading.
    pub available_torrents: HashMap<u32 /*crc*/, (TorrentFile<String> /*metadata*/, FileChunk /*file*/)>,

    // Where all torrents and their metadata are.
    pub working_directory: String,
}

impl Context {
    // Create a new context from a working directory.
    pub fn new(working_directory: String, self_id: u32) -> Self {
        Self {
            peers: HashMap::new(),
            dht: DistributedHashTable::new(self_id),
            available_torrents: HashMap::new(),
            working_directory,
        }
    }
}
