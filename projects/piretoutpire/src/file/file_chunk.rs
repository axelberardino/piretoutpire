use errors::{bail, AnyResult, Context};
use std::{
    cmp::min,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write},
    path::Path,
};

const CHUNK_SIZE: usize = 16 * 1024; // 16 Ko chunks.

pub struct FileChunk {
    file_size: usize,
    chunk_size: usize,
    writer: BufWriter<File>,
    reader: BufReader<File>,
}

impl FileChunk {
    // Open an already existing file. File must exists.
    pub fn open_existing<P>(path: P) -> AnyResult<Self>
    where
        P: AsRef<Path>,
    {
        let file = OpenOptions::new().read(true).open(&path)?;
        Ok(Self {
            file_size: file.metadata().context("can't get file size")?.len() as usize,
            chunk_size: CHUNK_SIZE,
            writer: BufWriter::new(OpenOptions::new().write(true).open(&path)?),
            reader: BufReader::new(file),
        })
    }

    // Open and allocate space for a new file. Existing file will be truncated.
    pub fn open_new<P>(path: P, preallocated_size: usize) -> AnyResult<Self>
    where
        P: AsRef<Path>,
    {
        // Care order matter! Write + create will create the file.
        Ok(Self {
            file_size: preallocated_size,
            chunk_size: preallocated_size,
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

    // Change the default chunk size.
    pub fn set_chunk_size(&mut self, size: usize) {
        self.chunk_size = size;
    }

    // Read a chunk by its index.
    pub fn read_chunk(&mut self, chunk_id: usize) -> AnyResult<Vec<u8>> {
        read_range(
            &mut self.reader,
            chunk_id * self.chunk_size,
            min((chunk_id + 1) * self.chunk_size, self.file_size),
        )
    }

    // Write a chunk to its index.
    pub fn write_chunk(&mut self, chunk_id: usize, data: &[u8]) -> AnyResult<()> {
        write_range(
            &mut self.writer,
            chunk_id * self.chunk_size,
            min((chunk_id + 1) * self.chunk_size, self.file_size),
            data,
        )
    }
}

// Read a range of bytes from a buffer.
fn read_range<T>(br: &mut T, from: usize, to: usize) -> AnyResult<Vec<u8>>
where
    T: BufRead + Seek,
{
    if from > to {
        bail!("From({}) is > than to({})", from, to);
    }

    br.seek(SeekFrom::Start(from as u64))?;
    let mut buf = vec![0u8; to - from];
    br.read_exact(&mut buf)?;
    Ok(buf.to_vec())
}

// Write a range of bytes into a buffer.
fn write_range<T>(bw: &mut T, from: usize, to: usize, data: &[u8]) -> AnyResult<()>
where
    T: Write + Seek,
{
    if from > to {
        bail!("From({}) is > than to({}) with data({})", from, to, data.len());
    }
    if to - from != data.len() {
        bail!(
            "Invalid chunk size from({}) + to({}) != data({})",
            from,
            to,
            data.len()
        );
    }
    bw.seek(SeekFrom::Start(from as u64))?;
    bw.write(data)?;
    bw.flush()?;
    Ok(())
}

#[cfg(test)]
#[path = "file_chunk_test.rs"]
mod file_chunk_test;
