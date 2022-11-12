use crate::utils::{u32_list_to_u8_array_unfailable, u32_to_u8_array, u8_array_to_u32, u8_array_to_u32_list};
use errors::{bail, AnyError};
use std::fmt::Display;

// Protocol constants ----------------------------------------------------------

const ORDER_SIZE: usize = 1;
const HANDSHAKE_SIZE: usize = 4;
const GETCHUNK_SIZE: usize = 4 + 4;
const SENDCHUNK_SIZE: usize = 4 + 4;
const FILEINFO_SIZE: usize = 4 + 4 + 4 + 1; // 3*u32 + at least 1 char filename
const FINDNODE_REQUEST_SIZE: usize = 4 + 4;
const FINDNODE_RESPONSE_SIZE: usize = 1;

const MIN_HANDSHAKE_SIZE: usize = ORDER_SIZE + HANDSHAKE_SIZE;
const MIN_GETCHUNK_SIZE: usize = ORDER_SIZE + GETCHUNK_SIZE;
const MIN_SENDCHUNK_SIZE: usize = ORDER_SIZE + SENDCHUNK_SIZE;
const MIN_FILEINFO_SIZE: usize = ORDER_SIZE + FILEINFO_SIZE;
const MIN_FINDNODE_REQUEST_SIZE: usize = ORDER_SIZE + FINDNODE_REQUEST_SIZE;
const MIN_FINDNODE_RESPONSE_SIZE: usize = ORDER_SIZE + FINDNODE_RESPONSE_SIZE;

const HANDSHAKE: u8 = 0x1;
const GET_CHUNK: u8 = 0x2;
const SEND_CHUNK: u8 = 0x3;
const FILE_INFO: u8 = 0x4;
const PING_REQUEST: u8 = 0x5;
const PING_RESPONSE: u8 = 0x6;
const STORE_REQUEST: u8 = 0x7;
const STORE_RESPONSE: u8 = 0x8;
const FIND_NODE_REQUEST: u8 = 0x9;
const FIND_NODE_RESPONSE: u8 = 0x10;
const FIND_VALUE_REQUEST: u8 = 0x11;
const FIND_VALUE_RESPONSE: u8 = 0x12;
const FIND_GET_PEERS: u8 = 0x13;
const FIND_SEND_PEERS: u8 = 0x14;

const ERROR_OCCURED: u8 = 0x80;

// Commands --------------------------------------------------------------------

// API used to communicate between peers. Handles both messages and errors.
#[derive(Debug)]
pub enum Command {
    // Half the range for error code
    ErrorOccured(ErrorCode),

    // File protocol
    Handshake(u32 /*crc*/),
    GetChunk(u32 /*crc*/, u32 /*chunk_id*/),
    SendChunk(u32 /*crc*/, u32 /*chunk_id*/, Vec<u8> /*chunk*/),
    FileInfo(FileInfo),
    // DHT protocol
    // PingRequest
    // PingResponse
    // StoreRequest
    // StoreResponse
    FindNodeRequest(u32 /*sender*/, u32 /*target*/),
    FindNodeResponse(Vec<u32> /*peers_found*/),
    // FindValueRequest
    // FindValueResponse

    // GetPeers(), // 0x06 ?
    // SeendPeers(), // 0x07 ?
}
// TODO send host:port list ^

// Command Convert -------------------------------------------------------------

// Convert a raw buffer into a command.
impl TryFrom<&[u8]> for Command {
    type Error = AnyError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if let Some(raw_command) = value.get(0) {
            Ok(match *raw_command {
                // Messages
                HANDSHAKE => {
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
                GET_CHUNK => {
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
                SEND_CHUNK => {
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
                FILE_INFO => {
                    // FIXME request + response
                    let file_info = FileInfo::try_from(&value[1..])?;
                    Self::FileInfo(file_info)
                }
                FIND_NODE_REQUEST => {
                    if value.len() < MIN_FINDNODE_REQUEST_SIZE {
                        bail!(
                            "can't decode find_node_request, size too low ({} < {})",
                            value.len(),
                            FINDNODE_REQUEST_SIZE
                        );
                    }

                    let slice: [u8; 4] = core::array::from_fn(|i| value[i + ORDER_SIZE]);
                    let sender = u8_array_to_u32(&slice);
                    let slice: [u8; 4] = core::array::from_fn(|i| value[i + ORDER_SIZE + 4]);
                    let target = u8_array_to_u32(&slice);
                    Self::FindNodeRequest(sender, target)
                }
                FIND_NODE_RESPONSE => {
                    if value.len() < MIN_FINDNODE_RESPONSE_SIZE {
                        bail!(
                            "can't decode find_node_response, size too low ({} < {})",
                            value.len(),
                            FINDNODE_RESPONSE_SIZE
                        );
                    }

                    let peers_found = u8_array_to_u32_list(&value[1..])?;
                    Self::FindNodeResponse(peers_found)
                }

                // Errors
                error if error >= ERROR_OCCURED => Self::ErrorOccured((error - ERROR_OCCURED).into()),
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
                let mut res = vec![HANDSHAKE];
                res.extend(u32_to_u8_array(crc));
                res
            }
            Command::GetChunk(crc, chunk_id) => {
                let mut res = vec![GET_CHUNK];
                res.extend(u32_to_u8_array(crc));
                res.extend(u32_to_u8_array(chunk_id));
                res
            }
            Command::SendChunk(crc, chunk_id, chunk_buf) => {
                let mut res = vec![SEND_CHUNK];
                res.extend(u32_to_u8_array(crc));
                res.extend(u32_to_u8_array(chunk_id));
                res.extend(chunk_buf);
                res
            }
            Command::FileInfo(file_info) => {
                let raw_file_info: Vec<u8> = file_info.into();
                let mut res = vec![FILE_INFO];
                res.extend(raw_file_info);
                res
            }
            Command::FindNodeRequest(sender, target) => {
                let mut res = vec![FIND_NODE_REQUEST];
                res.extend(u32_to_u8_array(sender));
                res.extend(u32_to_u8_array(target));
                res
            }
            Command::FindNodeResponse(peers_found) => {
                let mut res = vec![FIND_NODE_RESPONSE];
                res.extend(u32_list_to_u8_array_unfailable(peers_found.as_slice()));
                res
            }

            // Errors
            Command::ErrorOccured(error) => vec![ERROR_OCCURED + error as u8],
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

#[repr(u8)]
#[derive(Debug)]
pub enum ErrorCode {
    Unknown = 0,
    FileNotFound = 1,
    ChunkNotFound = 2,
    InvalidChunk = 3,
}

impl From<u8> for ErrorCode {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::FileNotFound,
            2 => Self::ChunkNotFound,
            3 => Self::InvalidChunk,
            _ => Self::Unknown,
        }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::Unknown => write!(fmt, "unknow error occured"),
            ErrorCode::FileNotFound => write!(fmt, "file not found"),
            ErrorCode::ChunkNotFound => write!(fmt, "chunk not found"),
            ErrorCode::InvalidChunk => write!(fmt, "invalid chunk"),
        }
    }
}

#[cfg(test)]
#[path = "protocol_test.rs"]
mod protocol_test;
