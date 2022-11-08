use super::*;

#[test]
fn test_read_one_big_chunk() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let data: Vec<u8> = vec![0, 1, 2, 3, 4];
    let mut fc = FileChunk::open_new(tmp_file.path(), data.len()).expect("file must exists!");

    fc.write_chunk(0, &data)?;
    let got = fc.read_chunk(0)?;

    assert_eq!(data, got);
    Ok(())
}

#[test]
fn test_read_no_remains_chunks() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let data: Vec<u8> = vec![0, 1, 2, 3, 4];
    let mut fc = FileChunk::open_new(tmp_file.path(), data.len()).expect("file must exists!");
    fc.set_chunk_size(1);

    for (idx, item) in data.iter().enumerate() {
        fc.write_chunk(idx, &[*item])?;
    }

    for (idx, expected) in data.iter().enumerate() {
        let got = fc.read_chunk(idx)?;
        assert_eq!(vec![*expected], got);
    }

    Ok(())
}

#[test]
fn test_read_with_remains_chunks() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let data: Vec<u8> = vec![0, 1, 2, 3, 4];
    let mut fc = FileChunk::open_new(tmp_file.path(), data.len()).expect("file must exists!");
    fc.set_chunk_size(2);

    fc.write_chunk(0, &data[0..2])?;
    fc.write_chunk(1, &data[2..4])?;
    fc.write_chunk(2, &data[4..5])?;

    assert_eq!(&data[0..2], fc.read_chunk(0)?);
    assert_eq!(&data[2..4], fc.read_chunk(1)?);
    assert_eq!(&data[4..5], fc.read_chunk(2)?);

    Ok(())
}

#[test]
fn test_read_existing_file() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let data: Vec<u8> = vec![0, 1, 2, 3, 4];

    {
        let mut fc = FileChunk::open_new(tmp_file.path(), data.len()).expect("file must exists!");
        fc.write_chunk(0, &data)?;
    }

    let mut fc = FileChunk::open_existing(tmp_file.path()).expect("file must exists!");
    let got = fc.read_chunk(0)?;
    assert_eq!(data, got);

    Ok(())
}

#[test]
fn test_invalid_file() -> AnyResult<()> {
    let fc = FileChunk::open_existing("/tmp/unknown");
    assert!(fc.is_err());

    Ok(())
}

#[test]
fn test_invalid_chunk_index() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let data: Vec<u8> = vec![0, 1, 2, 3, 4];
    let mut fc = FileChunk::open_new(tmp_file.path(), data.len()).expect("file must exists!");

    let res = fc.write_chunk(42, &data);
    assert!(res.is_err());

    let got = fc.read_chunk(42);
    assert!(got.is_err());

    Ok(())
}
