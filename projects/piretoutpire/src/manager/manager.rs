use super::{
    client::{
        handle_announce, handle_file_chunk, handle_file_info, handle_find_value, handle_get_peers,
        handle_message, handle_ping, handle_store,
    },
    command_handler::listen_to_command,
    context::{
        Context, DEFAULT_CONNECTION_TIMEOUT_MS, DEFAULT_DHT_DUMP_FREQUENCY_MS, DEFAULT_READ_TIMEOUT_MS,
        DEFAULT_WRITE_TIMEOUT_MS,
    },
    find_node::{find_closest_node, query_find_node},
};
use crate::{
    dht::peer_node::PeerNode,
    file::{file_chunk::FileChunk, torrent_file::TorrentFile},
    network::protocol::{FileInfo, Peer},
};
use errors::{reexports::eyre::ContextCompat, AnyResult};
use std::{
    net::SocketAddr,
    ops::{Deref, DerefMut},
    path::Path,
    sync::Arc,
    time::Duration,
};
use tokio::{
    self,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

// Handle everything about peer. RPC calls, connection handling, and
// configuration load and write.
pub struct Manager {
    id: u32,
    addr: SocketAddr,
    ctx: Arc<Mutex<Context>>,
    max_hop: Option<u32>,
    dht_config_filename: String,
}

impl Manager {
    // CONSTRUCTOR -------------------------------------------------------------

    // Create a new manager. Expect an address like: "127.0.0.1:8080".parse()
    pub fn new(id: u32, addr: SocketAddr, dht_config_filename: String, working_directory: String) -> Self {
        Self {
            id,
            addr,
            ctx: Arc::new(Mutex::new(Context::new(working_directory, id))),
            max_hop: None,
            dht_config_filename,
        }
    }

    // OPTIONS -----------------------------------------------------------------

    // Set the max hop possible when searchin for a node.
    // None = default behavior (stop when no closest host is found).
    // N = force to hop N times even if not the best route.
    pub fn set_max_hop(&mut self, max_hop: Option<u32>) {
        self.max_hop = max_hop;
    }

    // Enable the recent peer cache. On small network, with non uniform id
    // distribution, caching peers could be hard. The "recent" peers cache is
    // used on top of the routing table, to help finding peers. On big network,
    // it's usually not needed and could be disactivated.
    pub async fn set_recent_peers_cache_enable(&mut self, value: bool) {
        let mut guard = self.ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.dht.set_recent_peers_cache_enable(value);
    }

    // Force this peer to wait X ms before answering each rpc (for debug
    // purpose).
    pub async fn set_slowness(&mut self, value: Option<u64>) {
        let mut guard = self.ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.slowness = value.map(|val| Duration::from_millis(val));
    }

    /// Max wait time for initiating a connection (default is 200 ms).
    pub async fn set_connection_timeout(&mut self, value: Option<u64>) {
        let mut guard = self.ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.connection_timeout = Duration::from_millis(value.unwrap_or(DEFAULT_CONNECTION_TIMEOUT_MS));
    }

    /// Max wait time for sending a query (default is 200 ms).
    pub async fn set_write_timeout(&mut self, value: Option<u64>) {
        let mut guard = self.ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.write_timeout = Duration::from_millis(value.unwrap_or(DEFAULT_WRITE_TIMEOUT_MS));
    }

    /// Max wait time for receiving a query (default is 200 ms).
    pub async fn set_read_timeout(&mut self, value: Option<u64>) {
        let mut guard = self.ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.read_timeout = Duration::from_millis(value.unwrap_or(DEFAULT_READ_TIMEOUT_MS));
    }

    /// Frequency at which the dht is dump into the disk.
    pub async fn set_dht_dump_frequency(&mut self, value: Option<u64>) {
        let mut guard = self.ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.dht_dump_frequency = Duration::from_millis(value.unwrap_or(DEFAULT_DHT_DUMP_FREQUENCY_MS));
    }

    // CONFIG ------------------------------------------------------------------

    // Dump the dht into a file.
    pub async fn dump_dht(&self) -> AnyResult<()> {
        dump_dht(Arc::clone(&self.ctx), &self.dht_config_filename).await?;
        Ok(())
    }

    // Reload the dht from a given file.
    pub async fn load_dht(&mut self, path: &Path) -> AnyResult<()> {
        let mut guard = self.ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.dht.load_from_file(path).await?;
        Ok(())
    }

    // Get the number of known peers.
    pub async fn known_peers_count(&mut self) -> usize {
        let mut guard = self.ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.dht.known_peers_count().await
    }

    // LOCAL FILES -------------------------------------------------------------

    // FIXME load dir
    // Load all files in a given directory to be shared on the peer network.
    // pub async fn load_directory<P: AsRef<Path>>(&mut self, dir: P) -> AnyResult<()> {}

    // Load a file to be shared on the peer network.
    pub async fn load_file<P: AsRef<Path>>(&mut self, file: P) -> AnyResult<()> {
        let torrent = TorrentFile::new(
            file.as_ref().display().to_string() + ".metadata",
            file.as_ref().display().to_string(),
        )?;
        let chunks = FileChunk::open_existing(&torrent.metadata.original_file)?;

        let mut ctx = self.ctx.lock().await;
        ctx.available_torrents
            .insert(torrent.metadata.file_crc, (torrent, chunks));

        Ok(())
    }

    // SERVER ------------------------------------------------------------------

    // Start the backend server to listen to command and seed.
    pub async fn start_server(&self) -> AnyResult<()> {
        let dht_dump_frequency = {
            let guard = self.ctx.lock().await;
            let ctx = guard.deref();
            ctx.dht_dump_frequency
        };

        let listener = TcpListener::bind(self.addr).await?;

        // Let's write the peers list regularly on the disk.
        let dht_config_filename = self.dht_config_filename.clone();
        let ctx = Arc::clone(&self.ctx);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(dht_dump_frequency);
            loop {
                interval.tick().await;
                let _ = dump_dht(Arc::clone(&ctx), &dht_config_filename).await;
                println!("dht dumped!");
            }
        });

        // Accept all incoming connection, and spawn a new thread for each.
        loop {
            let (stream, _) = listener.accept().await?;
            let ctx = Arc::clone(&self.ctx);
            tokio::spawn(async move { listen_to_command(ctx, stream).await });
        }
    }

    // RPC ---------------------------------------------------------------------

    // Start to bootstrap the DHT from an entry point (any available peers).
    // Start by pinging it, then send a find_node on ourself.
    pub async fn bootstrap(&mut self, peer_addr: SocketAddr) -> AnyResult<Option<Peer>> {
        let peer = Peer {
            id: u32::MAX,
            addr: peer_addr,
        };

        // As we don't know the id of the peer yet, let's ask him, and put that
        // into our dht.
        let target = ping(Arc::clone(&self.ctx), peer, self.id).await?;

        let peer = Peer {
            id: target,
            addr: peer_addr,
        };

        // Ask for the entry node for ourself. He will add us into its table,
        // then give back 4 close nodes.
        let peer = find_closest_node(
            Arc::clone(&self.ctx),
            peer,
            self.id,
            self.id,
            self.max_hop,
            query_find_node,
        )
        .await?;
        Ok(peer)
    }

    // Find node will try to return the wanted peer, or the 4 most closest ones
    // if he's not found.
    pub async fn find_node(&mut self, target: u32) -> AnyResult<Vec<Peer>> {
        // Get the closest possible node from the target, to start the search.
        let peer = {
            let guard = self.ctx.lock().await;
            let ctx = guard.deref();
            ctx.dht
                .find_closest_peer(target)
                .await
                .filter(|peer| peer.id() == target)
        };

        match peer {
            Some(peer) if peer.id() == target => {
                // We already have it, so no need to make any RPC
                Ok(vec![peer.into()])
            }
            Some(peer) => {
                // Let's start the search
                let found = find_closest_node(
                    Arc::clone(&self.ctx),
                    peer.into(),
                    self.id,
                    target,
                    self.max_hop,
                    query_find_node,
                )
                .await?;
                match found {
                    Some(peer) => Ok(vec![peer]),
                    None => {
                        let guard = self.ctx.lock().await;
                        let ctx = guard.deref();
                        Ok(ctx
                            .dht
                            .find_closest_peers(target, 4)
                            .await
                            .map(Into::into)
                            .collect())
                    }
                }
            }
            None => {
                // No local peer to start the search.
                Ok(vec![])
            }
        }
    }

    // Ping a peer by its id. Return if we know the peer.
    pub async fn ping(&self, target: u32) -> AnyResult<bool> {
        let peer = {
            let mut guard = self.ctx.lock().await;
            let ctx = guard.deref_mut();
            ctx.dht
                .find_closest_peer(target)
                .await
                .filter(|peer| peer.id() == target)
        };

        if let Some(peer) = peer {
            Ok(ping(Arc::clone(&self.ctx), peer.into(), self.id).await.is_err())
        } else {
            Ok(false)
        }
    }

    // Send a message to a peer. Return if the peer acknowledge it.
    pub async fn send_message(&self, target: u32, message: String) -> AnyResult<bool> {
        let mut guard = self.ctx.lock().await;
        let ctx = guard.deref_mut();
        // Let's check if we have a candidate, and if our exact node.
        let close_peer = ctx.dht.find_closest_peer(target).await;

        let peer = match close_peer {
            Some(peer) if peer.id() == target => peer,
            Some(peer) => {
                // We don't have this peer, let's try to find it.
                find_closest_node(
                    Arc::clone(&self.ctx),
                    peer.into(),
                    self.id,
                    self.id,
                    self.max_hop,
                    query_find_node,
                )
                .await?;
                let close_peer_again = ctx.dht.find_closest_peer(target).await;
                match close_peer_again {
                    Some(peer) if peer.id() == target => peer,
                    _ => return Ok(false),
                }
            }
            None => return Ok(false),
        };

        let stream = Arc::new(Mutex::new(TcpStream::connect(peer.addr()).await?));
        handle_message(Arc::clone(&self.ctx), stream, message).await?;
        Ok(true)
    }

    // Return a file description from its crc
    pub async fn file_info(&mut self, crc: u32) -> AnyResult<Option<FileInfo>> {
        match self.get_peers(crc).await? {
            Some(peers) => {
                for peer in peers {
                    let stream = Arc::new(Mutex::new(TcpStream::connect(peer.addr).await?));
                    let res = handle_file_info(Arc::clone(&self.ctx), stream, crc).await?;
                    if res.is_some() {
                        return Ok(res);
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    // Start downloading a file, or resume downloading
    pub async fn download_file(&mut self, crc: u32) -> AnyResult<()> {
        let ctx = Arc::clone(&self.ctx);
        // FIXME ask for peers for download

        // Get some info about what to download
        let nb_chunks = {
            let mut guard = ctx.lock().await;
            let ctx = guard.deref_mut();

            let (_, chunks) = ctx
                .available_torrents
                .get(&crc)
                .context("unable to find associated chunks")?;

            chunks.nb_chunks()
        };

        // FIXME downloaded chunk abort too soon, handle that better
        {
            let client_addr: SocketAddr = "127.0.0.1:4000".parse()?;
            let stream = Arc::new(Mutex::new(TcpStream::connect(client_addr).await?));
            let mut queries = Vec::new();
            for chunk_id in 0..nb_chunks {
                let local_ctx = Arc::clone(&ctx);
                let stream = Arc::clone(&stream);

                let handle =
                    tokio::spawn(async move { handle_file_chunk(local_ctx, stream, crc, chunk_id).await });
                queries.push(handle);
            }
            for handle in queries {
                handle.await??;
            }
        }

        Ok(())
    }

    // Find the given value by its key. Search locally, then if not found, ask
    // peers for the value.
    pub async fn find_value(&mut self, target: u32) -> AnyResult<Option<String>> {
        let (value, closest_peers) = {
            let guard = self.ctx.lock().await;
            let ctx = guard.deref();
            (
                ctx.dht.get_value(target).map(Clone::clone),
                ctx.dht.find_closest_peers(target, 4).await,
            )
        };
        // We already have this value locally
        if value.is_some() {
            return Ok(value);
        }

        // Starting for the 4 closest peers, search for this value
        for peer in closest_peers {
            find_closest_node(
                Arc::clone(&self.ctx),
                peer.into(),
                self.id,
                target,
                self.max_hop,
                query_find_node,
            )
            .await?;

            let closest_peers = {
                let guard = self.ctx.lock().await;
                let ctx = guard.deref();
                ctx.dht.find_closest_peers(target, 4).await
            };
            for close_peer in closest_peers {
                let stream = Arc::new(Mutex::new(TcpStream::connect(close_peer.addr()).await?));
                let message = handle_find_value(Arc::clone(&self.ctx), stream, target).await?;
                if message.is_some() {
                    return Ok(message);
                }
            }
        }

        Ok(None)
    }

    // Find the given value by its key. Search locally, then if not found, ask
    // peers for the value.
    pub async fn store_value(&mut self, target: u32, message: String) -> AnyResult<usize> {
        // Store the value for us
        let closest_peer = {
            let mut guard = self.ctx.lock().await;
            let ctx = guard.deref_mut();
            ctx.dht.store_value(target, message.clone());
            ctx.dht.find_closest_peers(target, 1).await
        };

        if let Some(peer) = closest_peer.take(1).collect::<Vec<PeerNode>>().pop() {
            // Let's find the 4 closest nodes to us, and then ask them to store our
            // value.
            find_closest_node(
                Arc::clone(&self.ctx),
                peer.into(),
                self.id,
                target,
                self.max_hop,
                query_find_node,
            )
            .await?;

            let closest_peers = {
                let guard = self.ctx.lock().await;
                let ctx = guard.deref();
                ctx.dht.find_closest_peers(target, 4).await
            };
            let mut nb_store = 0;
            for close_peer in closest_peers {
                let stream = Arc::new(Mutex::new(TcpStream::connect(close_peer.addr()).await?));
                if handle_store(Arc::clone(&self.ctx), stream, target, message.clone())
                    .await
                    .is_ok()
                {
                    nb_store += 1;
                }
            }
            return Ok(nb_store);
        }

        Ok(0)
    }

    // Declare to closest peers that we're sharing a file.
    pub async fn announce(&mut self, crc: u32) -> AnyResult<usize> {
        // Store the value for us
        let closest_peer = {
            let mut guard = self.ctx.lock().await;
            let ctx = guard.deref_mut();
            ctx.dht.store_file_peer(
                crc,
                Peer {
                    id: self.id,
                    addr: self.addr,
                },
            );
            ctx.dht.find_closest_peers(crc, 1).await
        };

        if let Some(peer) = closest_peer.take(1).collect::<Vec<PeerNode>>().pop() {
            // Let's find the 4 closest nodes to us, and then ask them to store our
            // value.
            find_closest_node(
                Arc::clone(&self.ctx),
                peer.into(),
                self.id,
                crc,
                self.max_hop,
                query_find_node,
            )
            .await?;

            let closest_peers = {
                let guard = self.ctx.lock().await;
                let ctx = guard.deref();
                ctx.dht.find_closest_peers(crc, 4).await
            };
            let mut nb_store = 0;
            for close_peer in closest_peers {
                let stream = Arc::new(Mutex::new(TcpStream::connect(close_peer.addr()).await?));
                if handle_announce(Arc::clone(&self.ctx), stream, self.id, crc)
                    .await
                    .is_ok()
                {
                    nb_store += 1;
                }
            }
            return Ok(nb_store);
        }

        Ok(0)
    }

    // Get all peers who owned a file, given its crc.
    pub async fn get_peers(&mut self, crc: u32) -> AnyResult<Option<Vec<Peer>>> {
        // Start to search locally.
        let closest_peers = {
            let guard = self.ctx.lock().await;
            let ctx = guard.deref();
            if let Some(peer_iter) = ctx.dht.get_file_peers(crc) {
                return Ok(Some(peer_iter.map(Clone::clone).collect()));
            }

            ctx.dht.find_closest_peers(crc, 4).await
        };

        // Starting for the 4 closest peers, search for this value
        for peer in closest_peers {
            find_closest_node(
                Arc::clone(&self.ctx),
                peer.into(),
                self.id,
                crc,
                self.max_hop,
                query_find_node,
            )
            .await?;

            let closest_peers = {
                let guard = self.ctx.lock().await;
                let ctx = guard.deref();
                ctx.dht.find_closest_peers(crc, 4).await
            };
            for close_peer in closest_peers {
                let stream = Arc::new(Mutex::new(TcpStream::connect(close_peer.addr()).await?));
                let message = handle_get_peers(Arc::clone(&self.ctx), stream, crc).await?;
                if message.is_some() {
                    return Ok(message);
                }
            }
        }

        Ok(None)
    }
}

// Helpers ---------------------------------------------------------------------

// Ping a peer from its real address, ask him its id and put it into our dht.
async fn ping(ctx: Arc<Mutex<Context>>, peer: Peer, sender: u32) -> AnyResult<u32> {
    let stream = Arc::new(Mutex::new(TcpStream::connect(peer.addr).await?));
    let target = handle_ping(Arc::clone(&ctx), stream, sender).await?;

    // The peer just answered us, let's add him into our dht.
    {
        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.dht.add_node(target, peer.addr).await;
    }

    Ok(target)
}

// Dump the dht into a file.
async fn dump_dht(ctx: Arc<Mutex<Context>>, dht_config_filename: &String) -> AnyResult<()> {
    let guard = ctx.lock().await;
    let ctx = guard.deref();
    ctx.dht.dump_to_file(Path::new(dht_config_filename)).await?;
    Ok(())
}
