use super::{
    command_handler::{ask_for_chunk, get_file_info, listen_to_command},
    context::Context,
};
use crate::file::{file_chunk::FileChunk, torrent_file::TorrentFile};
use errors::{reexports::eyre::ContextCompat, AnyResult};
use std::{net::SocketAddr, ops::DerefMut, path::Path, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

pub struct Manager {
    addr: SocketAddr,
    ctx: Arc<Mutex<Context>>,
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
            let stream = Arc::new(Mutex::new(TcpStream::connect(client_addr).await?));
            get_file_info(local_ctx, stream, crc).await?;
        }

        // Get some info about what to download
        let nb_chunks = {
            let mut guard = ctx.lock().await;
            let ctx = guard.deref_mut();

            let (_, chunks) = ctx
                .available_torrents
                .get(&crc)
                .context("unable to find associated chunks")?;

            chunks.nb_chunks()
        };

        {
            let client_addr: SocketAddr = "127.0.0.1:4000".parse()?;
            let stream = Arc::new(Mutex::new(TcpStream::connect(client_addr).await?));
            for chunk_id in 0..nb_chunks {
                let local_ctx = Arc::clone(&ctx);
                let stream = Arc::clone(&stream);

                let handle =
                    tokio::spawn(async move { ask_for_chunk(local_ctx, stream, crc, chunk_id).await });
                handle.await??;
            }
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
            let mut ctx = self.ctx.lock().await;
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
