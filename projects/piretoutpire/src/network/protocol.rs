use super::chunk_pieces::{ChunkPieces, NetChunk};
use errors::{bail, reexports::eyre::ContextCompat, AnyError};

pub trait FileSharable {
    type Addr;

    fn send_chunk(to: Self::Addr, chunk_id: u64, chunk: NetChunk);
    fn get_chunk(from: Self::Addr, chunk_id: u64) -> NetChunk;

    fn chunks_owned(chunks: ChunkPieces);
    fn chunks_wanted() -> ChunkPieces;
}

#[derive(Debug)]
pub enum Command {
    // Handshake({is_seeder: bool}), // 0x01
    GetChunk(u32),      // 0x02
    SendChunk(Vec<u8>), // 0x03
}

// Convert a raw buffer into a command.
impl TryFrom<&[u8]> for Command {
    type Error = AnyError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if let Some(raw_command) = value.get(0) {
            Ok(match raw_command {
                0x2 => Command::GetChunk(*value.get(1).context("invalid get chunk command")? as u32),
                0x3 => Command::SendChunk(value.iter().skip(1).map(|item| *item).collect::<Vec<u8>>()),
                _ => bail!("Unknown command {}", raw_command),
            })
        } else {
            bail!("empty order");
        }
    }
}

// Convert a command to a raw buffer.
impl From<Command> for Vec<u8> {
    fn from(value: Command) -> Self {
        match value {
            Command::GetChunk(chunk_id) => {
                let mut res = vec![0x2];
                let encoded = u32_to_u8_array(chunk_id);
                res.extend(encoded);
                res
            }
            Command::SendChunk(chunk_buf) => {
                let mut res = vec![0x3];
                res.extend(chunk_buf);
                res
            }
        }
    }
}

fn u32_to_u8_array(x: u32) -> [u8; 4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;

    [b1, b2, b3, b4]
}
