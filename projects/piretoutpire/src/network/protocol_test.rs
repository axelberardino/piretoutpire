use super::*;
use errors::AnyResult;

// SubMessages -----------------------------------------------------------------

#[test]
fn test_file_info_protocol() -> AnyResult<()> {
    let file_info = FileInfo {
        file_size: 1234,
        chunk_size: 2,
        file_crc: 3613099103,
        original_filename: "my_file.txt".to_owned(),
    };

    let raw_buf: Vec<u8> = file_info.into();
    let raw_buf = raw_buf.as_slice();
    #[rustfmt::skip]
    assert_eq!(&[
            0, 0, 4, 210,
            0, 0, 0, 2,
            215, 91, 132, 95,
            0, 0, 0, 11, 109, 121, 95, 102, 105, 108, 101, 46, 116, 120, 116
        ],
        raw_buf
    );

    let decoded_file_info = FileInfo::try_from(raw_buf)?;
    assert_eq!(1234, decoded_file_info.file_size);
    assert_eq!(2, decoded_file_info.chunk_size);
    assert_eq!(3613099103, decoded_file_info.file_crc);
    assert_eq!("my_file.txt".to_owned(), decoded_file_info.original_filename);

    Ok(())
}

#[test]
fn test_peer_protocol() -> AnyResult<()> {
    let peer = Peer {
        id: 1234,
        addr: "127.0.0.1:4000".parse()?,
    };

    let raw_buf: Vec<u8> = peer.into();
    let raw_buf = raw_buf.as_slice();
    #[rustfmt::skip]
    assert_eq!(&[
            0, 0, 4, 210,
            0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 52, 48, 48, 48
        ],
        raw_buf
    );

    let decoded_peer = Peer::try_from(raw_buf)?;
    assert_eq!(1234, decoded_peer.id);
    assert_eq!("127.0.0.1:4000".parse::<SocketAddr>()?, decoded_peer.addr);

    Ok(())
}

// Messages --------------------------------------------------------------------

#[test]
fn test_file_info_request_protocol() -> AnyResult<()> {
    let cmd = Command::FileInfoRequest(1234);
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            1,
            0, 0, 4, 210,
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::FileInfoRequest(crc) => assert_eq!(1234, crc),
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_file_info_response_protocol() -> AnyResult<()> {
    let cmd = Command::FileInfoResponse(FileInfo {
        file_size: 38,
        chunk_size: 42,
        file_crc: 99887766,
        original_filename: "filename".to_owned(),
    });
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            2,
            0, 0, 0, 38,
            0, 0, 0, 42,
            5, 244, 42, 150,
            0, 0, 0, 8, 102, 105, 108, 101, 110, 97, 109, 101
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::FileInfoResponse(file_info) => {
            assert_eq!(38, file_info.file_size);
            assert_eq!(42, file_info.chunk_size);
            assert_eq!(99887766, file_info.file_crc);
            assert_eq!("filename", file_info.original_filename);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_chunk_request_protocol() -> AnyResult<()> {
    let cmd = Command::ChunkRequest(1234, 4567);
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            3,
            0, 0, 4, 210,
            0, 0, 17, 215
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::ChunkRequest(crc, chunk_id) => {
            assert_eq!(1234, crc);
            assert_eq!(4567, chunk_id);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_chunk_response_protocol() -> AnyResult<()> {
    let cmd = Command::ChunkResponse(1234, 4567, vec![90, 48, 234]);
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            4,
            0, 0, 4, 210,
            0, 0, 17, 215,
            90, 48, 234
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::ChunkResponse(crc, chunk_id, chunk) => {
            assert_eq!(1234, crc);
            assert_eq!(4567, chunk_id);
            assert_eq!(vec![90, 48, 234], chunk);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_ping_request_protocol() -> AnyResult<()> {
    let peer = Peer {
        id: 1234,
        addr: "127.0.0.1:4000".parse()?,
    };

    let cmd = Command::PingRequest(peer.clone());
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            5,
            0, 0, 4, 210,
            0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 52, 48, 48, 48,
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::PingRequest(sender) => {
            assert_eq!(peer, sender);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_ping_response_protocol() -> AnyResult<()> {
    let cmd = Command::PingResponse(1234);
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            6,
            0, 0, 4, 210,
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::PingResponse(target) => {
            assert_eq!(1234, target);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_find_node_request_protocol() -> AnyResult<()> {
    let peer = Peer {
        id: 1234,
        addr: "127.0.0.1:4000".parse()?,
    };

    let cmd = Command::FindNodeRequest(peer.clone(), 4567);
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            9,
            0, 0, 4, 210,
            0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 52, 48, 48, 48,
            0, 0, 17, 215,
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::FindNodeRequest(sender, target) => {
            assert_eq!(peer, sender);
            assert_eq!(4567, target);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_find_node_response_protocol() -> AnyResult<()> {
    let cmd = Command::FindNodeResponse(vec![Peer {
        id: 1234,
        addr: "127.0.0.1:4000".parse()?,
    }]);
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            10,
            0, 0, 0, 1,
                0, 0, 4, 210,
                0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 52, 48, 48, 48
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::FindNodeResponse(peers) => {
            assert_eq!(1, peers.len());
            let peer = &peers[0];
            assert_eq!(1234, peer.id);
            assert_eq!("127.0.0.1:4000".parse::<SocketAddr>()?, peer.addr);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_find_node_list_response_protocol() -> AnyResult<()> {
    let cmd = Command::FindNodeResponse(vec![
        Peer {
            id: 1234,
            addr: "127.0.0.1:4000".parse()?,
        },
        Peer {
            id: 4567,
            addr: "127.0.0.1:5000".parse()?,
        },
    ]);
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            10,
            0, 0, 0, 2,
                0, 0, 4, 210,
                0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 52, 48, 48, 48,
                0, 0, 17, 215,
                0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 53, 48, 48, 48
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::FindNodeResponse(peers) => {
            assert_eq!(2, peers.len());
            let peer = &peers[0];
            assert_eq!(1234, peer.id);
            assert_eq!("127.0.0.1:4000".parse::<SocketAddr>()?, peer.addr);
            let peer = &peers[1];
            assert_eq!(4567, peer.id);
            assert_eq!("127.0.0.1:5000".parse::<SocketAddr>()?, peer.addr);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_store_request_protocol() -> AnyResult<()> {
    let peer = Peer {
        id: 1234,
        addr: "127.0.0.1:4000".parse()?,
    };

    let cmd = Command::StoreRequest(peer.clone(), 666, "hello".to_owned());
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            7,
            0, 0, 4, 210,
            0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 52, 48, 48, 48,
            0, 0, 2, 154,
            0, 0, 0, 5, 104, 101, 108, 108, 111
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::StoreRequest(sender, key, message) => {
            assert_eq!(peer, sender);
            assert_eq!(666, key);
            assert_eq!("hello", message);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_store_response_protocol() -> AnyResult<()> {
    let cmd = Command::StoreResponse();
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            8,
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::StoreResponse() => {}
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_find_value_request_protocol() -> AnyResult<()> {
    let peer = Peer {
        id: 1234,
        addr: "127.0.0.1:4000".parse()?,
    };

    let cmd = Command::FindValueRequest(peer.clone(), 666);
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            11,
            0, 0, 4, 210,
            0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 52, 48, 48, 48,
            0, 0, 2, 154
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::FindValueRequest(sender, key) => {
            assert_eq!(peer, sender);
            assert_eq!(666, key);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_find_value_response_protocol() -> AnyResult<()> {
    let cmd = Command::FindValueResponse("hello".to_owned());
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            12,
            0, 0, 0, 5, 104, 101, 108, 108, 111
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::FindValueResponse(message) => {
            assert_eq!("hello", message);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_message_request_protocol() -> AnyResult<()> {
    let cmd = Command::MessageRequest("hello".to_owned());
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            13,
            0, 0, 0, 5, 104, 101, 108, 108, 111
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::MessageRequest(message) => {
            assert_eq!("hello", message);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_message_response_protocol() -> AnyResult<()> {
    let cmd = Command::MessageResponse();
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            14,
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::MessageResponse() => {}
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_announce_request_protocol() -> AnyResult<()> {
    let peer = Peer {
        id: 1234,
        addr: "127.0.0.1:4000".parse()?,
    };

    let cmd = Command::AnnounceRequest(peer.clone(), 4567);
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            15,
            0, 0, 4, 210,
            0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 52, 48, 48, 48,
            0, 0, 17, 215
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::AnnounceRequest(sender, crc) => {
            assert_eq!(peer, sender);
            assert_eq!(4567, crc);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_announce_response_protocol() -> AnyResult<()> {
    let cmd = Command::AnnounceResponse();
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            16,
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::AnnounceResponse() => {}
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_get_peers_request_protocol() -> AnyResult<()> {
    let cmd = Command::GetPeersRequest(1234);
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            17,
            0, 0, 4, 210,
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::GetPeersRequest(crc) => assert_eq!(1234, crc),
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_get_peers_response_protocol() -> AnyResult<()> {
    let cmd = Command::GetPeersResponse(vec![Peer {
        id: 1234,
        addr: "127.0.0.1:4000".parse()?,
    }]);
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            18,
            0, 0, 0, 1,
                0, 0, 4, 210,
                0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 52, 48, 48, 48
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::GetPeersResponse(peers) => {
            assert_eq!(1, peers.len());
            let peer = &peers[0];
            assert_eq!(1234, peer.id);
            assert_eq!("127.0.0.1:4000".parse::<SocketAddr>()?, peer.addr);
        }
        _ => panic!(),
    }

    Ok(())
}

#[test]
fn test_get_peers_list_response_protocol() -> AnyResult<()> {
    let cmd = Command::GetPeersResponse(vec![
        Peer {
            id: 1234,
            addr: "127.0.0.1:4000".parse()?,
        },
        Peer {
            id: 4567,
            addr: "127.0.0.1:5000".parse()?,
        },
    ]);
    let raw_buf: Vec<u8> = cmd.into();
    let raw_buf = raw_buf.as_slice();

    #[rustfmt::skip]
    assert_eq!(&[
            18,
            0, 0, 0, 2,
                0, 0, 4, 210,
                0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 52, 48, 48, 48,
                0, 0, 17, 215,
                0, 0, 0, 14, 49, 50, 55, 46, 48, 46, 48, 46, 49, 58, 53, 48, 48, 48
        ],
        raw_buf
    );

    match Command::try_from(raw_buf)? {
        Command::GetPeersResponse(peers) => {
            assert_eq!(2, peers.len());
            let peer = &peers[0];
            assert_eq!(1234, peer.id);
            assert_eq!("127.0.0.1:4000".parse::<SocketAddr>()?, peer.addr);
            let peer = &peers[1];
            assert_eq!(4567, peer.id);
            assert_eq!("127.0.0.1:5000".parse::<SocketAddr>()?, peer.addr);
        }
        _ => panic!(),
    }

    Ok(())
}
