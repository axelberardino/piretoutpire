use super::context::Context;
use crate::{
    file::{file_chunk::FileChunk, torrent_file::TorrentFile},
    network::{
        api::{find_node, get_chunk, handshake},
        protocol::Command,
    },
};
use errors::{bail, AnyResult};
use std::{ops::DerefMut, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex};

// Client API ------------------------------------------------------------------

// Get file info from its ID (crc).
// Start by sending a Handshake request, and received either an error code
// or a FileInfo response.
pub async fn handle_file_info(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    crc: u32,
) -> AnyResult<()> {
    let command = handshake(Arc::clone(&stream), crc).await?;

    match command {
        Command::FileInfo(file_info) => {
            eprintln!("received file_info {:?}", &file_info);
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
        }
        Command::ErrorOccured(error) => eprintln!("peer return error: {}", error),
        _ => bail!("Wrong command received: {:?}", command),
    }
    Ok(())
}

// Ask for a given file chunk.
// Start by sending a GetChunk request, and received either an error code
// or the chunk as a raw buffer.
pub async fn handle_get_chunk(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    crc: u32,
    chunk_id: u32,
) -> AnyResult<()> {
    let command = get_chunk(Arc::clone(&stream), crc, chunk_id).await?;

    match command {
        Command::SendChunk(crc, chunk_id, raw_chunk) => {
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
        }
        Command::ErrorOccured(error) => eprintln!("peer return error: {}", error),
        _ => bail!("Wrong command received: {:?}", command),
    }

    Ok(())
}

// Ask for a node in the DHT.
pub async fn handle_find_node(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    sender: u32,
    target: u32,
) -> AnyResult<()> {
    let command = find_node(Arc::clone(&stream), sender, target).await?;

    match command {
        Command::FindNodeResponse(peers_found) => {
            eprintln!("received find node {:?}", &peers_found);
            let mut guard = ctx.lock().await;
            let _ctx = guard.deref_mut();
            // ctx.dht.find_node(sender, target)
        }
        Command::ErrorOccured(error) => eprintln!("peer return error: {}", error),
        _ => bail!("Wrong command received: {:?}", command),
    }
    Ok(())

    // let mut guard = stream.lock().await;
    // let (_, writer) = guard.split();
    // let mut writer = tokio::io::BufWriter::new(writer);
    // drop(guard);
}
