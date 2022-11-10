use super::file_chunk::{FileFixedSizedChunk, DEFAULT_CHUNK_SIZE};
use crc32fast::Hasher;
use errors::{bail, AnyResult};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

/// Hold metadata about a shared file.
#[derive(Debug)]
pub struct TorrentFile<P: AsRef<Path>> {
    torrent_file: P,
    pub metadata: Metadata,
}

impl<P: AsRef<Path>> TorrentFile<P> {
    // Create a new metadata file from a file.
    // If the file doesn't exist, an error will be raised.
    pub fn new(torrent_file: P, original_file: P) -> AnyResult<Self> {
        Self::new_with_chunk_size::<DEFAULT_CHUNK_SIZE>(torrent_file, original_file)
    }

    pub fn new_with_chunk_size<const CHUNK_SIZE: u64>(torrent_file: P, original_file: P) -> AnyResult<Self> {
        let torrent = Self {
            torrent_file,
            metadata: Metadata::extract_from_file::<P, CHUNK_SIZE>(original_file)?,
        };
        torrent.metadata.dump(&torrent.torrent_file)?;

        Ok(torrent)
    }

    // Load an existing torrent metadata from an existing file.
    pub fn load(torrent_file: P) -> AnyResult<Self> {
        let metadata = Metadata::load(&torrent_file)?;
        let mut reader = BufReader::new(OpenOptions::new().read(true).open(&metadata.original_file)?);
        let mut whole_file_hasher = Hasher::new();

        loop {
            let buf = reader.fill_buf()?;
            let len = buf.len();
            if len == 0 {
                break;
            }
            whole_file_hasher.update(buf);
            reader.consume(len);
        }

        let got_crc = whole_file_hasher.finalize();

        if got_crc != metadata.file_crc {
            bail!(
                "crc is different expected {} but got {}",
                got_crc,
                metadata.file_crc
            );
        }

        Ok(Self {
            metadata,
            torrent_file,
        })
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub original_file: String,
    pub file_size: u64,
    pub file_crc: u32,
    pub chunk_size: u64,
    // contains none for incomplete chunks
    // contains the crc of already got chunks
    pub completed_chunks: Vec<Option<u32>>,
}

impl Metadata {
    // Load a file and create all in-memory metadata for this file.
    fn extract_from_file<P: AsRef<Path>, const CHUNK_SIZE: u64>(original_file: P) -> AnyResult<Self> {
        let mut chunks = FileFixedSizedChunk::<CHUNK_SIZE>::open_existing(&original_file)?;
        let mut whole_file_hasher = Hasher::new();
        let mut completed_chunks = vec![None; chunks.nb_chunks() as usize];

        for idx in 0..chunks.nb_chunks() {
            let chunk = chunks.read_chunk(idx)?;
            completed_chunks[idx as usize] = Some(crc32fast::hash(&chunk));
            whole_file_hasher.update(&chunk);
        }

        Ok(Self {
            original_file: original_file.as_ref().display().to_string(),
            file_size: chunks.file_size(),
            file_crc: whole_file_hasher.finalize(),
            chunk_size: CHUNK_SIZE,
            completed_chunks,
        })
    }

    // Load an existing torrent metadata from an existing file.
    fn load<P: AsRef<Path>>(torrent_file: P) -> AnyResult<Self> {
        let reader = BufReader::new(File::open(torrent_file)?);
        let raw: Metadata = serde_json::from_reader(reader)?;

        Ok(raw)
    }

    // Dump current information about the share file into a metadata file.
    fn dump<P: AsRef<Path>>(&self, torrent_file: P) -> AnyResult<()> {
        let raw_json_str = serde_json::to_string(&self)?;
        let mut writer = BufWriter::new(
            OpenOptions::new()
                .truncate(true)
                .create(true)
                .write(true)
                .open(torrent_file)?,
        );
        writer.write_all(raw_json_str.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
#[path = "torrent_file_test.rs"]
mod torrent_file_test;
