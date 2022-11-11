use crate::{
    file::{
        file_chunk::{FileChunk, DEFAULT_CHUNK_SIZE},
        torrent_file::TorrentFile,
    },
    network::protocol::{Command, ErrorCode, FileInfo},
};
use errors::{bail, reexports::eyre::ContextCompat, AnyResult};
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
    pub peers: HashMap<u32, SocketAddr>, // TODO: DHT

    // List of all available torrents currently owned, or currently downloading.
    pub available_torrents: HashMap<u32 /*crc*/, (TorrentFile<String> /*metadata*/, FileChunk /*file*/)>,

    // Where all torrents and their metadata are.
    pub working_directory: String,
}

impl Context {
    pub fn new(working_directory: String) -> Self {
        Self {
            peers: HashMap::new(),
            available_torrents: HashMap::new(),
            working_directory,
        }
    }
}

impl Manager {
    // Expect an address like: "127.0.0.1:8080".parse()
    pub fn new(addr: SocketAddr, working_directory: String) -> Self {
        Self {
            addr,
            ctx: Arc::new(Mutex::new(Context::new(working_directory))),
        }
    }

    // Start downloading a file, or resume downloading
    pub async fn download_file(&mut self, crc: u32) -> AnyResult<()> {
        let ctx = Arc::clone(&self.ctx);
        // TODO ask for peers.

        // Get file info
        {
            let client_addr: SocketAddr = "127.0.0.1:4000".parse()?;
            let local_ctx = Arc::clone(&ctx);
            let stream = TcpStream::connect(client_addr).await?;
            get_file_info(local_ctx, stream, crc).await?;
        }

        // Get some info about what to download
        let nb_chunks = {
            let mut guard = ctx.lock().expect("invalid mutex");
            let ctx = guard.deref_mut();

            let (_, chunks) = ctx
                .available_torrents
                .get(&crc)
                .context("unable to find associated chunks")?;

            chunks.nb_chunks()
        };

        for chunk_id in 0..nb_chunks {
            let local_ctx = Arc::clone(&ctx);
            let handle = tokio::spawn(async move {
                let client_addr: SocketAddr = "127.0.0.1:4000".parse()?;
                let stream = TcpStream::connect(client_addr).await?;
                ask_for_chunk(local_ctx, stream, crc, chunk_id).await
            });
            handle.await??;
        }

        // self.start_stream().await?;
        Ok(())
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
            tokio::spawn(async move { listen_to_command(ctx, stream).await });
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

// Interpret a command and act accordingly. This is where request/response are
// handled.
async fn apply_command(
    ctx: Arc<Mutex<Context>>,
    writer: &mut BufWriter<WriteHalf<'_>>,
    command: Command,
) -> AnyResult<()> {
    match command {
        // Message handling
        Command::Handshake(crc) => {
            eprintln!("handshake, ask for crc {}", crc);
            let response: Vec<u8> = {
                let mut guard = ctx.lock().expect("invalid mutex");
                let ctx = guard.deref_mut();

                match ctx.available_torrents.get(&crc) {
                    Some((torrent, _)) => {
                        let file_info = FileInfo {
                            file_size: torrent.metadata.file_size,
                            chunk_size: torrent.metadata.chunk_size,
                            file_crc: torrent.metadata.file_crc,
                            original_filename: torrent.metadata.original_filename.clone(),
                        };
                        Command::FileInfo(file_info).into()
                    }
                    None => Command::ErrorOccured(ErrorCode::FileNotFound).into(),
                }
            };
            eprintln!("sending buf {:?}", &response);
            writer.write_all(response.as_slice()).await?;
        }
        Command::GetChunk(crc, chunk_id) => {
            eprintln!("applying get_chunk {}", chunk_id);
            let response: Vec<u8> = {
                let mut guard = ctx.lock().expect("invalid mutex");
                let ctx = guard.deref_mut();

                match ctx.available_torrents.get_mut(&crc) {
                    Some((torrent, chunks)) => {
                        if chunk_id as usize >= torrent.metadata.completed_chunks.len() {
                            Command::ErrorOccured(ErrorCode::InvalidChunk).into()
                        } else {
                            match chunks.read_chunk(chunk_id) {
                                Ok(chunk) => Command::SendChunk(crc, chunk_id, chunk).into(),
                                Err(_) => Command::ErrorOccured(ErrorCode::ChunkNotFound).into(),
                            }
                        }
                    }
                    None => Command::ErrorOccured(ErrorCode::FileNotFound).into(),
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
                        chunks.write_chunk(chunk_id, raw_chunk.as_slice())?;
                    }
                }
                None => eprintln!("Got chunk for an unknown file"),
            }
        }
        Command::FileInfo(file_info) => {
            eprintln!("received file_info {:?}", &file_info);
            let mut guard = ctx.lock().expect("invalid mutex");
            let ctx = guard.deref_mut();

            match ctx.available_torrents.entry(file_info.file_crc) {
                std::collections::hash_map::Entry::Occupied(entry) => {
                    eprintln!("already got the asked torrent {}", entry.key())
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

        // Error handling
        Command::ErrorOccured(ErrorCode::FileNotFound) => eprintln!("peer don't have this file"),
        Command::ErrorOccured(ErrorCode::ChunkNotFound) => eprintln!("peer don't have this chunk"),
        Command::ErrorOccured(ErrorCode::InvalidChunk) => eprintln!("peer said chunk was invalid"),
    }

    writer.flush().await?;
    Ok(())
}

// Start to listen to command. One instance will be spawn for each peer.
async fn listen_to_command(ctx: Arc<Mutex<Context>>, mut stream: TcpStream) -> AnyResult<()> {
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

// Get file info from its ID (crc).
// Start by sending a Handshake request, and received either an error code
// or a FileInfo response.
async fn get_file_info(ctx: Arc<Mutex<Context>>, mut stream: TcpStream, crc: u32) -> AnyResult<()> {
    let peer_addr = stream.peer_addr()?;
    eprintln!("Connecting to {}", peer_addr);
    let (reader, writer) = stream.split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut writer = tokio::io::BufWriter::new(writer);

    let buf: Vec<u8> = Command::Handshake(crc).into();
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

// Ask for a given file chunk.
// Start by sending a GetChunk request, and received either an error code
// or the chunk as a raw buffer.
async fn ask_for_chunk(
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
