use super::chunk_pieces::{ChunkPieces, NetChunk};
use errors::{bail, AnyError};

pub trait FileSharable {
    type Addr;

    fn send_chunk(to: Self::Addr, chunk_id: u64, chunk: NetChunk);
    fn get_chunk(from: Self::Addr, chunk_id: u64) -> NetChunk;

    fn chunks_owned(chunks: ChunkPieces);
    fn chunks_wanted() -> ChunkPieces;
}

#[derive(Debug)]
pub enum Command {
    Handshake(u32),          // 0x01, crc
    GetChunk(u32),           // 0x02, chunk_id
    SendChunk(u32, Vec<u8>), // 0x03, chunk_id, chunk
}
// TODO send host:port list

// Convert a raw buffer into a command.
impl TryFrom<&[u8]> for Command {
    type Error = AnyError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if let Some(raw_command) = value.get(0) {
            Ok(match raw_command {
                0x1 => {
                    if value.len() < 5 {
                        bail!("can't decode crc");
                    }
                    let slice: [u8; 4] = core::array::from_fn(|i| value[i + 1]);
                    let crc = u8_array_to_u32(&slice);
                    Command::Handshake(crc)
                }
                0x2 => {
                    if value.len() < 5 {
                        bail!("can't decode chunk_id");
                    }
                    let slice: [u8; 4] = core::array::from_fn(|i| value[i + 1]);
                    let chunk_id = u8_array_to_u32(&slice);
                    Command::GetChunk(chunk_id)
                }
                0x3 => {
                    if value.len() < 5 {
                        bail!("can't decode chunk_id");
                    }
                    let slice: [u8; 4] = core::array::from_fn(|i| value[i + 1]);
                    let chunk_id = u8_array_to_u32(&slice);
                    let chunk = value.iter().skip(5).map(|item| *item).collect::<Vec<u8>>();
                    Command::SendChunk(chunk_id, chunk)
                }
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
            Command::Handshake(crc) => {
                let mut res = vec![0x1];
                res.extend(u32_to_u8_array(crc));
                res
            }
            Command::GetChunk(chunk_id) => {
                let mut res = vec![0x2];
                res.extend(u32_to_u8_array(chunk_id));
                res
            }
            Command::SendChunk(chunk_id, chunk_buf) => {
                let mut res = vec![0x3];
                res.extend(u32_to_u8_array(chunk_id));
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

fn u8_array_to_u32(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 24)
        + ((array[1] as u32) << 16)
        + ((array[2] as u32) << 8)
        + ((array[3] as u32) << 0)
}

#[cfg(test)]
#[path = "protocol_test.rs"]
mod protocol_test;
