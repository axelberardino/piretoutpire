use super::*;

#[test]
fn test_read_one_big_chunk() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let data: Vec<u8> = vec![0, 1, 2, 3, 4];
    let mut fc = FileChunk::open_new(tmp_file.path(), data.len() as u64).expect("file must exists!");

    fc.write_chunk(0, &data)?;
    let got = fc.read_chunk(0)?;

    assert_eq!(data, got);
    Ok(())
}

#[test]
fn test_read_no_remains_chunks() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let data: Vec<u8> = vec![0, 1, 2, 3, 4];
    let mut fc =
        FileFixedSizedChunk::<1>::open_new(tmp_file.path(), data.len() as u64).expect("file must exists!");

    for (idx, item) in data.iter().enumerate() {
        fc.write_chunk(idx as u64, &[*item])?;
    }

    for (idx, expected) in data.iter().enumerate() {
        let got = fc.read_chunk(idx as u64)?;
        assert_eq!(vec![*expected], got);
    }

    Ok(())
}

#[test]
fn test_read_with_remains_chunks() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let data: Vec<u8> = vec![0, 1, 2, 3, 4];
    let mut fc =
        FileFixedSizedChunk::<2>::open_new(tmp_file.path(), data.len() as u64).expect("file must exists!");

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
        let mut fc = FileChunk::open_new(tmp_file.path(), data.len() as u64).expect("file must exists!");
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
fn test_invalid_init() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let fc = FileChunk::open_new(tmp_file.path(), 0);
    assert!(fc.is_err());

    Ok(())
}

#[test]
fn test_invalid_chunk_index() -> AnyResult<()> {
    let tmp_file = temp_file::empty();
    let data: Vec<u8> = vec![0, 1, 2, 3, 4];
    let mut fc = FileChunk::open_new(tmp_file.path(), data.len() as u64).expect("file must exists!");

    let res = fc.write_chunk(42, &data);
    assert!(res.is_err());

    let got = fc.read_chunk(42);
    assert!(got.is_err());

    let res = fc.write_chunk(42, &data);
    assert!(res.is_err());

    Ok(())
}

#[test]
fn test_nb_chunks() -> AnyResult<()> {
    {
        let tmp_file = temp_file::empty();
        let fc = FileChunk::open_new(tmp_file.path(), 5).expect("file must exists!");
        assert_eq!(1, fc.nb_chunks());
    }
    {
        let tmp_file = temp_file::empty();
        let fc = FileFixedSizedChunk::<1>::open_new(tmp_file.path(), 5).expect("file must exists!");
        assert_eq!(5, fc.nb_chunks());
    }
    {
        let tmp_file = temp_file::empty();
        let fc = FileFixedSizedChunk::<2>::open_new(tmp_file.path(), 5).expect("file must exists!");
        assert_eq!(3, fc.nb_chunks());
    }

    Ok(())
}
