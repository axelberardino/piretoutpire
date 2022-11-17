use crate::{
    dht::dht::DistributedHashTable,
    file::{file_chunk::FileChunk, torrent_file::TorrentFile},
};
use std::{collections::HashMap, time::Duration};

pub const DEFAULT_READ_TIMEOUT_MS: u64 = 200;
pub const DEFAULT_WRITE_TIMEOUT_MS: u64 = 200;
pub const DEFAULT_CONNECTION_TIMEOUT_MS: u64 = 200;
pub const DEFAULT_DHT_DUMP_FREQUENCY_MS: u64 = 30 * 1000; // 30 sec

// Context handle everything about shared context
pub struct Context {
    // Contains a trackerless list of local peers
    pub dht: DistributedHashTable,

    // List of all available torrents currently owned, or currently downloading
    pub available_torrents: HashMap<u32 /*crc*/, (TorrentFile<String> /*metadata*/, FileChunk /*file*/)>,

    // Where all torrents and their metadata are
    pub working_directory: String,

    /// Simulate a slowness, for debug purpose.
    pub slowness: Option<Duration>,

    /// Max wait time for initiating a connection.
    pub connection_timeout: Duration,

    /// Max wait time for sending a query.
    pub write_timeout: Duration,

    /// Max wait time for receiving a query.
    pub read_timeout: Duration,

    /// Frequency at which the dht is dump on the disk.
    pub dht_dump_frequency: Duration,

    /// Where to save the dht
    pub dht_config_filename: String,
}

impl Context {
    // Create a new context from a working directory
    pub fn new(dht_config_filename: String, working_directory: String, self_id: u32) -> Self {
        Self {
            dht: DistributedHashTable::new(self_id),
            available_torrents: HashMap::new(),
            dht_config_filename,
            working_directory,
            slowness: None,
            connection_timeout: Duration::from_millis(DEFAULT_CONNECTION_TIMEOUT_MS),
            write_timeout: Duration::from_millis(DEFAULT_WRITE_TIMEOUT_MS),
            read_timeout: Duration::from_millis(DEFAULT_READ_TIMEOUT_MS),
            dht_dump_frequency: Duration::from_millis(DEFAULT_DHT_DUMP_FREQUENCY_MS),
        }
    }
}

// Special constructor for test purpose
#[cfg(test)]
impl Context {
    // Create a new context with a cache lru enabled/disabled
    pub fn new_test(self_id: u32, enable_lru: bool) -> Self {
        let mut dht = DistributedHashTable::new(self_id);
        dht.set_recent_peers_cache_enable(enable_lru);
        Self {
            dht,
            available_torrents: HashMap::new(),
            dht_config_filename: "".to_owned(),
            working_directory: "".to_owned(),
            slowness: None,
            connection_timeout: Duration::from_millis(DEFAULT_CONNECTION_TIMEOUT_MS),
            write_timeout: Duration::from_millis(DEFAULT_WRITE_TIMEOUT_MS),
            read_timeout: Duration::from_millis(DEFAULT_READ_TIMEOUT_MS),
            dht_dump_frequency: Duration::from_millis(DEFAULT_DHT_DUMP_FREQUENCY_MS),
        }
    }
}
