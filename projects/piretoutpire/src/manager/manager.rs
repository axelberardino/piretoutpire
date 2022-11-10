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
    net::SocketAddr,
    ops::DerefMut,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{tcp::WriteHalf, TcpListener, TcpStream},
};

pub struct Manager {
    addr: SocketAddr,
    ctx: Arc<Mutex<Context>>,
}

pub struct Context {
    // Realtime list of peers
    pub peers: HashMap<u32, SocketAddr>,

    // TODO  Associate that to a hashmap crc + struct
    pub available_torrents: HashMap<u32 /*crc*/, (TorrentFile<String> /*metadata*/, FileChunk /*file*/)>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            available_torrents: HashMap::new(),
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
        // TODO ask for peers.

        let chunk_id = 0;
        let crc = 3613099103;

        let handle = tokio::spawn(async move { ask_for_chunks(ctx, stream, crc, chunk_id).await });
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
            ctx.available_torrents
                .insert(torrent.metadata.file_crc, (torrent, chunks));
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

macro_rules! read_all {
    ($reader:ident) => {{
        let mut res_buf: Vec<u8> = Vec::with_capacity(DEFAULT_CHUNK_SIZE as usize);
        const BUF_SIZE: usize = 8 * 1024;
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
        while let Ok(bytes) = $reader.read(&mut buf[..]).await {
            dbg!(&bytes, &buf[..bytes]);
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

async fn apply_command(
    ctx: Arc<Mutex<Context>>,
    writer: &mut BufWriter<WriteHalf<'_>>,
    command: Command,
) -> AnyResult<()> {
    match command {
        Command::Handshake(crc) => {
            eprintln!("handshake, ask for crc {}", crc);
        }
        Command::GetChunk(crc, chunk_id) => {
            eprintln!("applying get_chunk {}", chunk_id);
            let response: Vec<u8> = {
                let mut guard = ctx.lock().expect("invalid mutex");
                let ctx = guard.deref_mut();

                match ctx.available_torrents.get_mut(&crc) {
                    Some((torrent, chunks)) => {
                        if chunk_id as usize >= torrent.metadata.completed_chunks.len() {
                            (Command::InvalidChunk).into()
                        } else {
                            match chunks.read_chunk(chunk_id as u64) {
                                Ok(chunk) => Command::SendChunk(crc, chunk_id, chunk).into(),
                                Err(_) => Command::ChunkNotFound.into(),
                            }
                        }
                    }
                    None => Command::FileNotFound.into(),
                }
            };
            eprintln!("sending buf {:?}", &response);
            writer.write_all(response.as_slice()).await?;
        }
        Command::SendChunk(crc, chunk_id, raw_chunk) => {
            eprintln!("received buf {:?}", &raw_chunk);
            let mut guard = ctx.lock().expect("invalid mutex");
            let ctx = guard.deref_mut();

            match ctx.available_torrents.get_mut(&crc) {
                Some((torrent, chunks)) => {
                    if chunk_id as usize >= torrent.metadata.completed_chunks.len() {
                        eprintln!("Invalid chunk_id {} for {}", chunk_id, crc);
                    } else {
                        // Update metadata and write local chunk.
                        torrent.metadata.completed_chunks[chunk_id as usize] = Some(0);
                        chunks.write_chunk(chunk_id as u64, raw_chunk.as_slice())?;
                    }
                }
                None => eprintln!("Got chunk for an unknown file"),
            }
        }
        Command::FileNotFound => eprintln!("peer don't have this file"),
        Command::ChunkNotFound => eprintln!("peer don't have this chunk"),
        Command::InvalidChunk => eprintln!("peer said chunk was invalid"),
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
        let raw_order = read_all!(reader);
        let len = raw_order.len();
        if len == 0 {
            break;
        }
        dbg!(&raw_order);

        match raw_order.as_slice().try_into() {
            Ok(command) => apply_command(Arc::clone(&ctx), &mut writer, command).await?,
            Err(err) => eprintln!("Unknown command received! {}", err),
        }
    }

    Ok(())
}

async fn ask_for_chunks(
    ctx: Arc<Mutex<Context>>,
    mut stream: TcpStream,
    crc: u32,
    chunk_id: u32,
) -> AnyResult<()> {
    let peer_addr = stream.peer_addr()?;
    eprintln!("Connecting to {}", peer_addr);
    let (reader, writer) = stream.split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut writer = tokio::io::BufWriter::new(writer);

    let buf: Vec<u8> = Command::GetChunk(crc, chunk_id).into();
    writer.write_all(buf.as_slice()).await?;
    writer.flush().await?;

    let raw_chunk = read_all!(reader);
    let len = raw_chunk.len();
    if len == 0 {
        bail!("invalid buffer");
    }

    match raw_chunk.as_slice().try_into() {
        Ok(command) => apply_command(Arc::clone(&ctx), &mut writer, command).await?,
        Err(err) => eprintln!("Unknown command received! {}", err),
    }

    Ok(())
}
