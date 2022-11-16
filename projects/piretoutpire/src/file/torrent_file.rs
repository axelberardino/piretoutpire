use super::file_chunk::{FileFixedSizedChunk, DEFAULT_CHUNK_SIZE};
use crate::{network::protocol::FileInfo, utils::div_ceil};
use crc32fast::Hasher;
use errors::{bail, reexports::eyre::ContextCompat, AnyResult};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

// TORRENT FILE ----------------------------------------------------------------

/// Hold metadata about a shared file.
#[derive(Debug)]
pub struct TorrentFile<P: AsRef<Path>> {
    torrent_file: P,
    pub metadata: Metadata,
}

impl<P: AsRef<Path>> TorrentFile<P> {
    // Create a new metadata file from a metadata info, and preallocate real
    // file on the disk.
    // If the file can't be written, an error will be raised.
    pub fn new(torrent_file: P, file_to_preallocate: P, file_info: &FileInfo) -> AnyResult<Self> {
        Self::new_with_chunk_size::<DEFAULT_CHUNK_SIZE>(torrent_file, file_to_preallocate, file_info)
    }

    // Create a new metadata file from an existing file.
    // If the file doesn't exist, an error will be raised.
    pub fn from(torrent_file: P, original_file: P) -> AnyResult<Self> {
        Self::from_with_chunk_size::<DEFAULT_CHUNK_SIZE>(torrent_file, original_file)
    }

    // Create a new metadata file from an existing file with a specific chunk_size.
    fn from_with_chunk_size<const CHUNK_SIZE: u32>(torrent_file: P, original_file: P) -> AnyResult<Self> {
        let torrent = Self {
            torrent_file,
            metadata: Metadata::extract_from_file::<P, CHUNK_SIZE>(original_file)?,
        };
        torrent.dump()?;
        Ok(torrent)
    }

    // Create a new metadata file from a metadata info, and preallocate real
    // file on the disk with a specific chunk_size.
    fn new_with_chunk_size<const CHUNK_SIZE: u32>(
        torrent_file: P,
        file_to_preallocate: P,
        file_info: &FileInfo,
    ) -> AnyResult<Self> {
        // Create metadata torrent file on disk.
        let torrent = Self {
            torrent_file,
            metadata: Metadata::new::<CHUNK_SIZE>(
                file_info.original_filename.clone(),
                file_to_preallocate.as_ref().display().to_string(),
                file_info.file_size,
                file_info.file_crc,
            ),
        };
        torrent.dump()?;

        // Allocate space on disk for the downloaded file.
        allocate_space_on_disk(file_to_preallocate, file_info.file_size as usize)?;

        Ok(torrent)
    }

    // Flush to a metadta file.
    pub fn dump(&self) -> AnyResult<()> {
        self.metadata.dump(&self.torrent_file)
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

    // Preallocate an in-memory metadata.
    pub fn preallocate(
        torrent_file: P,
        original_filename: String,
        file_size: u32,
        file_crc: u32,
        chunk_size: u32,
    ) -> Self {
        Self {
            metadata: Metadata {
                original_file: torrent_file
                    .as_ref()
                    .with_file_name(&original_filename)
                    .display()
                    .to_string(),
                file_size,
                file_crc,
                chunk_size,
                completed_chunks: vec![None; div_ceil(file_size, chunk_size) as usize],
                original_filename,
            },
            torrent_file,
        }
    }
}

// METADATA --------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    // Filename given by the peer
    pub original_filename: String,
    // Filename written locally on the disk
    pub original_file: String,
    // Size of the file
    pub file_size: u32,
    // Crc of the file
    pub file_crc: u32,
    // Size of an individual chunk
    pub chunk_size: u32,
    // contains none for incomplete chunks
    // contains the crc of already got chunks
    pub completed_chunks: Vec<Option<u32>>,
}

impl Metadata {
    fn new<const CHUNK_SIZE: u32>(
        original_filename: String,
        original_file: String,
        file_size: u32,
        file_crc: u32,
    ) -> Self {
        Self {
            original_filename,
            original_file,
            file_size,
            file_crc,
            chunk_size: CHUNK_SIZE,
            completed_chunks: vec![None; div_ceil(file_size, CHUNK_SIZE) as usize],
        }
    }

    // Load a file and create all in-memory metadata for this file.
    fn extract_from_file<P: AsRef<Path>, const CHUNK_SIZE: u32>(original_file: P) -> AnyResult<Self> {
        let mut chunks = FileFixedSizedChunk::<CHUNK_SIZE>::open_existing(&original_file)?;
        let mut whole_file_hasher = Hasher::new();
        let mut completed_chunks = vec![None; chunks.nb_chunks() as usize];

        for idx in 0..chunks.nb_chunks() {
            let chunk = chunks.read_chunk(idx)?;
            completed_chunks[idx as usize] = Some(crc32fast::hash(&chunk));
            whole_file_hasher.update(&chunk);
        }

        let filename = original_file.as_ref().file_name().context("invalid filename")?;
        let filename = filename.to_str().context("invalid string")?;
        Ok(Self {
            original_filename: filename.to_string(),
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

// HELPER ----------------------------------------------------------------------

// Create a new empty file with a specific size on the disk.
fn allocate_space_on_disk<P: AsRef<Path>>(filename: P, size: usize) -> AnyResult<()> {
    let file = OpenOptions::new()
        .truncate(true)
        .create(true)
        .write(true)
        .open(&filename)?;

    let mut writer = BufWriter::new(file);
    let mut buffer = [0; 1024];
    let mut remaining_size = size;

    while remaining_size > 0 {
        let to_write = std::cmp::min(remaining_size, buffer.len());
        let buffer = &mut buffer[..to_write];
        writer.write(buffer).unwrap();
        remaining_size -= to_write;
    }

    Ok(())
}

#[cfg(test)]
#[path = "torrent_file_test.rs"]
mod torrent_file_test;
