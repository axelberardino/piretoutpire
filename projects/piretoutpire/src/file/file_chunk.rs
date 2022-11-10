use errors::{bail, AnyResult, Context};
use std::{
    cmp::min,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write},
    path::Path,
};

pub const DEFAULT_CHUNK_SIZE: u64 = 1024 * 1024; // 1 Mo chunks.

pub type FileChunk = FileFixedSizedChunk<DEFAULT_CHUNK_SIZE>;

pub struct FileFixedSizedChunk<const CHUNK_SIZE: u64> {
    file_size: u64,
    writer: BufWriter<File>,
    reader: BufReader<File>,
}

impl<const CHUNK_SIZE: u64> FileFixedSizedChunk<CHUNK_SIZE> {
    // Open an already existing file. File must exists.
    pub fn open_existing<P>(path: P) -> AnyResult<Self>
    where
        P: AsRef<Path>,
    {
        let file = OpenOptions::new().read(true).open(&path)?;
        Ok(Self {
            file_size: file.metadata().context("can't get file size")?.len(),
            writer: BufWriter::new(OpenOptions::new().write(true).open(&path)?),
            reader: BufReader::new(file),
        })
    }

    // Open and allocate space for a new file. Existing file will be truncated.
    pub fn open_new<P>(path: P, preallocated_size: u64) -> AnyResult<Self>
    where
        P: AsRef<Path>,
    {
        if preallocated_size == 0 {
            bail!("initial allocated size can't be 0");
        }
        // Care order matter! Write + create will create the file.
        Ok(Self {
            file_size: preallocated_size,
            writer: BufWriter::new(
                OpenOptions::new()
                    .truncate(true)
                    .create(true)
                    .write(true)
                    .open(&path)?,
            ),
            reader: BufReader::new(OpenOptions::new().read(true).open(&path)?),
        })
    }

    // Read a chunk by its index.
    pub fn read_chunk(&mut self, chunk_id: u64) -> AnyResult<Vec<u8>> {
        read_range(
            &mut self.reader,
            chunk_id * CHUNK_SIZE,
            min((chunk_id + 1) * CHUNK_SIZE, self.file_size),
        )
    }

    // Write a chunk to its index.
    pub fn write_chunk(&mut self, chunk_id: u64, data: &[u8]) -> AnyResult<()> {
        write_range(
            &mut self.writer,
            chunk_id * CHUNK_SIZE,
            min((chunk_id + 1) * CHUNK_SIZE, self.file_size),
            data,
        )
    }

    // Number of chunks the file is currently split in.
    pub fn nb_chunks(&self) -> u64 {
        ((self.file_size + CHUNK_SIZE - 1) / CHUNK_SIZE) as u64
    }

    // Size of the file when retrieved.
    pub fn file_size(&self) -> u64 {
        self.file_size
    }
}

// Read a range of bytes from a buffer.
fn read_range<T>(br: &mut T, from: u64, to: u64) -> AnyResult<Vec<u8>>
where
    T: BufRead + Seek,
{
    if from > to {
        bail!("From({}) is > than to({})", from, to);
    }

    br.seek(SeekFrom::Start(from as u64))?;
    let mut buf = vec![0u8; (to - from) as usize];
    br.read_exact(&mut buf)?;
    Ok(buf.to_vec())
}

// Write a range of bytes into a buffer.
fn write_range<T>(bw: &mut T, from: u64, to: u64, data: &[u8]) -> AnyResult<()>
where
    T: Write + Seek,
{
    if from > to {
        bail!("From({}) is > than to({}) with data({})", from, to, data.len());
    }
    if to - from != data.len() as u64 {
        bail!(
            "Invalid chunk size from({}) + to({}) != data({})",
            from,
            to,
            data.len()
        );
    }
    bw.seek(SeekFrom::Start(from as u64))?;
    bw.write_all(data)?;
    bw.flush()?;
    Ok(())
}

#[cfg(test)]
#[path = "file_chunk_test.rs"]
mod file_chunk_test;
