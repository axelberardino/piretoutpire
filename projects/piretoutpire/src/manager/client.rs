use super::context::Context;
use crate::{
    file::{file_chunk::FileChunk, torrent_file::TorrentFile},
    network::{
        api::{announce, file_chunk, file_info, find_node, find_value, get_peers, ping, send_message, store},
        protocol::{Command, ErrorCode, FileInfo, Peer},
    },
};
use errors::{bail, AnyResult};
use std::{net::SocketAddr, ops::DerefMut, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex};

// Helpers ---------------------------------------------------------------------

// Mark that we're trying to contact a given peer.
async fn peer_was_requested(ctx: Arc<Mutex<Context>>, target: u32) {
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();
    ctx.dht.peer_was_requested(target).await;
}

// Mark that we succeed to contact a given peer.
async fn peer_has_responded(ctx: Arc<Mutex<Context>>, target: u32) {
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();
    ctx.dht.peer_has_responded(target).await;
}

// Client API ------------------------------------------------------------------

// Get file info from its ID (crc), then put it into our local store.
pub async fn handle_file_info(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    crc: u32,
) -> AnyResult<Option<FileInfo>> {
    let command = file_info(Arc::clone(&ctx), Arc::clone(&stream), crc).await?;

    match command {
        Command::FileInfoResponse(file_info) => {
            let mut guard = ctx.lock().await;
            let ctx = guard.deref_mut();

            match ctx.available_torrents.entry(file_info.file_crc) {
                std::collections::hash_map::Entry::Occupied(entry) => {
                    eprintln!("already got the asked torrent {}", entry.key());
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    let torrent = TorrentFile::preallocate(
                        format!(
                            "{}/{}.metadata",
                            ctx.working_directory, file_info.original_filename
                        ),
                        file_info.original_filename.clone(),
                        file_info.file_size,
                        file_info.file_crc,
                        file_info.chunk_size,
                    );
                    torrent.dump()?;
                    let chunks = FileChunk::open_new(
                        format!("{}/{}", ctx.working_directory, file_info.original_filename),
                        file_info.file_size,
                    )?;

                    entry.insert((torrent, chunks));
                }
            }
            Ok(Some(file_info))
        }
        Command::ErrorOccured(error) if error == ErrorCode::FileNotFound => Ok(None),
        Command::ErrorOccured(error) => bail!("peer return error: {}", error),
        _ => bail!("Wrong command received: {:?}", command),
    }
}

// Ask for a given file chunk.
// Start by sending a GetChunk request, and received either an error code
// or the chunk as a raw buffer.
pub async fn handle_file_chunk(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    crc: u32,
    chunk_id: u32,
) -> AnyResult<bool> {
    let command = file_chunk(Arc::clone(&ctx), Arc::clone(&stream), crc, chunk_id).await?;

    match command {
        Command::ChunkResponse(crc, chunk_id, raw_chunk) => {
            eprintln!("received buf {:?}", &raw_chunk);
            let mut guard = ctx.lock().await;
            let ctx = guard.deref_mut();

            match ctx.available_torrents.get_mut(&crc) {
                Some((torrent, chunks)) => {
                    if chunk_id as usize >= torrent.metadata.completed_chunks.len() {
                        eprintln!("Invalid chunk_id {} for {}", chunk_id, crc);
                    } else {
                        // Update metadata and write local chunk.
                        torrent.metadata.completed_chunks[chunk_id as usize] =
                            Some(crc32fast::hash(&raw_chunk));
                        torrent.dump()?;
                        chunks.write_chunk(chunk_id, raw_chunk.as_slice())?;
                    }
                }
                None => eprintln!("Got chunk for an unknown file"),
            }
            Ok(true)
        }
        Command::ErrorOccured(error) if error == ErrorCode::ChunkNotFound => Ok(false),
        Command::ErrorOccured(error) if error == ErrorCode::FileNotFound => Ok(false),
        _ => bail!("Wrong command received: {:?}", command),
    }
}

// Ask for a node in the DHT.
pub async fn handle_find_node(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    sender_addr: SocketAddr,
    sender_id: u32,
    target: u32,
) -> AnyResult<Vec<Peer>> {
    peer_was_requested(Arc::clone(&ctx), sender_id).await;
    let command = find_node(
        Arc::clone(&ctx),
        Arc::clone(&stream),
        sender_addr,
        sender_id,
        target,
    )
    .await?;
    peer_has_responded(Arc::clone(&ctx), sender_id).await;

    match command {
        Command::FindNodeResponse(peers_found) => Ok(peers_found),
        Command::ErrorOccured(error) => bail!("peer return error: {}", error),
        _ => bail!("Wrong command received: {:?}", command),
    }
}

// Ask a peer for it's id, and check if he's alive.
pub async fn handle_ping(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    sender_addr: SocketAddr,
    sender_id: u32,
) -> AnyResult<u32> {
    peer_was_requested(Arc::clone(&ctx), sender_id).await;
    let command = ping(Arc::clone(&ctx), Arc::clone(&stream), sender_addr, sender_id).await?;
    peer_has_responded(Arc::clone(&ctx), sender_id).await;

    match command {
        Command::PingResponse(target) => Ok(target),
        Command::ErrorOccured(error) => bail!("peer return error: {}", error),
        _ => bail!("Wrong command received: {:?}", command),
    }
}

// Ask a peer to store a value ina given key.
pub async fn handle_store(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    sender_addr: SocketAddr,
    sender_id: u32,
    key: u32,
    value: String,
) -> AnyResult<()> {
    let command = store(
        Arc::clone(&ctx),
        Arc::clone(&stream),
        sender_addr,
        sender_id,
        key,
        value,
    )
    .await?;

    match command {
        Command::StoreResponse() => Ok(()),
        Command::ErrorOccured(error) => bail!("peer return error: {}", error),
        _ => bail!("Wrong command received: {:?}", command),
    }
}

// Ask a peer for a store value in its kv_store, for a given key.
pub async fn handle_find_value(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    sender_addr: SocketAddr,
    sender_id: u32,
    key: u32,
) -> AnyResult<Option<String>> {
    let command = find_value(Arc::clone(&ctx), Arc::clone(&stream), sender_addr, sender_id, key).await?;

    match command {
        Command::FindValueResponse(message) => Ok(Some(message)),
        Command::ErrorOccured(ErrorCode::KeyNotFound) => Ok(None),
        Command::ErrorOccured(error) => bail!("peer return error: {}", error),
        _ => bail!("Wrong command received: {:?}", command),
    }
}

// Send a message to a peer.
pub async fn handle_message(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    message: String,
) -> AnyResult<()> {
    let command = send_message(Arc::clone(&ctx), Arc::clone(&stream), message).await?;

    match command {
        Command::MessageResponse() => Ok(()),
        Command::ErrorOccured(error) => bail!("peer return error: {}", error),
        _ => bail!("Wrong command received: {:?}", command),
    }
}

// Send to a peer that a given peer own a file (by its crc).
pub async fn handle_announce(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    sender_addr: SocketAddr,
    sender_id: u32,
    crc: u32,
) -> AnyResult<()> {
    peer_was_requested(Arc::clone(&ctx), sender_id).await;
    let command = announce(Arc::clone(&ctx), Arc::clone(&stream), sender_addr, sender_id, crc).await?;
    peer_was_requested(Arc::clone(&ctx), sender_id).await;

    match command {
        Command::AnnounceResponse() => Ok(()),
        Command::ErrorOccured(error) => bail!("peer return error: {}", error),
        _ => bail!("Wrong command received: {:?}", command),
    }
}

// Get the list of peers who own a given file (by its crc).
pub async fn handle_get_peers(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    crc: u32,
) -> AnyResult<Option<Vec<Peer>>> {
    let command = get_peers(Arc::clone(&ctx), Arc::clone(&stream), crc).await?;

    match command {
        Command::GetPeersResponse(found_peers) => Ok(Some(found_peers)),
        Command::ErrorOccured(error) if error == ErrorCode::FileNotFound => Ok(None),
        Command::ErrorOccured(error) => bail!("peer return error: {}", error),
        _ => bail!("Wrong command received: {:?}", command),
    }
}
