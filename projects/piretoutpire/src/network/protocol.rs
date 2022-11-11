use crate::utils::{u32_to_u8_array, u8_array_to_u32};
use errors::{bail, AnyError};

// Protocol constants ----------------------------------------------------------

const ORDER_SIZE: usize = 1;
const HANDSHAKE_SIZE: usize = 4;
const GETCHUNK_SIZE: usize = 4 + 4;
const SENDCHUNK_SIZE: usize = 4 + 4;
const FILEINFO_SIZE: usize = 4 + 4 + 4 + 1; // 3*u32 + at least 1 char filename

const MIN_HANDSHAKE_SIZE: usize = ORDER_SIZE + HANDSHAKE_SIZE;
const MIN_GETCHUNK_SIZE: usize = ORDER_SIZE + GETCHUNK_SIZE;
const MIN_SENDCHUNK_SIZE: usize = ORDER_SIZE + SENDCHUNK_SIZE;
const MIN_FILEINFO_SIZE: usize = ORDER_SIZE + FILEINFO_SIZE;

// Commands --------------------------------------------------------------------

// API used to communicate between peers. Handles both messages and errors.
#[derive(Debug)]
pub enum Command {
    // Half the range for error code
    ErrorOccured(ErrorCode), // 0x80 + ErrorCode
    // Other half for messages
    Handshake(u32),               // 0x01, crc
    GetChunk(u32, u32),           // 0x02, crc, chunk_id
    SendChunk(u32, u32, Vec<u8>), // 0x03, crc, chunk_id, chunk
    FileInfo(FileInfo),           // 0x04, FileInfo
}
// TODO send host:port list ^

// Command Convert -------------------------------------------------------------

// Convert a raw buffer into a command.
impl TryFrom<&[u8]> for Command {
    type Error = AnyError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if let Some(raw_command) = value.get(0) {
            Ok(match raw_command {
                // Messages
                0x1 => {
                    if value.len() < MIN_HANDSHAKE_SIZE {
                        bail!(
                            "can't decode handshake, size too low ({} < {})",
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
                            "can't decode get_chunk, size too low ({} < {})",
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
                            "can't decode send_chunk, size too low ({} < {})",
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
                0x4 => {
                    let file_info = FileInfo::try_from(&value[1..])?;
                    Self::FileInfo(file_info)
                }

                // Errors
                0x81 => Self::ErrorOccured(ErrorCode::FileNotFound),
                0x82 => Self::ErrorOccured(ErrorCode::ChunkNotFound),
                0x83 => Self::ErrorOccured(ErrorCode::InvalidChunk),
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
            // Messages
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
            Command::FileInfo(file_info) => {
                let raw_file_info: Vec<u8> = file_info.into();
                let mut res = vec![0x4];
                res.extend(raw_file_info);
                res
            }

            // Errors
            Command::ErrorOccured(ErrorCode::FileNotFound) => vec![0x81],
            Command::ErrorOccured(ErrorCode::ChunkNotFound) => vec![0x82],
            Command::ErrorOccured(ErrorCode::InvalidChunk) => vec![0x83],
        }
    }
}

// FileInfo --------------------------------------------------------------------

// Contains all metadata information about a given file.
#[derive(Debug)]
pub struct FileInfo {
    pub file_size: u32,
    pub chunk_size: u32,
    pub file_crc: u32,
    pub original_filename: String,
}

// FileInfo Convert ------------------------------------------------------------

// Convert a raw buffer into a command.
impl TryFrom<&[u8]> for FileInfo {
    type Error = AnyError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < MIN_FILEINFO_SIZE {
            bail!(
                "can't decode file_info, size too low ({} < {})",
                value.len(),
                MIN_FILEINFO_SIZE
            );
        }

        let slice: [u8; 4] = core::array::from_fn(|i| value[i]);
        let file_size = u8_array_to_u32(&slice);
        let slice: [u8; 4] = core::array::from_fn(|i| value[i + 4]);
        let chunk_size = u8_array_to_u32(&slice);
        let slice: [u8; 4] = core::array::from_fn(|i| value[i + 4 + 4]);
        let file_crc = u8_array_to_u32(&slice);

        let raw_str = value.iter().skip(4 + 4 + 4).map(|ch| *ch).collect::<Vec<u8>>();
        let original_filename = String::from_utf8(raw_str)?;

        Ok(Self {
            file_size,
            chunk_size,
            file_crc,
            original_filename,
        })
    }
}

impl From<FileInfo> for Vec<u8> {
    fn from(value: FileInfo) -> Self {
        let mut res = Vec::with_capacity(FILEINFO_SIZE);
        res.extend(u32_to_u8_array(value.file_size));
        res.extend(u32_to_u8_array(value.chunk_size));
        res.extend(u32_to_u8_array(value.file_crc));
        res.extend(value.original_filename.as_bytes());
        res
    }
}

// Error codes -----------------------------------------------------------------

#[derive(Debug)]
pub enum ErrorCode {
    FileNotFound = 1,
    ChunkNotFound = 2,
    InvalidChunk = 3,
}

#[cfg(test)]
#[path = "protocol_test.rs"]
mod protocol_test;
