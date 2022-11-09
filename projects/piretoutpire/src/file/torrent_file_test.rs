use super::*;
use crate::file::file_chunk::FileFixedSizedChunk;

#[test]
fn test_create_metadata_from_one_chunk_file() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let data: Vec<u8> = vec![0, 1, 2, 3, 4];
    let mut fc = FileChunk::open_new(tmp_file.path(), data.len() as u64).expect("file must exists!");
    fc.write_chunk(0, &data)?;

    let tmp_torrent_file = temp_file::empty();
    let torrent = TorrentFile::new(tmp_torrent_file.path(), tmp_file.path())?;
    assert_eq!(1364906956, torrent.metadata.file_crc);
    assert_eq!(5, torrent.metadata.file_size);
    assert_eq!(DEFAULT_CHUNK_SIZE, torrent.metadata.chunk_size);
    assert_eq!(vec![Some(1364906956)], torrent.metadata.completed_chunks);

    Ok(())
}

#[test]
fn test_create_metadata_from_many_chunks_file() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let data: Vec<u8> = vec![0, 1, 2, 3, 4];
    let mut fc =
        FileFixedSizedChunk::<1>::open_new(tmp_file.path(), data.len() as u64).expect("file must exists!");
    for (idx, item) in data.iter().enumerate() {
        fc.write_chunk(idx as u64, &[*item])?;
    }

    let tmp_torrent_file = temp_file::empty();
    let torrent = TorrentFile::new_with_chunk_size::<1>(tmp_torrent_file.path(), tmp_file.path())?;
    assert_eq!(1364906956, torrent.metadata.file_crc);
    assert_eq!(5, torrent.metadata.file_size);
    assert_eq!(1, torrent.metadata.chunk_size);
    assert_eq!(
        vec![
            Some(3523407757),
            Some(2768625435),
            Some(1007455905),
            Some(1259060791),
            Some(3580832660)
        ],
        torrent.metadata.completed_chunks
    );

    Ok(())
}

#[test]
fn test_reload_existing_torrent() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let tmp_torrent_file = temp_file::empty();

    {
        let data: Vec<u8> = vec![0, 1, 2, 3, 4];
        let mut fc = FileChunk::open_new(tmp_file.path(), data.len() as u64).expect("file must exists!");
        fc.write_chunk(0, &data)?;

        _ = TorrentFile::new(tmp_torrent_file.path(), tmp_file.path())?;
    }

    let torrent = TorrentFile::load(tmp_torrent_file.path())?;
    assert_eq!(1364906956, torrent.metadata.file_crc);
    assert_eq!(5, torrent.metadata.file_size);
    assert_eq!(DEFAULT_CHUNK_SIZE, torrent.metadata.chunk_size);
    assert_eq!(vec![Some(1364906956)], torrent.metadata.completed_chunks);

    Ok(())
}

#[test]
fn test_reload_existing_torrent_but_not_found_associated_file() -> AnyResult<()> {
    let tmp_torrent_file = temp_file::empty();

    {
        let tmp_file = temp_file::empty();
        let data: Vec<u8> = vec![0, 1, 2, 3, 4];
        let mut fc = FileChunk::open_new(tmp_file.path(), data.len() as u64).expect("file must exists!");
        fc.write_chunk(0, &data)?;

        _ = TorrentFile::new(tmp_torrent_file.path(), tmp_file.path())?;
    }

    let torrent = TorrentFile::load(tmp_torrent_file.path());
    assert!(torrent.is_err());

    Ok(())
}

#[test]
fn test_reload_existing_torrent_but_invalid_crc_file() -> AnyResult<()> {
    let tmp_torrent_file = temp_file::empty();
    let tmp_file = temp_file::empty();

    {
        let data: Vec<u8> = vec![0, 1, 2, 3, 4];
        let mut fc = FileChunk::open_new(tmp_file.path(), data.len() as u64).expect("file must exists!");
        fc.write_chunk(0, &data)?;

        _ = TorrentFile::new(tmp_torrent_file.path(), tmp_file.path())?;
    }

    // Corrupt file.
    {
        let mut file = OpenOptions::new().write(true).open(tmp_file.path())?;
        file.write(&[4, 2, 1, 1, 1])?;
        file.flush()?;
    }

    let torrent = TorrentFile::load(tmp_torrent_file.path());
    assert!(torrent.is_err());

    Ok(())
}
