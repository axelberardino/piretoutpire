use super::context::Context;
use crate::{
    dht::peer_node::PeerNode,
    network::protocol::{Command, ErrorCode, FileInfo, Peer},
};
use std::{net::SocketAddr, ops::DerefMut, sync::Arc};
use tokio::sync::Mutex;

// Server API ------------------------------------------------------------------

// FIXME: rename? Handle handshake.
pub async fn serve_handshake(ctx: Arc<Mutex<Context>>, crc: u32) -> Command {
    eprintln!("handshake, ask for crc {}", crc);
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();

    match ctx.available_torrents.get(&crc) {
        Some((torrent, _)) => {
            let file_info = FileInfo {
                file_size: torrent.metadata.file_size,
                chunk_size: torrent.metadata.chunk_size,
                file_crc: torrent.metadata.file_crc,
                original_filename: torrent.metadata.original_filename.clone(),
            };
            Command::FileInfo(file_info)
        }
        None => Command::ErrorOccured(ErrorCode::FileNotFound),
    }
}

// Serve chunk asked by a client.
pub async fn serve_get_chunk(ctx: Arc<Mutex<Context>>, crc: u32, chunk_id: u32) -> Command {
    eprintln!("applying get_chunk {}/{}", crc, chunk_id);
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();

    match ctx.available_torrents.get_mut(&crc) {
        Some((torrent, chunks)) => {
            if chunk_id as usize >= torrent.metadata.completed_chunks.len() {
                Command::ErrorOccured(ErrorCode::InvalidChunk)
            } else {
                match chunks.read_chunk(chunk_id) {
                    Ok(chunk) => Command::SendChunk(crc, chunk_id, chunk),
                    Err(_) => Command::ErrorOccured(ErrorCode::ChunkNotFound),
                }
            }
        }
        None => Command::ErrorOccured(ErrorCode::FileNotFound),
    }
}

// Serve find a node. Will return the 3 closest node from the provided one.
pub async fn serve_find_node(
    ctx: Arc<Mutex<Context>>,
    sender_addr: SocketAddr,
    sender: u32,
    target: u32,
) -> Command {
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();
    let peers = ctx
        .dht
        .find_node(PeerNode::new(sender, sender_addr), target)
        .await
        .map(|peer| Peer {
            id: peer.id(),
            addr: peer.addr(),
        })
        .collect::<Vec<_>>();

    eprintln!(
        "received find_node from {}({}) for {} and send back {:?}",
        sender, sender_addr, target, peers
    );
    Command::FindNodeResponse(peers)
}
