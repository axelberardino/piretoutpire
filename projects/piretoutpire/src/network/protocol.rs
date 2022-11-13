use crate::utils::{
    addr_to_u8_array, string_to_u8_array, u32_to_u8_array, u8_array_to_addr, u8_array_to_string,
    u8_array_to_u32,
};
use errors::{bail, AnyError};
use std::{fmt::Display, net::SocketAddr};

// Protocol constants ----------------------------------------------------------

const ORDER_SIZE: usize = 1; // u8
const FILEINFO_REQUEST_SIZE: usize = 4; // u32
const CHUNK_REQUEST_SIZE: usize = 4 + 4; // 2*u32
const CHUNK_RESPONSE_SIZE: usize = 4 + 4; // 2*u32 + chunk(0+)
const FILEINFO_RESPONSE_SIZE: usize = 4 + 4 + 4 + 4; // 3*u32 + str(4+)
const FINDNODE_REQUEST_SIZE: usize = 4 + 4;
const FINDNODE_RESPONSE_SIZE: usize = 1; // list(1+)
const PEER_SIZE: usize = 4 + 4; // id(4) + str(4+)

const MIN_FILEINFO_REQUEST_SIZE: usize = ORDER_SIZE + FILEINFO_REQUEST_SIZE;
const MIN_CHUNK_REQUEST_SIZE: usize = ORDER_SIZE + CHUNK_REQUEST_SIZE;
const MIN_CHUNK_RESPONSE_SIZE: usize = ORDER_SIZE + CHUNK_RESPONSE_SIZE;
const MIN_FILEINFO_RESPONSE_SIZE: usize = ORDER_SIZE + FILEINFO_RESPONSE_SIZE;
const MIN_FINDNODE_REQUEST_SIZE: usize = ORDER_SIZE + FINDNODE_REQUEST_SIZE;
const MIN_FINDNODE_RESPONSE_SIZE: usize = ORDER_SIZE + FINDNODE_RESPONSE_SIZE;
const MIN_PEER_SIZE: usize = ORDER_SIZE + PEER_SIZE;

const FILEINFO_REQUEST: u8 = 0x1;
const GET_CHUNK: u8 = 0x2;
const SEND_CHUNK: u8 = 0x3;
const FILE_INFO_RESPONSE: u8 = 0x4;
const _PING_REQUEST: u8 = 0x5;
const _PING_RESPONSE: u8 = 0x6;
const _STORE_REQUEST: u8 = 0x7;
const _STORE_RESPONSE: u8 = 0x8;
const FIND_NODE_REQUEST: u8 = 0x9;
const FIND_NODE_RESPONSE: u8 = 0x10;
const _FIND_VALUE_REQUEST: u8 = 0x11;
const _FIND_VALUE_RESPONSE: u8 = 0x12;
const _FIND_GET_PEERS: u8 = 0x13;
const _FIND_SEND_PEERS: u8 = 0x14;

const ERROR_OCCURED: u8 = 0x80;

// Commands --------------------------------------------------------------------

// API used to communicate between peers. Handles both messages and errors.
#[derive(Debug)]
pub enum Command {
    // Half the range for error code
    ErrorOccured(ErrorCode),

    // File protocol
    FileInfoRequest(u32 /*crc*/),
    FileInfoResponse(FileInfo),
    ChunkRequest(u32 /*crc*/, u32 /*chunk_id*/),
    ChunkResponse(u32 /*crc*/, u32 /*chunk_id*/, Vec<u8> /*chunk*/),

    // DHT protocol
    // PingRequest (sender_uid, if somone ping you, you're still there!)
    // PingResponse (Or here, put the node_id inside).
    // StoreRequest
    // StoreResponse
    FindNodeRequest(u32 /*sender*/, u32 /*target*/),
    FindNodeResponse(Vec<Peer> /*peers_found*/),
    // FindValueRequest
    // FindValueResponse

    // GetPeers(), // 0x06 ?
    // SeendPeers(), // 0x07 ?
}

// Convert a raw buffer into a command.
impl TryFrom<&[u8]> for Command {
    type Error = AnyError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if let Some(raw_command) = value.get(0) {
            Ok(match *raw_command {
                // Messages
                FILEINFO_REQUEST => {
                    if value.len() < MIN_FILEINFO_REQUEST_SIZE {
                        bail!(
                            "can't decode handshake, size too low ({} < {})",
                            value.len(),
                            MIN_FILEINFO_REQUEST_SIZE
                        );
                    }
                    let slice: [u8; 4] = core::array::from_fn(|idx| value[idx + ORDER_SIZE]);
                    let crc = u8_array_to_u32(&slice);
                    Self::FileInfoRequest(crc)
                }
                GET_CHUNK => {
                    if value.len() < MIN_CHUNK_REQUEST_SIZE {
                        bail!(
                            "can't decode get_chunk, size too low ({} < {})",
                            value.len(),
                            MIN_CHUNK_REQUEST_SIZE
                        );
                    }
                    let slice: [u8; 4] = core::array::from_fn(|idx| value[idx + ORDER_SIZE]);
                    let crc = u8_array_to_u32(&slice);
                    let slice: [u8; 4] = core::array::from_fn(|idx| value[idx + ORDER_SIZE + 4]);
                    let chunk_id = u8_array_to_u32(&slice);
                    Self::ChunkRequest(crc, chunk_id)
                }
                SEND_CHUNK => {
                    if value.len() < MIN_CHUNK_RESPONSE_SIZE {
                        bail!(
                            "can't decode send_chunk, size too low ({} < {})",
                            value.len(),
                            MIN_CHUNK_RESPONSE_SIZE
                        );
                    }

                    let slice: [u8; 4] = core::array::from_fn(|idx| value[idx + ORDER_SIZE]);
                    let crc = u8_array_to_u32(&slice);
                    let slice: [u8; 4] = core::array::from_fn(|idx| value[idx + ORDER_SIZE + 4]);
                    let chunk_id = u8_array_to_u32(&slice);
                    let chunk = value
                        .iter()
                        .skip(MIN_CHUNK_RESPONSE_SIZE)
                        .map(|item| *item)
                        .collect::<Vec<u8>>();
                    Self::ChunkResponse(crc, chunk_id, chunk)
                }
                FILE_INFO_RESPONSE => {
                    if value.len() < MIN_FILEINFO_RESPONSE_SIZE {
                        bail!(
                            "can't decode file_info, size too low ({} < {})",
                            value.len(),
                            MIN_FILEINFO_RESPONSE_SIZE
                        );
                    }

                    let file_info = FileInfo::try_from(&value[1..])?;
                    Self::FileInfoResponse(file_info)
                }
                FIND_NODE_REQUEST => {
                    if value.len() < MIN_FINDNODE_REQUEST_SIZE {
                        bail!(
                            "can't decode find_node_request, size too low ({} < {})",
                            value.len(),
                            MIN_FINDNODE_REQUEST_SIZE
                        );
                    }

                    let slice: [u8; 4] = core::array::from_fn(|idx| value[idx + ORDER_SIZE]);
                    let sender = u8_array_to_u32(&slice);
                    let slice: [u8; 4] = core::array::from_fn(|idx| value[idx + ORDER_SIZE + 4]);
                    let target = u8_array_to_u32(&slice);
                    Self::FindNodeRequest(sender, target)
                }
                FIND_NODE_RESPONSE => {
                    if value.len() < MIN_FINDNODE_RESPONSE_SIZE {
                        bail!(
                            "can't decode find_node_response, size too low ({} < {})",
                            value.len(),
                            MIN_FINDNODE_RESPONSE_SIZE
                        );
                    }

                    let slice: [u8; 4] = core::array::from_fn(|idx| value[idx + ORDER_SIZE]);
                    let list_size = u8_array_to_u32(&slice) as usize;
                    let res = (0..list_size).try_fold(
                        (Vec::<Peer>::new(), ORDER_SIZE + 4),
                        |(mut acc, shift), _| {
                            let raw = &value[shift..];
                            // size of addr is after id (in pos 4).
                            let slice: [u8; 4] = core::array::from_fn(|idx| value[shift + idx + 4]);
                            let addr_size = u8_array_to_u32(&slice) as usize;

                            let peer = Peer::try_from(raw)?;
                            acc.push(peer);
                            Ok::<(Vec<Peer>, usize), AnyError>((acc, shift + addr_size + 4))
                        },
                    )?;
                    let (peers_list, _) = res;
                    Self::FindNodeResponse(peers_list)
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
            Command::FileInfoRequest(crc) => {
                let mut res = vec![FILEINFO_REQUEST];
                res.extend(u32_to_u8_array(crc));
                res
            }
            Command::ChunkRequest(crc, chunk_id) => {
                let mut res = vec![GET_CHUNK];
                res.extend(u32_to_u8_array(crc));
                res.extend(u32_to_u8_array(chunk_id));
                res
            }
            Command::ChunkResponse(crc, chunk_id, chunk_buf) => {
                let mut res = vec![SEND_CHUNK];
                res.extend(u32_to_u8_array(crc));
                res.extend(u32_to_u8_array(chunk_id));
                res.extend(chunk_buf);
                res
            }
            Command::FileInfoResponse(file_info) => {
                let raw_file_info: Vec<u8> = file_info.into();
                let mut res = vec![FILE_INFO_RESPONSE];
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
                res.extend(u32_to_u8_array(peers_found.len() as u32));
                peers_found.into_iter().fold(res, |mut acc, peer| {
                    let raw: Vec<u8> = peer.into();
                    acc.extend(raw);
                    acc
                })
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

// Convert a raw buffer into a command.
impl TryFrom<&[u8]> for FileInfo {
    type Error = AnyError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < MIN_FILEINFO_RESPONSE_SIZE {
            bail!(
                "can't decode file_info, size too low ({} < {})",
                value.len(),
                MIN_FILEINFO_RESPONSE_SIZE
            );
        }

        let slice: [u8; 4] = core::array::from_fn(|i| value[i]);
        let file_size = u8_array_to_u32(&slice);
        let slice: [u8; 4] = core::array::from_fn(|i| value[i + 4]);
        let chunk_size = u8_array_to_u32(&slice);
        let slice: [u8; 4] = core::array::from_fn(|i| value[i + 4 + 4]);
        let file_crc = u8_array_to_u32(&slice);

        let raw_str = value.iter().skip(4 + 4 + 4).map(|ch| *ch).collect::<Vec<u8>>();
        let original_filename = u8_array_to_string(raw_str.as_slice())?;

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
        let mut res = Vec::with_capacity(FILEINFO_RESPONSE_SIZE);
        res.extend(u32_to_u8_array(value.file_size));
        res.extend(u32_to_u8_array(value.chunk_size));
        res.extend(u32_to_u8_array(value.file_crc));
        res.extend(string_to_u8_array(value.original_filename));
        res
    }
}

// Peer ------------------------------------------------------------------------

// Struct used to hold a peer.
#[derive(Debug, Clone)]
pub struct Peer {
    pub id: u32,
    pub addr: SocketAddr,
}

// Convert a raw buffer into a command.
impl TryFrom<&[u8]> for Peer {
    type Error = AnyError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < MIN_PEER_SIZE {
            bail!(
                "can't decode peer, size too low ({} < {})",
                value.len(),
                MIN_PEER_SIZE
            );
        }

        let slice: [u8; 4] = core::array::from_fn(|i| value[i]);
        let id = u8_array_to_u32(&slice);

        let raw_str = value.iter().skip(4).map(|ch| *ch).collect::<Vec<u8>>();
        let addr = u8_array_to_addr(raw_str.as_slice())?;

        Ok(Self { id, addr })
    }
}

impl From<Peer> for Vec<u8> {
    fn from(value: Peer) -> Self {
        let mut res = Vec::with_capacity(PEER_SIZE);
        res.extend(u32_to_u8_array(value.id));
        res.extend(addr_to_u8_array(value.addr));
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
