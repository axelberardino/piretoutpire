use super::{
    client::{handle_file_chunk, handle_file_info, handle_find_node, handle_message, handle_ping},
    command_handler::listen_to_command,
    context::Context,
};
use crate::{
    file::{file_chunk::FileChunk, torrent_file::TorrentFile},
    network::protocol::Peer,
    utils::distance,
};
use errors::{reexports::eyre::ContextCompat, AnyError, AnyResult};
use std::{
    collections::HashSet,
    future::Future,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    path::Path,
    sync::Arc,
};
use tokio::{
    self,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

pub struct Manager {
    id: u32,
    addr: SocketAddr,
    ctx: Arc<Mutex<Context>>,
    max_hop: Option<u32>,
}

impl Manager {
    // Expect an address like: "127.0.0.1:8080".parse()
    pub fn new(id: u32, addr: SocketAddr, working_directory: String) -> Self {
        Self {
            id,
            addr,
            ctx: Arc::new(Mutex::new(Context::new(working_directory, id))),
            max_hop: None,
        }
    }

    // Set the max hop possible when searchin for a node.
    // None = default behavior (stop when no closest host is found).
    // N = force to hop N times even if not the best route.
    pub fn set_max_hop(&mut self, max_hop: Option<u32>) {
        self.max_hop = max_hop;
    }

    // Dump the dht into a file.
    pub async fn dump_dht(&self, path: &Path) -> AnyResult<()> {
        let guard = self.ctx.lock().await;
        let ctx = guard.deref();
        ctx.dht.dump_to_file(path).await?;
        Ok(())
    }

    // Reload the dht from a given file.
    pub async fn load_dht(&mut self, path: &Path) -> AnyResult<()> {
        let mut guard = self.ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.dht.load_from_file(path).await?;
        Ok(())
    }

    // Start to bootstrap the DHT from an entry point (any available peer).
    pub async fn bootstrap(&mut self, peer_addr: SocketAddr) -> AnyResult<()> {
        let peer = Peer {
            id: u32::MAX,
            addr: peer_addr,
        };

        // As we don't know the id of the pee yet, let's ask him, and put that
        // into our dht.
        let target = ping(Arc::clone(&self.ctx), peer, self.id).await?;

        let peer = Peer {
            id: target,
            addr: peer_addr,
        };

        // Ask for the entry node for ourself. He will add us into its table,
        // then give back 4 close nodes.
        find_closest_node(
            Arc::clone(&self.ctx),
            peer,
            self.id,
            self.id,
            self.max_hop,
            query_find_node,
        )
        .await?;
        Ok(())
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
                    Peer {
                        id: peer.id(),
                        addr: peer.addr(),
                    },
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
        handle_message(stream, message).await?;
        Ok(true)
    }

    // Start downloading a file, or resume downloading
    pub async fn download_file(&mut self, crc: u32) -> AnyResult<()> {
        let ctx = Arc::clone(&self.ctx);
        // TODO ask for peers.

        // Get file info
        {
            let client_addr: SocketAddr = "127.0.0.1:4000".parse()?;
            let local_ctx = Arc::clone(&ctx);
            let stream = Arc::new(Mutex::new(TcpStream::connect(client_addr).await?));
            handle_file_info(local_ctx, stream, crc).await?;
        }

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

        // self.start_stream().await?;
        Ok(())
    }

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

    // FIXME
    // Load all files in a given directory to be shared on the peer network.
    // pub async fn load_directory<P: AsRef<Path>>(&mut self, dir: P) -> AnyResult<()> {}

    pub async fn start_server(&self) -> AnyResult<()> {
        let listener = TcpListener::bind(self.addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;
            let ctx = Arc::clone(&self.ctx);
            tokio::spawn(async move { listen_to_command(ctx, stream).await });
        }
    }
}

// Search for a requested node until finding it. Will stop if the most closest
// ones found in a row are not closer.
// Return either the found peer or none.
async fn find_closest_node<F, T>(
    ctx: Arc<Mutex<Context>>,
    initial_peer: Peer,
    sender: u32,
    target: u32,
    max_hop: Option<u32>,
    query_func: F,
) -> AnyResult<Option<Peer>>
where
    F: FnMut(Arc<Mutex<Context>>, Peer, u32, u32) -> T + Send + Copy + 'static,
    T: Future<Output = AnyResult<Vec<Peer>>> + Send + 'static,
{
    let mut hop = 0;
    let mut queue = vec![initial_peer];
    let mut visited = HashSet::<u32>::new();
    visited.insert(sender); // Let's avoid ourself.
    let mut best_distance = u32::MAX;
    let mut found_peer = None::<Peer>;

    loop {
        hop += 1;

        // Just launch 3 find_node at the same time, with the first 3
        // non-visited peers in the queue. Will drain peer from the queue, until
        // the queue is empty or 3 non visited has been queried.
        // Will responsd with a queue containing from 0 up to 3*4 uniques nodes.
        let next_queue = parallel_find_node(
            Arc::clone(&ctx),
            sender,
            target,
            &mut queue,
            &mut visited,
            query_func,
            3,
        )
        .await?;

        dbg!(next_queue
            .iter()
            .map(|x| (x.id, distance(x.id, target)))
            .collect::<Vec<_>>());

        // Let's check if the best peers is better than the previous hop.
        let mut better_distance_found = false;
        if let Some(peer) = next_queue.first() {
            let distance = distance(peer.id, target);
            println!(
                "maxdist={} | peer={:04b}({}) target={:04b}({}) == {:04b}({})",
                best_distance, peer.id, peer.id, target, target, distance, distance
            );
            dbg!(best_distance, distance, peer.id, target);
            if distance < best_distance {
                best_distance = distance;
                better_distance_found = true;
            }
            if distance == 0 {
                found_peer = Some(peer.clone());
            }
        }

        queue.extend(next_queue);
        // Now let's sort the queue putting the best at the end (easier for
        // poping values).
        queue.sort_by_key(|peer| distance(peer.id, target));
        queue.reverse();
        queue.dedup_by_key(|peer| peer.id);
        dbg!(&queue);

        // Handle strategy here.
        if let Some(max_hop) = max_hop {
            // If we found the exact peer, put it in our dht, and stop searching.
            if let Some(peer) = &found_peer {
                let mut guard = ctx.lock().await;
                let ctx = guard.deref_mut();
                ctx.dht.add_node(peer.id, peer.addr).await;
                break;
            }
            // Hop strategy: stop after doing N hops, or if we found the target.
            // If all nodes have been visited, also stop.
            if hop >= max_hop || queue.is_empty() {
                break;
            }
        } else {
            // Classic strategy: stop when the next route is not closer to the
            // target than this one.
            // If the next group queried didn't return a better result, we stop
            // to hop.
            if !better_distance_found {
                break;
            }
        }
    }

    Ok(found_peer)
}

// Will drain N values from the main task queues, launch them in parallel, then
// return a flatten results of peers. Peers will be sorted by relevancy (closest
// first).
async fn parallel_find_node<F, T>(
    ctx: Arc<Mutex<Context>>,
    sender: u32,
    target: u32,
    queue: &mut Vec<Peer>,
    visited: &mut HashSet<u32>,
    mut query_func: F,
    nb_parallel: usize,
) -> AnyResult<Vec<Peer>>
where
    F: FnMut(Arc<Mutex<Context>>, Peer, u32, u32) -> T + Send + Copy + 'static,
    T: Future<Output = AnyResult<Vec<Peer>>> + Send + 'static,
{
    let mut next_queue = Vec::new();

    // Just launch 3 concurrent tasks.
    let mut queries = Vec::new();
    let mut nb_tasks = 0;

    while let Some(peer) = queue.pop() {
        if visited.contains(&peer.id) {
            continue;
        }

        nb_tasks += 1;
        let ctx = Arc::clone(&ctx);
        let handle = tokio::spawn(async move {
            let peer_id = peer.id;
            let peers = query_func(ctx, peer, sender, target).await?;
            Ok::<(u32, Vec<Peer>), AnyError>((peer_id, peers))
        });

        queries.push(handle);
        if nb_tasks >= nb_parallel {
            break;
        }
    }

    // Now wait for all tasks to complete and put result in a queue.
    for handle in queries {
        let (peer_id, peers) = handle.await??;
        visited.insert(peer_id);
        // Let's keep the 4 best nodes found.
        next_queue.extend(peers.into_iter());
    }

    // Keep only non-visited nodes.
    dbg!(&next_queue);
    next_queue.retain(|peer| !visited.contains(&peer.id));

    // Sort peers by relevancy (the closest first)
    next_queue.sort_by_key(|peer| distance(peer.id, target));
    next_queue.dedup_by_key(|peer| peer.id);

    Ok(next_queue)
}

// Query the distant nodes and update the current context.
async fn query_find_node(
    ctx: Arc<Mutex<Context>>,
    peer: Peer,
    sender: u32,
    target: u32,
) -> AnyResult<Vec<Peer>> {
    let stream = Arc::new(Mutex::new(TcpStream::connect(peer.addr).await?));
    let peers = handle_find_node(stream, sender, target).await?;

    // The peer just answered us, let's add him into our dht.
    {
        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.dht.add_node(peer.id, peer.addr).await;
    }

    Ok(peers)
}

// Ping a peer, and put it's id into our dht.
async fn ping(ctx: Arc<Mutex<Context>>, peer: Peer, sender: u32) -> AnyResult<u32> {
    let stream = Arc::new(Mutex::new(TcpStream::connect(peer.addr).await?));
    let target = handle_ping(stream, sender).await?;

    // The peer just answered us, let's add him into our dht.
    {
        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.dht.add_node(target, peer.addr).await;
    }

    Ok(target)
}

#[cfg(test)]
#[path = "manager_test.rs"]
mod manager_test;
