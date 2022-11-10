use super::{
    chunk_pieces::{ChunkPieces, NetChunk},
    protocol::FileSharable,
};
use std::net::SocketAddr;

pub struct FileSharing {}

impl FileSharing {
    pub fn new() -> Self {
        Self {}
    }
}

impl FileSharable for FileSharing {
    type Addr = SocketAddr;

    fn send_chunk(to: Self::Addr, chunk_id: u64, chunk: NetChunk) {
        todo!()
    }

    fn get_chunk(from: Self::Addr, chunk_id: u64) -> NetChunk {
        todo!()
    }

    fn chunks_owned(chunks: ChunkPieces) {
        todo!()
    }

    fn chunks_wanted() -> ChunkPieces {
        todo!()
    }
}

#[cfg(test)]
#[path = "file_sharing_test.rs"]
mod file_sharing_test;
