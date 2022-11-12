use super::protocol::Command;
use errors::{bail, AnyResult};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
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
    ($reader:ident) => {{
        let mut res_buf: Vec<u8> = Vec::new();
        const BUF_SIZE: usize = 8 * 1024;
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
        while let Ok(bytes) = $reader.read(&mut buf[..]).await {
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
pub async fn send_raw_unary(stream: Arc<Mutex<TcpStream>>, request: &[u8]) -> AnyResult<Vec<u8>> {
    let mut guard = stream.lock().await;
    let (reader, writer) = guard.split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut writer = tokio::io::BufWriter::new(writer);

    writer.write_all(request).await?;
    writer.flush().await?;

    let raw_chunk = read_all!(reader);
    let len = raw_chunk.len();
    if len == 0 {
        bail!("invalid buffer");
    }

    Ok(raw_chunk)
}

// API -------------------------------------------------------------------------
// All unary send a request and handle the response in the command handler.

// Ask for a chunk of a given file by its id.
pub async fn get_chunk(stream: Arc<Mutex<TcpStream>>, crc: u32, chunk_id: u32) -> AnyResult<Command> {
    let request: Vec<u8> = Command::GetChunk(crc, chunk_id).into();
    let raw_response = send_raw_unary(stream, request.as_slice()).await?;
    raw_response.as_slice().try_into()
}

// Ask for a chunk of a given file by its id.
pub async fn handshake(stream: Arc<Mutex<TcpStream>>, crc: u32) -> AnyResult<Command> {
    let request: Vec<u8> = Command::Handshake(crc).into();
    let raw_response = send_raw_unary(stream, request.as_slice()).await?;
    raw_response.as_slice().try_into()
}

// // Get file info from its ID (crc).
// // Start by sending a Handshake request, and received either an error code
// // or a FileInfo response.
// pub async fn find_node(stream: TcpStream, sender: u32, target: u32) -> AnyResult<()> {
//     let request: Vec<u8> = Command::FindNodeRequest(sender, target).into();
//     let raw_response = send_raw_unary(stream, request.as_slice()).await?;

//     // FIXME handle response
//     Ok(())
// }
