use crate::{
    file::{
        file_chunk::{FileChunk, DEFAULT_CHUNK_SIZE},
        torrent_file::TorrentFile,
    },
    network::protocol::Command,
};
use errors::{bail, AnyResult};
use std::{
    collections::HashMap,
    io::BufReader,
    net::SocketAddr,
    ops::DerefMut,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener, TcpStream,
    },
};

pub struct Manager {
    addr: SocketAddr,
    ctx: Arc<Mutex<Context>>,
}

pub struct Context {
    // Realtime list of peers
    pub peers: HashMap<u32, SocketAddr>,
    // Metadata
    pub torrent: Option<TorrentFile<String>>,
    // File
    pub chunks: Option<FileChunk>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            torrent: None,
            chunks: None,
        }
    }
}

impl Manager {
    // Expect an address like: "127.0.0.1:8080".parse()
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            ctx: Arc::new(Mutex::new(Context::new())),
        }
    }

    // Start downloading a file, or resume downloading
    pub async fn download_file(&mut self, crc: u32) -> AnyResult<()> {
        let client_addr: SocketAddr = "127.0.0.1:4000".parse()?;
        let stream = TcpStream::connect(client_addr).await?;

        let ctx = Arc::clone(&self.ctx);
        let handle = tokio::spawn(async move { ask_for_chunks(ctx, stream).await });
        // self.start_stream().await?;
        handle.await?
    }

    // Start to share a file on the peer network, as a seeder.
    pub async fn share_existing_file<P: AsRef<Path>>(&mut self, file: P) -> AnyResult<()> {
        let torrent = TorrentFile::new(
            file.as_ref().display().to_string() + ".metadata",
            file.as_ref().display().to_string(),
        )?;
        let chunks = FileChunk::open_existing(&torrent.metadata.original_file)?;
        {
            let mut ctx = self.ctx.lock().expect("invalid mutex");
            ctx.torrent = Some(torrent);
            ctx.chunks = Some(chunks);
        }

        self.start_stream().await
    }

    async fn start_stream(&self) -> AnyResult<()> {
        let listener = TcpListener::bind(self.addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;
            let ctx = Arc::clone(&self.ctx);
            tokio::spawn(async move { handle_connection(ctx, stream).await });
        }
    }
}

// async fn read_all(reader: &mut BufReader<ReadHalf<'_>>) -> AnyResult<Vec<u8>> {
//     let mut raw_order: Vec<u8> = Vec::with_capacity(DEFAULT_CHUNK_SIZE as usize);
//     let mut buf: [u8; 8 * 1024] = [0; 8 * 1024];
//     while let Ok(bytes) = reader.read(&mut buf[..]).await {
//         if bytes == 0 {
//             break;
//         }
//         raw_order.extend_from_slice(&buf[..bytes]);
//     }
//     Ok(raw_order)
// }

async fn apply_command(
    ctx: Arc<Mutex<Context>>,
    writer: &mut BufWriter<WriteHalf<'_>>,
    command: Command,
) -> AnyResult<()> {
    match command {
        Command::GetChunk(chunk_id) => {
            eprintln!("applying get_chunk {}", chunk_id);
            let buf = {
                let mut guard = ctx.lock().expect("invalid mutex");
                let ctx = guard.deref_mut();

                let torrent = ctx.torrent.as_ref().unwrap();
                if chunk_id as usize >= torrent.metadata.completed_chunks.len() {
                    bail!(
                        "invalid chunk_id, id ({}) >= len({})",
                        chunk_id,
                        torrent.metadata.completed_chunks.len()
                    )
                }
                let chunk = ctx.chunks.as_mut().unwrap().read_chunk(chunk_id as u64)?;
                let buf: Vec<u8> = Command::SendChunk(chunk).into();
                buf
            };
            eprintln!("sending buf {:?}", &buf);
            writer.write_all(buf.as_slice()).await?;
        }
        Command::SendChunk(chunks) => eprintln!("sending buf {:?}", &chunks),
    }

    writer.flush().await?;
    Ok(())
}

async fn handle_connection(ctx: Arc<Mutex<Context>>, mut stream: TcpStream) -> AnyResult<()> {
    let peer_addr = stream.peer_addr()?;
    eprintln!("Connected to {}", peer_addr);
    let (reader, writer) = stream.split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut writer = tokio::io::BufWriter::new(writer);

    loop {
        let mut raw_order: Vec<u8> = Vec::with_capacity(DEFAULT_CHUNK_SIZE as usize);
        const BUF_SIZE: usize = 8 * 1024;
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
        while let Ok(bytes) = reader.read(&mut buf[..]).await {
            dbg!(&bytes, &buf[..bytes]);
            if bytes == 0 {
                break;
            }
            raw_order.extend_from_slice(&buf[..bytes]);
            if bytes < BUF_SIZE {
                break;
            }
        }

        let raw_order = raw_order.as_slice();
        let len = raw_order.len();
        if len == 0 {
            break;
        }
        dbg!(&raw_order);

        match raw_order.try_into() {
            Ok(command) => apply_command(Arc::clone(&ctx), &mut writer, command).await?,
            Err(err) => eprintln!("Unknown command received! {}", err),
        }
    }

    Ok(())
}

async fn ask_for_chunks(ctx: Arc<Mutex<Context>>, mut stream: TcpStream) -> AnyResult<()> {
    let peer_addr = stream.peer_addr()?;
    eprintln!("Connecting to {}", peer_addr);
    let (reader, writer) = stream.split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut writer = tokio::io::BufWriter::new(writer);

    let chunk_id = 0;

    let buf: Vec<u8> = Command::GetChunk(chunk_id).into();
    dbg!(&buf);
    writer.write_all(buf.as_slice()).await?;
    writer.flush().await?;

    let mut raw_chunk: Vec<u8> = Vec::with_capacity(DEFAULT_CHUNK_SIZE as usize);
    const BUF_SIZE: usize = 8 * 1024;
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
    while let Ok(bytes) = reader.read(&mut buf[..]).await {
        if bytes == 0 {
            break;
        }
        raw_chunk.extend_from_slice(&buf[..bytes]);
        if bytes < BUF_SIZE {
            break;
        }
    }

    let raw_chunk = raw_chunk.as_slice();
    let len = raw_chunk.len();
    if len == 0 {
        bail!("invalid buffer");
    }

    // // Update client chunks.
    // {
    //     let mut guard = ctx.lock().expect("invalid mutex");
    //     let ctx = guard.deref_mut();

    //     let torrent = ctx.torrent.as_mut().unwrap();
    //     torrent.metadata.completed_chunks[chunk_id as usize] = Some(0);

    //     let chunks = ctx.chunks.as_mut().unwrap();
    //     chunks.write_chunk(chunk_id as u64, raw_chunk)?;
    // }
    dbg!(&raw_chunk);
    match raw_chunk.try_into() {
        Ok(command) => apply_command(Arc::clone(&ctx), &mut writer, command).await?,
        Err(err) => eprintln!("Unknown command received! {}", err),
    }

    Ok(())
}
