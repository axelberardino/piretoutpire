use crate::{
    dht::peer_node::PeerNode,
    utils::{
        addr_to_u8_array, string_to_u8_array, u32_to_u8_array, u8_array_to_addr, u8_array_to_string,
        u8_array_to_u32,
    },
};
use errors::{bail, AnyError};
use std::{fmt::Display, net::SocketAddr};

// Protocol constants ----------------------------------------------------------

const ORDER_SIZE: usize = 1; // u8
const PEER_SIZE: usize = 4 + 4; // id(4) + str(4+)
const FILEINFO_REQUEST_SIZE: usize = 4; // u32
const CHUNK_REQUEST_SIZE: usize = 4 + 4; // 2*u32
const CHUNK_RESPONSE_SIZE: usize = 4 + 4; // 2*u32 + chunk(0+)
const FILEINFO_RESPONSE_SIZE: usize = 4 + 4 + 4 + 4; // 3*u32 + str(4+)
const FINDNODE_REQUEST_SIZE: usize = 4 + 4;
const FINDNODE_RESPONSE_SIZE: usize = 1; // list(1+)
const PING_REQUEST_SIZE: usize = 4; // u32
const PING_RESPONSE_SIZE: usize = 4; // u32
const STORE_REQUEST_SIZE: usize = 4 + 1; // u32 + str(0+)
const STORE_RESPONSE_SIZE: usize = 0; // just an acknowledge
const FIND_VALUE_REQUEST_SIZE: usize = 4; // u32
const FIND_VALUE_RESPONSE_SIZE: usize = 1; // str(0+)
const MESSAGE_REQUEST_SIZE: usize = 1; // u32 + str(0+)
const MESSAGE_RESPONSE_SIZE: usize = 0; // just an acknowledge

const MIN_PEER_SIZE: usize = ORDER_SIZE + PEER_SIZE;
const MIN_FILEINFO_REQUEST_SIZE: usize = ORDER_SIZE + FILEINFO_REQUEST_SIZE;
const MIN_CHUNK_REQUEST_SIZE: usize = ORDER_SIZE + CHUNK_REQUEST_SIZE;
const MIN_CHUNK_RESPONSE_SIZE: usize = ORDER_SIZE + CHUNK_RESPONSE_SIZE;
const MIN_FILEINFO_RESPONSE_SIZE: usize = ORDER_SIZE + FILEINFO_RESPONSE_SIZE;
const MIN_FINDNODE_REQUEST_SIZE: usize = ORDER_SIZE + FINDNODE_REQUEST_SIZE;
const MIN_FINDNODE_RESPONSE_SIZE: usize = ORDER_SIZE + FINDNODE_RESPONSE_SIZE;
const MIN_PING_REQUEST_SIZE: usize = ORDER_SIZE + PING_REQUEST_SIZE;
const MIN_PING_RESPONSE_SIZE: usize = ORDER_SIZE + PING_RESPONSE_SIZE;
const MIN_STORE_REQUEST_SIZE: usize = ORDER_SIZE + STORE_REQUEST_SIZE;
const MIN_STORE_RESPONSE_SIZE: usize = ORDER_SIZE + STORE_RESPONSE_SIZE;
const MIN_FIND_VALUE_REQUEST_SIZE: usize = ORDER_SIZE + FIND_VALUE_REQUEST_SIZE;
const MIN_FIND_VALUE_RESPONSE_SIZE: usize = ORDER_SIZE + FIND_VALUE_RESPONSE_SIZE;
const MIN_MESSAGE_REQUEST_SIZE: usize = ORDER_SIZE + MESSAGE_REQUEST_SIZE;
const MIN_MESSAGE_RESPONSE_SIZE: usize = ORDER_SIZE + MESSAGE_RESPONSE_SIZE;

// File protocol.
const FILEINFO_REQUEST: u8 = 0x1;
const FILE_INFO_RESPONSE: u8 = 0x2;
const CHUNK_REQUEST: u8 = 0x3;
const CHUNK_RESPONSE: u8 = 0x4;
// DHT protocol.
const PING_REQUEST: u8 = 0x5;
const PING_RESPONSE: u8 = 0x6;
const STORE_REQUEST: u8 = 0x7;
const STORE_RESPONSE: u8 = 0x8;
const FIND_NODE_REQUEST: u8 = 0x9;
const FIND_NODE_RESPONSE: u8 = 0xA;
const FIND_VALUE_REQUEST: u8 = 0xB;
const FIND_VALUE_RESPONSE: u8 = 0xC;
// Message protocol.
const MESSAGE_REQUEST: u8 = 0xD;
const MESSAGE_RESPONSE: u8 = 0xE;

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
    PingRequest(u32 /*sender*/),
    PingResponse(u32 /*target*/),
    FindNodeRequest(u32 /*sender*/, u32 /*target*/),
    FindNodeResponse(Vec<Peer> /*peers_found*/),
    StoreRequest(u32 /*key*/, String /*message*/),
    StoreResponse(),
    FindValueRequest(u32 /*key*/),
    FindValueResponse(String /*message*/),

    // Message protocol
    MessageRequest(String /*message*/),
    MessageResponse(),
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
                CHUNK_REQUEST => {
                    if value.len() < MIN_CHUNK_REQUEST_SIZE {
                        bail!(
                            "can't decode chunk_request, size too low ({} < {})",
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
                CHUNK_RESPONSE => {
                    if value.len() < MIN_CHUNK_RESPONSE_SIZE {
                        bail!(
                            "can't decode chunk_reponse, size too low ({} < {})",
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
                PING_REQUEST => {
                    if value.len() < MIN_PING_REQUEST_SIZE {
                        bail!(
                            "can't decode ping_request, size too low ({} < {})",
                            value.len(),
                            MIN_PING_REQUEST_SIZE
                        );
                    }
                    let slice: [u8; 4] = core::array::from_fn(|idx| value[idx + ORDER_SIZE]);
                    let crc = u8_array_to_u32(&slice);
                    Self::PingRequest(crc)
                }
                PING_RESPONSE => {
                    if value.len() < MIN_PING_RESPONSE_SIZE {
                        bail!(
                            "can't decode ping_response, size too low ({} < {})",
                            value.len(),
                            MIN_PING_RESPONSE_SIZE
                        );
                    }
                    let slice: [u8; 4] = core::array::from_fn(|idx| value[idx + ORDER_SIZE]);
                    let crc = u8_array_to_u32(&slice);
                    Self::PingResponse(crc)
                }
                STORE_REQUEST => {
                    if value.len() < MIN_STORE_REQUEST_SIZE {
                        bail!(
                            "can't decode store_request, size too low ({} < {})",
                            value.len(),
                            MIN_STORE_REQUEST_SIZE
                        );
                    }
                    let slice: [u8; 4] = core::array::from_fn(|idx| value[idx + ORDER_SIZE]);
                    let crc = u8_array_to_u32(&slice);
                    let raw_str = value
                        .iter()
                        .skip(4 + ORDER_SIZE)
                        .map(|ch| *ch)
                        .collect::<Vec<u8>>();
                    let message = u8_array_to_string(raw_str.as_slice())?;
                    Self::StoreRequest(crc, message)
                }
                STORE_RESPONSE => {
                    if value.len() < MIN_STORE_RESPONSE_SIZE {
                        bail!(
                            "can't decode store_response, size too low ({} < {})",
                            value.len(),
                            MIN_STORE_RESPONSE_SIZE
                        );
                    }
                    Self::StoreResponse()
                }
                FIND_VALUE_REQUEST => {
                    if value.len() < MIN_FIND_VALUE_REQUEST_SIZE {
                        bail!(
                            "can't decode find_value_request, size too low ({} < {})",
                            value.len(),
                            MIN_FIND_VALUE_REQUEST_SIZE
                        );
                    }
                    let slice: [u8; 4] = core::array::from_fn(|idx| value[idx + ORDER_SIZE]);
                    let key = u8_array_to_u32(&slice);
                    Self::FindValueRequest(key)
                }
                FIND_VALUE_RESPONSE => {
                    if value.len() < MIN_FIND_VALUE_RESPONSE_SIZE {
                        bail!(
                            "can't decode find_value_response, size too low ({} < {})",
                            value.len(),
                            MIN_FIND_VALUE_RESPONSE_SIZE
                        );
                    }
                    let raw_str = value.iter().skip(ORDER_SIZE).map(|ch| *ch).collect::<Vec<u8>>();
                    let message = u8_array_to_string(raw_str.as_slice())?;
                    Self::FindValueResponse(message)
                }
                MESSAGE_REQUEST => {
                    if value.len() < MIN_MESSAGE_REQUEST_SIZE {
                        bail!(
                            "can't decode message_request, size too low ({} < {})",
                            value.len(),
                            MIN_MESSAGE_REQUEST_SIZE
                        );
                    }
                    let raw_str = value.iter().skip(ORDER_SIZE).map(|ch| *ch).collect::<Vec<u8>>();
                    let message = u8_array_to_string(raw_str.as_slice())?;
                    Self::MessageRequest(message)
                }
                MESSAGE_RESPONSE => {
                    if value.len() < MIN_MESSAGE_RESPONSE_SIZE {
                        bail!(
                            "can't decode message_response, size too low ({} < {})",
                            value.len(),
                            MIN_MESSAGE_RESPONSE_SIZE
                        );
                    }
                    Self::MessageResponse()
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
                let mut res = vec![CHUNK_REQUEST];
                res.extend(u32_to_u8_array(crc));
                res.extend(u32_to_u8_array(chunk_id));
                res
            }
            Command::ChunkResponse(crc, chunk_id, chunk_buf) => {
                let mut res = vec![CHUNK_RESPONSE];
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
            Command::PingRequest(crc) => {
                let mut res = vec![PING_REQUEST];
                res.extend(u32_to_u8_array(crc));
                res
            }
            Command::PingResponse(crc) => {
                let mut res = vec![PING_RESPONSE];
                res.extend(u32_to_u8_array(crc));
                res
            }
            Command::StoreRequest(key, message) => {
                let mut res = vec![STORE_REQUEST];
                res.extend(u32_to_u8_array(key));
                res.extend(string_to_u8_array(message));
                res
            }
            Command::StoreResponse() => {
                vec![STORE_RESPONSE]
            }
            Command::FindValueRequest(key) => {
                let mut res = vec![FIND_VALUE_REQUEST];
                res.extend(u32_to_u8_array(key));
                res
            }
            Command::FindValueResponse(message) => {
                let mut res = vec![FIND_VALUE_RESPONSE];
                res.extend(string_to_u8_array(message));
                res
            }
            Command::MessageRequest(message) => {
                let mut res = vec![MESSAGE_REQUEST];
                res.extend(string_to_u8_array(message));
                res
            }
            Command::MessageResponse() => {
                vec![MESSAGE_RESPONSE]
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

impl From<PeerNode> for Peer {
    fn from(value: PeerNode) -> Self {
        Self {
            id: value.id(),
            addr: value.addr(),
        }
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
    KeyNotFound = 4,
}

impl From<u8> for ErrorCode {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::FileNotFound,
            2 => Self::ChunkNotFound,
            3 => Self::InvalidChunk,
            4 => Self::KeyNotFound,
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
            ErrorCode::KeyNotFound => write!(fmt, "key not found"),
        }
    }
}

#[cfg(test)]
#[path = "protocol_test.rs"]
mod protocol_test;
