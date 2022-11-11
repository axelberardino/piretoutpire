use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

// Hold state about a peer in the routing table.
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    nb_successive_queries: usize,
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
    // Peer failed to answer, but we don't mark it s bad yet
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
            nb_successive_queries: 0,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn status(&self) -> PeerStatus {
        let last_request = match self.last_request {
            Some(last_req) => last_req,
            None => return PeerStatus::Unknown,
        };
        if self.nb_successive_queries > 0 && last_request.elapsed() > Duration::from_secs(15) {
            return PeerStatus::Bad;
        }
        if self.last_response.is_some() {
            return PeerStatus::Good;
        }
        PeerStatus::Questionable
    }

    pub fn mark_outgoing_request(&mut self) {
        self.last_request = Some(Instant::now());
        self.nb_successive_queries += 1;
    }

    pub fn mark_response(&mut self) {
        let now = Instant::now();
        self.last_response = Some(now);
        if self.last_request.is_none() {
            self.last_request = Some(now);
        }
        self.nb_successive_queries = 0;
    }
}
