use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

// Hold state about a peer in the routing table.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct PeerNode {
    // Id of the peer
    id: u32,
    // Network address of the peer
    addr: SocketAddr,
    // Last time a request was sent
    #[serde(skip)]
    last_request: Option<Instant>,
    // Last time a response was sent
    #[serde(skip)]
    last_response: Option<Instant>,
    // Number of queries made in a row
    #[serde(skip)]
    nb_successive_try: usize,
}

// Status of a peer.
// As described here: https://www.bittorrent.org/beps/bep_0005.html
#[derive(Eq, PartialEq)]
pub enum PeerStatus {
    // Is there and answering
    Good,
    // Didn't test if still there, but not out either
    // Usually didn't being ping'ed during the last 15 min
    Questionable,
    // Peer definitively quit
    Bad,
    // Peer status unknown, usually when loading the dht from a file.
    Unknown,
}

impl PeerNode {
    // Construct a new peer node
    pub fn new(id: u32, addr: SocketAddr) -> Self {
        Self {
            id,
            addr,
            last_request: None,
            last_response: None,
            nb_successive_try: 0,
        }
    }

    // Get the peer id.
    pub fn id(&self) -> u32 {
        self.id
    }

    // Set the peer id.
    pub fn set_id(&mut self, new_id: u32) {
        self.id = new_id;
    }

    // Get the real ip address.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    // Return the status of the node by looking on recent events.
    pub fn status(&self) -> PeerStatus {
        let last_request = match self.last_request {
            Some(last_req) => last_req,
            None => return PeerStatus::Unknown,
        };
        if self.nb_successive_try > 0 && last_request.elapsed() > Duration::from_secs(15) {
            return PeerStatus::Bad;
        }
        if self.last_response.is_some() {
            return PeerStatus::Good;
        }
        PeerStatus::Questionable
    }

    // Update the last request made
    pub fn update_last_request(&mut self) {
        self.last_request = Some(Instant::now());
        self.nb_successive_try += 1;
    }

    // Update the last response made
    pub fn update_last_response(&mut self) {
        let now = Instant::now();
        self.last_response = Some(now);
        if self.last_request.is_none() {
            self.last_request = Some(now);
        }
        self.nb_successive_try = 0;
    }
}
