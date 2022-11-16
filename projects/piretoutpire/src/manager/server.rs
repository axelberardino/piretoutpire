use super::context::Context;
use crate::{
    dht::peer_node::PeerNode,
    network::protocol::{Command, ErrorCode, FileInfo, Peer},
};
use colored::Colorize;
use std::{net::SocketAddr, ops::DerefMut, sync::Arc};
use tokio::sync::Mutex;

// Server API ------------------------------------------------------------------

// Pretty prints the server logs
macro_rules! log {
    () => {
        print!("\n")
    };
    ($header:ident, $fmt:literal, $($args:expr),* $(,)?) => {
        let head = format!("{}", $header);
        let msg = format!($fmt, $($args),+);
        println!("{}{}", head, msg.green())
    };
}

// Give the file metadata information given its id/crc.
pub async fn serve_file_info(ctx: Arc<Mutex<Context>>, incoming_addr: SocketAddr, crc: u32) -> Command {
    let header = "[FILE_INFO]".to_owned().blue().on_truecolor(35, 38, 39).bold();
    let prefix = format!(" asked crc {} from {}", crc, incoming_addr,);

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
            log!(header, "{}, and send back {:?}", prefix, file_info);
            Command::FileInfoResponse(file_info)
        }
        None => {
            log!(header, "{}, but can't found the resource", prefix);
            Command::ErrorOccured(ErrorCode::FileNotFound)
        }
    }
}

// Serve chunk asked by a client.
pub async fn serve_file_chunk(
    ctx: Arc<Mutex<Context>>,
    incoming_addr: SocketAddr,
    crc: u32,
    chunk_id: u32,
) -> Command {
    let header = "[CHUNK]".to_owned().blue().on_truecolor(35, 38, 39).bold();
    let prefix = format!(" asked by {} asking for {}/{}", incoming_addr, crc, chunk_id);

    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();

    match ctx.available_torrents.get_mut(&crc) {
        Some((torrent, chunks)) => {
            if chunk_id as usize >= torrent.metadata.completed_chunks.len() {
                log!(header, "{}, but chunk was invalid", prefix);
                Command::ErrorOccured(ErrorCode::InvalidChunk)
            } else {
                match chunks.read_chunk(chunk_id) {
                    Ok(chunk) => {
                        log!(header, "{}, and send back {} bytes", prefix, chunk.len());
                        Command::ChunkResponse(crc, chunk_id, chunk)
                    }
                    Err(_) => {
                        log!(header, "{}, but chunk was not found", prefix);
                        Command::ErrorOccured(ErrorCode::ChunkNotFound)
                    }
                }
            }
        }
        None => {
            log!(header, "{}, but file was not found", prefix);
            Command::ErrorOccured(ErrorCode::FileNotFound)
        }
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

    let header = "[FIND_NODE]".to_owned().blue().on_truecolor(35, 38, 39).bold();
    log!(
        header,
        " asked by {}({}) for {} and send back {:?}",
        sender,
        sender_addr,
        target,
        peers
    );
    Command::FindNodeResponse(peers)
}

// Received the sender id, and response with this server id.
pub async fn serve_ping(ctx: Arc<Mutex<Context>>, sender_addr: SocketAddr, sender_id: u32) -> Command {
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();

    let own_id = ctx.dht.id();
    ctx.dht.add_node(sender_id, sender_addr).await;

    let header = "[PING]".to_owned().blue().on_truecolor(35, 38, 39).bold();
    log!(
        header,
        " asked by {}({}), sending this server peer id {}",
        sender_id,
        sender_addr,
        own_id
    );
    Command::PingResponse(own_id)
}

// Allow a client to put a value inside this server.
pub async fn serve_store(
    ctx: Arc<Mutex<Context>>,
    sender_addr: SocketAddr,
    sender_id: u32,
    key: u32,
    message: String,
) -> Command {
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();

    let header = "[STORE_VALUE]".to_owned().blue().on_truecolor(35, 38, 39).bold();
    log!(
        header,
        " asked by {}({}), store {}={}",
        sender_id,
        sender_addr,
        key,
        &message
    );

    ctx.dht.store_value(key, message);
    ctx.dht.add_node(sender_id, sender_addr).await;

    Command::StoreResponse()
}

// Allow a client to put a value inside this server.
pub async fn serve_find_value(
    ctx: Arc<Mutex<Context>>,
    sender_addr: SocketAddr,
    sender_id: u32,
    key: u32,
) -> Command {
    let header = "[GET]".to_owned().blue().on_truecolor(35, 38, 39).bold();
    let prefix = format!(" {}({}) ask for {}", sender_id, sender_addr, key,);

    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();
    ctx.dht.add_node(sender_id, sender_addr).await;
    let message = ctx.dht.get_value(key);

    match message {
        Some(message) => {
            log!(header, "{}={}", prefix, message);
            Command::FindValueResponse(message.clone())
        }
        None => {
            log!(header, "{}, but the key was not found", prefix);
            Command::ErrorOccured(ErrorCode::KeyNotFound)
        }
    }
}

// Display the message the user send.
pub async fn serve_message(_: Arc<Mutex<Context>>, incoming_addr: SocketAddr, message: String) -> Command {
    let header = "[MESSAGE]".to_owned().yellow().on_truecolor(35, 38, 39).bold();
    log!(header, " user {}, send: \"{}\"", incoming_addr, message);

    Command::MessageResponse()
}

// Announce is a way to tell a peer that a given user own a file (by given the
// crc as an identifier).
pub async fn serve_announce(
    ctx: Arc<Mutex<Context>>,
    sender_addr: SocketAddr,
    sender_id: u32,
    crc: u32,
) -> Command {
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();

    let header = "[ANNOUNCE]".to_owned().blue().on_truecolor(35, 38, 39).bold();
    log!(
        header,
        " peer {}({}), announce: he has the file {}",
        sender_addr,
        sender_id,
        crc
    );

    ctx.dht.store_file_peer(
        crc,
        Peer {
            id: sender_id,
            addr: sender_addr,
        },
    );
    ctx.dht.add_node(sender_id, sender_addr).await;

    Command::AnnounceResponse()
}

// Get the list of all peers which are sharing a file, given its id/crc.
pub async fn serve_get_peers(ctx: Arc<Mutex<Context>>, incoming_addr: SocketAddr, crc: u32) -> Command {
    let header = "[GET_PEERS]".to_owned().blue().on_truecolor(35, 38, 39).bold();
    let prefix = format!(" peer {}, ask peers for {}", incoming_addr, crc);

    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();
    let peers = ctx.dht.get_file_peers(crc);

    match peers {
        Some(peers) => {
            let peers = peers.map(Clone::clone).collect();
            log!(header, "{}={:?}", prefix, peers);
            Command::GetPeersResponse(peers)
        }
        None => {
            log!(header, "{}, but the key was not found", prefix);
            Command::ErrorOccured(ErrorCode::FileNotFound)
        }
    }
}
