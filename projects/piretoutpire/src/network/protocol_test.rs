use super::*;
use crate::file::file_chunk::DEFAULT_CHUNK_SIZE;
use errors::AnyResult;

#[test]
fn test_file_info_protocol() -> AnyResult<()> {
    let file_info = FileInfo {
        file_size: 1234,
        chunk_size: DEFAULT_CHUNK_SIZE,
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
    assert_eq!(DEFAULT_CHUNK_SIZE, decoded_file_info.chunk_size);
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
