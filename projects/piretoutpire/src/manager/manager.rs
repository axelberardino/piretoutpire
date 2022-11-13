use super::{
    client::{handle_file_chunk, handle_file_info, handle_find_node},
    command_handler::listen_to_command,
    context::Context,
};
use crate::{
    file::{file_chunk::FileChunk, torrent_file::TorrentFile},
    network::protocol::Peer,
    utils::distance,
};
use errors::{reexports::eyre::ContextCompat, AnyError, AnyResult};
use std::{collections::HashSet, net::SocketAddr, ops::DerefMut, path::Path, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

pub struct Manager {
    id: u32,
    addr: SocketAddr,
    ctx: Arc<Mutex<Context>>,
}

impl Manager {
    // Expect an address like: "127.0.0.1:8080".parse()
    pub fn new(id: u32, addr: SocketAddr, working_directory: String) -> Self {
        Self {
            id,
            addr,
            ctx: Arc::new(Mutex::new(Context::new(working_directory, id))),
        }
    }

    // Start to bootstrap the DHT from an entry point (any available peer).
    pub async fn bootstrap(&mut self, peer_addr: SocketAddr) -> AnyResult<()> {
        find_closest_node(
            Arc::clone(&self.ctx),
            Peer {
                id: u32::MAX,
                addr: peer_addr,
            },
            self.id,
            self.id,
        )
        .await?;
        Ok(())
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

// Send for a requested node until finding it. Will stop if the most closest
// ones found in a row are not closer.
// Return either the found peer or none.
async fn find_closest_node(
    ctx: Arc<Mutex<Context>>,
    initial_peer: Peer,
    sender: u32,
    target: u32,
) -> AnyResult<Option<Peer>> {
    let mut queue = vec![initial_peer];
    let mut visited = HashSet::<u32>::new();
    let mut best_distance = u32::MAX;
    let mut found_peer = None::<Peer>;

    loop {
        let mut best_distance_found = false;
        let mut next_queue = Vec::new();

        let mut queries = Vec::new();
        for peer in queue.drain(..) {
            if visited.contains(&peer.id) {
                continue;
            }

            let ctx = Arc::clone(&ctx);
            let handle = tokio::spawn(async move {
                let peer_id = peer.id;
                // FIXME: mmock that
                let peers = query_find_node(ctx, peer, sender, target).await?;
                Ok::<(u32, Vec<Peer>), AnyError>((peer_id, peers))
            });

            queries.push(handle);
        }

        for handle in queries {
            let (peer_id, peers) = handle.await??;
            visited.insert(peer_id);
            // Let's keep the 4 best nodes found.
            next_queue.extend(peers.into_iter());
        }

        // Sort peers by relevancy (the closest first), and only keep the 4 best
        // unique ones.
        next_queue.sort_by_key(|peer| distance(peer.id, target));
        next_queue.dedup_by_key(|peer| peer.id);
        next_queue = next_queue.into_iter().take(4).collect();

        // Let's check if the best peers is better than the previous hop.
        if let Some(peer) = next_queue.first() {
            let distance = distance(peer.id, target);
            if distance < best_distance {
                best_distance = distance;
                best_distance_found = true;
            }
            if distance == 0 {
                found_peer = Some(peer.clone());
            }
        }

        // If the next group queried didn't return a better result, we stop to hop.
        if !best_distance_found {
            break;
        }
    }

    Ok(found_peer)
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

#[cfg(test)]
#[path = "manager_test.rs"]
mod manager_test;
