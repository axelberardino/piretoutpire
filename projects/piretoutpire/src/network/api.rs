use super::protocol::{Command, Peer};
use crate::manager::context::Context;
use errors::{bail, AnyResult};
use std::{net::SocketAddr, ops::Deref, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
    time::{sleep, timeout},
};

// UTILS -----------------------------------------------------------------------

// Read the buffer until reaching the end of the command, or EOF.
//
// By default, read_to_end wait for an EOF (which never happen in a stream), and
// read_exact raise an error if the remaining data to read is smaller than the
// given value. This macro use a small 8 Ko buffer to force reading until what
// we declared as an end of data (meaning if the last packet is smaller than the
// buffer, let's consider we reach the end).
#[macro_export]
macro_rules! read_all {
    ($reader:ident, $timeout:ident) => {{
        let mut res_buf: Vec<u8> = Vec::new();
        const BUF_SIZE: usize = 8 * 1024;
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
        while let Ok(bytes) = tokio::time::timeout($timeout, $reader.read(&mut buf[..])).await? {
            if bytes == 0 {
                break;
            }
            res_buf.extend_from_slice(&buf[..bytes]);
            if bytes < BUF_SIZE {
                break;
            }
        }
        res_buf
    }};
}

// Send a raw request u8 encoded, and wait for a respone.
// Return a raw buffer which must be interpreted.
pub async fn send_raw_unary(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    request: &[u8],
) -> AnyResult<Vec<u8>> {
    let (slowness, read_timeout, write_timeout) = {
        let guard = ctx.lock().await;
        let ctx = guard.deref();
        (ctx.slowness, ctx.read_timeout, ctx.write_timeout)
    };

    let mut guard = stream.lock().await;
    let (reader, writer) = guard.split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut writer = tokio::io::BufWriter::new(writer);

    if let Some(wait_time) = slowness {
        sleep(wait_time).await;
    }
    timeout(write_timeout, writer.write_all(request)).await??;
    timeout(write_timeout, writer.flush()).await??;

    let raw_chunk = read_all!(reader, read_timeout);
    let len = raw_chunk.len();
    if len == 0 {
        bail!("invalid buffer");
    }

    Ok(raw_chunk)
}

// API -------------------------------------------------------------------------
// All unary send a request and handle the response in the command handler.

// Ask for a chunk of a given file by its id.
pub async fn file_chunk(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    crc: u32,
    chunk_id: u32,
) -> AnyResult<Command> {
    let request: Vec<u8> = Command::ChunkRequest(crc, chunk_id).into();
    let raw_response = send_raw_unary(ctx, stream, request.as_slice()).await?;
    raw_response.as_slice().try_into()
}

// Ask for a chunk of a given file by its id.
pub async fn file_info(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    crc: u32,
) -> AnyResult<Command> {
    let request: Vec<u8> = Command::FileInfoRequest(crc).into();
    let raw_response = send_raw_unary(ctx, stream, request.as_slice()).await?;
    raw_response.as_slice().try_into()
}

// Search for a given peer.
pub async fn find_node(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    sender_addr: SocketAddr,
    sender_id: u32,
    target: u32,
) -> AnyResult<Command> {
    let peer = Peer {
        id: sender_id,
        addr: sender_addr,
    };

    let request: Vec<u8> = Command::FindNodeRequest(peer, target).into();
    let raw_response = send_raw_unary(ctx, stream, request.as_slice()).await?;
    raw_response.as_slice().try_into()
}

// Ping a peer, checking if he's alive and get its id.
pub async fn ping(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    sender_addr: SocketAddr,
    sender_id: u32,
) -> AnyResult<Command> {
    let peer = Peer {
        id: sender_id,
        addr: sender_addr,
    };

    let request: Vec<u8> = Command::PingRequest(peer).into();
    let raw_response = send_raw_unary(ctx, stream, request.as_slice()).await?;
    raw_response.as_slice().try_into()
}

// Store a value on a peer.
pub async fn store(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    sender_addr: SocketAddr,
    sender_id: u32,
    key: u32,
    value: String,
) -> AnyResult<Command> {
    let peer = Peer {
        id: sender_id,
        addr: sender_addr,
    };

    let request: Vec<u8> = Command::StoreRequest(peer, key, value).into();
    let raw_response = send_raw_unary(ctx, stream, request.as_slice()).await?;
    raw_response.as_slice().try_into()
}

// Search a given value on a peer.
pub async fn find_value(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    sender_addr: SocketAddr,
    sender_id: u32,
    key: u32,
) -> AnyResult<Command> {
    let peer = Peer {
        id: sender_id,
        addr: sender_addr,
    };

    let request: Vec<u8> = Command::FindValueRequest(peer, key).into();
    let raw_response = send_raw_unary(ctx, stream, request.as_slice()).await?;
    raw_response.as_slice().try_into()
}

// Send a message to a peer.
pub async fn send_message(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    message: String,
) -> AnyResult<Command> {
    let request: Vec<u8> = Command::MessageRequest(message).into();
    let raw_response = send_raw_unary(ctx, stream, request.as_slice()).await?;
    raw_response.as_slice().try_into()
}

// Send to a peer that a given peer own a file (by its crc).
pub async fn announce(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    sender_addr: SocketAddr,
    sender_id: u32,
    crc: u32,
) -> AnyResult<Command> {
    let peer = Peer {
        id: sender_id,
        addr: sender_addr,
    };

    let request: Vec<u8> = Command::AnnounceRequest(peer, crc).into();
    let raw_response = send_raw_unary(ctx, stream, request.as_slice()).await?;
    raw_response.as_slice().try_into()
}

// Get the list of peers who own a given file (by its crc).
pub async fn get_peers(
    ctx: Arc<Mutex<Context>>,
    stream: Arc<Mutex<TcpStream>>,
    crc: u32,
) -> AnyResult<Command> {
    let request: Vec<u8> = Command::GetPeersRequest(crc).into();
    let raw_response = send_raw_unary(ctx, stream, request.as_slice()).await?;
    raw_response.as_slice().try_into()
}
