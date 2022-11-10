use errors::{bail, AnyError};

#[derive(Debug)]
pub enum Command {
    Handshake(u32),               // 0x1, crc
    GetChunk(u32, u32),           // 0x2, crc, chunk_id
    SendChunk(u32, u32, Vec<u8>), // 0x3, crc, chunk_id, chunk
    FileNotFound,                 // 0x4
    ChunkNotFound,                // 0x5
    InvalidChunk,                 // 0x6
}
// TODO send host:port list ^

const ORDER_SIZE: usize = 1;
const HANDSHAKE_SIZE: usize = 4;
const GETCHUNK_SIZE: usize = 4 + 4;
const SENDCHUNK_SIZE: usize = 4 + 4;

const MIN_HANDSHAKE_SIZE: usize = ORDER_SIZE + HANDSHAKE_SIZE;
const MIN_GETCHUNK_SIZE: usize = ORDER_SIZE + GETCHUNK_SIZE;
const MIN_SENDCHUNK_SIZE: usize = ORDER_SIZE + SENDCHUNK_SIZE;

// Convert a raw buffer into a command.
impl TryFrom<&[u8]> for Command {
    type Error = AnyError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if let Some(raw_command) = value.get(0) {
            Ok(match raw_command {
                0x1 => {
                    if value.len() < MIN_HANDSHAKE_SIZE {
                        bail!(
                            "can't decode handshake size too low ({} < {})",
                            value.len(),
                            MIN_HANDSHAKE_SIZE
                        );
                    }
                    let slice: [u8; 4] = core::array::from_fn(|i| value[i + ORDER_SIZE]);
                    let crc = u8_array_to_u32(&slice);
                    Self::Handshake(crc)
                }
                0x2 => {
                    if value.len() < MIN_GETCHUNK_SIZE {
                        bail!(
                            "can't decode get_chunk size too low ({} < {})",
                            value.len(),
                            MIN_GETCHUNK_SIZE
                        );
                    }
                    let slice: [u8; 4] = core::array::from_fn(|i| value[i + ORDER_SIZE]);
                    let crc = u8_array_to_u32(&slice);
                    let slice: [u8; 4] = core::array::from_fn(|i| value[i + ORDER_SIZE + 4]);
                    let chunk_id = u8_array_to_u32(&slice);
                    Self::GetChunk(crc, chunk_id)
                }
                0x3 => {
                    if value.len() < MIN_SENDCHUNK_SIZE {
                        bail!(
                            "can't decode send_chunk size too low ({} < {})",
                            value.len(),
                            MIN_SENDCHUNK_SIZE
                        );
                    }

                    let slice: [u8; 4] = core::array::from_fn(|i| value[i + ORDER_SIZE]);
                    let crc = u8_array_to_u32(&slice);
                    let slice: [u8; 4] = core::array::from_fn(|i| value[i + ORDER_SIZE + 4]);
                    let chunk_id = u8_array_to_u32(&slice);
                    let chunk = value
                        .iter()
                        .skip(MIN_SENDCHUNK_SIZE)
                        .map(|item| *item)
                        .collect::<Vec<u8>>();
                    Self::SendChunk(crc, chunk_id, chunk)
                }
                0x4 => Self::FileNotFound,
                0x5 => Self::ChunkNotFound,
                0x6 => Self::InvalidChunk,
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
            Command::GetChunk(crc, chunk_id) => {
                let mut res = vec![0x2];
                res.extend(u32_to_u8_array(crc));
                res.extend(u32_to_u8_array(chunk_id));
                res
            }
            Command::SendChunk(crc, chunk_id, chunk_buf) => {
                let mut res = vec![0x3];
                res.extend(u32_to_u8_array(crc));
                res.extend(u32_to_u8_array(chunk_id));
                res.extend(chunk_buf);
                res
            }
            Command::FileNotFound => vec![0x4],
            Command::ChunkNotFound => vec![0x5],
            Command::InvalidChunk => vec![0x6],
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
