use super::context::Context;
use crate::{
    dht::peer_node::PeerNode,
    network::protocol::{Command, ErrorCode, FileInfo, Peer},
};
use std::{net::SocketAddr, ops::DerefMut, sync::Arc};
use tokio::sync::Mutex;

// Server API ------------------------------------------------------------------

// FIXME: rename? Handle handshake.
pub async fn serve_file_info(ctx: Arc<Mutex<Context>>, sender_addr: SocketAddr, crc: u32) -> Command {
    eprintln!("file_info received from {}({})", crc, sender_addr);
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
            Command::FileInfoResponse(file_info)
        }
        None => Command::ErrorOccured(ErrorCode::FileNotFound),
    }
}

// Serve chunk asked by a client.
pub async fn serve_file_chunk(
    ctx: Arc<Mutex<Context>>,
    sender_addr: SocketAddr,
    crc: u32,
    chunk_id: u32,
) -> Command {
    eprintln!(
        "file_chunk received from {} asking for {}/{}",
        sender_addr, crc, chunk_id
    );
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();

    match ctx.available_torrents.get_mut(&crc) {
        Some((torrent, chunks)) => {
            if chunk_id as usize >= torrent.metadata.completed_chunks.len() {
                Command::ErrorOccured(ErrorCode::InvalidChunk)
            } else {
                match chunks.read_chunk(chunk_id) {
                    Ok(chunk) => Command::ChunkResponse(crc, chunk_id, chunk),
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

// Received the sender id, and response with this server id.
pub async fn serve_ping(ctx: Arc<Mutex<Context>>, sender_addr: SocketAddr, crc: u32) -> Command {
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();

    let own_id = ctx.dht.id();
    eprintln!("received ping from {}({}), sending {}", crc, sender_addr, own_id);
    Command::PingResponse(own_id)
}

// Allow a client to put a value inside this server.
pub async fn serve_store(
    ctx: Arc<Mutex<Context>>,
    sender_addr: SocketAddr,
    key: u32,
    message: String,
) -> Command {
    eprintln!("received store from ({}) for {}={}", sender_addr, key, message);
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();

    //FIXME

    Command::StoreResponse()
}

// Allow a client to put a value inside this server.
pub async fn serve_find_value(ctx: Arc<Mutex<Context>>, sender_addr: SocketAddr, key: u32) -> Command {
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();

    //FIXME + ErrorOccured(KeyNotFound)
    let msg = "FIXME".to_owned();
    eprintln!("received find_value from ({}) for {}={}", sender_addr, key, msg);

    Command::FindValueResponse(msg)
}

// Display the message the user send.
pub async fn serve_message(ctx: Arc<Mutex<Context>>, sender_addr: SocketAddr, message: String) -> Command {
    eprintln!("{} send us this message {}", sender_addr, message);
    Command::MessageResponse()
}
