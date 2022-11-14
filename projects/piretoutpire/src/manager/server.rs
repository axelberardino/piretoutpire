use super::context::Context;
use crate::{
    dht::peer_node::PeerNode,
    network::protocol::{Command, ErrorCode, FileInfo, Peer},
};
use colored::Colorize;
use std::{net::SocketAddr, ops::DerefMut, sync::Arc};
use tokio::sync::Mutex;

// Server API ------------------------------------------------------------------

// FIXME: rename? Handle handshake.
pub async fn serve_file_info(ctx: Arc<Mutex<Context>>, sender_addr: SocketAddr, crc: u32) -> Command {
    let prefix = format!(
        "{} received crc {} from {}",
        "[FILE_INFO]".to_owned().blue().on_truecolor(35, 38, 39).bold(),
        crc,
        sender_addr,
    );

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
            println!("{}, and send back {:?}", prefix, file_info);
            Command::FileInfoResponse(file_info)
        }
        None => {
            println!("{}, but can't found the resource", prefix);
            Command::ErrorOccured(ErrorCode::FileNotFound)
        }
    }
}

// Serve chunk asked by a client.
pub async fn serve_file_chunk(
    ctx: Arc<Mutex<Context>>,
    sender_addr: SocketAddr,
    crc: u32,
    chunk_id: u32,
) -> Command {
    let prefix = format!(
        "{} received from {} asking for {}/{}",
        "[CHUNK]".to_owned().blue().on_truecolor(35, 38, 39).bold(),
        sender_addr,
        crc,
        chunk_id,
    );
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();

    match ctx.available_torrents.get_mut(&crc) {
        Some((torrent, chunks)) => {
            if chunk_id as usize >= torrent.metadata.completed_chunks.len() {
                println!("{}, but chunk was invalid", prefix);
                Command::ErrorOccured(ErrorCode::InvalidChunk)
            } else {
                match chunks.read_chunk(chunk_id) {
                    Ok(chunk) => {
                        println!("{}, and send back {} bytes", prefix, chunk.len());
                        Command::ChunkResponse(crc, chunk_id, chunk)
                    }
                    Err(_) => {
                        println!("{}, but chunk was not found", prefix);
                        Command::ErrorOccured(ErrorCode::ChunkNotFound)
                    }
                }
            }
        }
        None => {
            println!("{}, but file was not found", prefix);
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

    println!(
        "{} received from {}({}) for {} and send back {:?}",
        "[FIND_NODE]".to_owned().blue().on_truecolor(35, 38, 39).bold(),
        sender,
        sender_addr,
        target,
        peers
    );
    Command::FindNodeResponse(peers)
}

// Received the sender id, and response with this server id.
pub async fn serve_ping(ctx: Arc<Mutex<Context>>, sender_addr: SocketAddr, crc: u32) -> Command {
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();

    let own_id = ctx.dht.id();
    println!(
        "{} received from {}({}), sending {}",
        "[PING]".to_owned().blue().on_truecolor(35, 38, 39).bold(),
        crc,
        sender_addr,
        own_id
    );
    Command::PingResponse(own_id)
}

// Allow a client to put a value inside this server.
pub async fn serve_store(
    ctx: Arc<Mutex<Context>>,
    sender_addr: SocketAddr,
    key: u32,
    message: String,
) -> Command {
    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();
    println!(
        "{} received from {}, store {}={}",
        "[STORE_VALUE]".to_owned().blue().on_truecolor(35, 38, 39).bold(),
        sender_addr,
        key,
        &message
    );
    ctx.dht.store_value(key, message);

    Command::StoreResponse()
}

// Allow a client to put a value inside this server.
pub async fn serve_find_value(ctx: Arc<Mutex<Context>>, sender_addr: SocketAddr, key: u32) -> Command {
    let prefix = format!(
        "{} {} ask for {}",
        "[GET]".to_owned().blue().on_truecolor(35, 38, 39).bold(),
        sender_addr,
        key,
    );

    let mut guard = ctx.lock().await;
    let ctx = guard.deref_mut();
    let message = ctx.dht.get_value(key);

    match message {
        Some(message) => {
            println!("{}={}", prefix, message);
            Command::FindValueResponse(message.clone())
        }
        None => {
            println!("{}, but the key was not found", prefix);
            Command::ErrorOccured(ErrorCode::KeyNotFound)
        }
    }
}

// Display the message the user send.
pub async fn serve_message(_: Arc<Mutex<Context>>, sender_addr: SocketAddr, message: String) -> Command {
    println!(
        "{} user {}, send: \"{}\"",
        "[MESSAGE]".to_owned().yellow().on_truecolor(35, 38, 39).bold(),
        sender_addr,
        message
    );

    Command::MessageResponse()
}
