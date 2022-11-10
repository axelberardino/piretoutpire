use crate::network::{
    chunk_pieces::{ChunkPieces, NetChunk},
    protocol::FileSharable,
};

pub struct FileSharingMock {
    data: Vec<u8>,
}

impl FileSharingMock {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl FileSharable for FileSharingMock {
    type Addr = u8;

    fn send_chunk(to: Self::Addr, chunk_id: u64, chunk: NetChunk) {}

    fn get_chunk(from: Self::Addr, chunk_id: u64) -> NetChunk {
        todo!()
    }

    fn chunks_owned(chunks: ChunkPieces) {}

    fn chunks_wanted() -> ChunkPieces {
        todo!()
    }
}

#[test]
fn test_simple_share() {
    // let share = FileSharingMock::new();
}
