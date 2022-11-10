use errors::{bail, AnyResult};

/// The range of the wanted pieces.
pub struct ChunkRange((u32, u32));

impl ChunkRange {
    pub fn new(from: u32, to: u32) -> AnyResult<Self> {
        if from < to {
            bail!("invalid from({}) which is lesser than to({})", from, to);
        }
        Ok(Self { 0: (from, to) })
    }

    pub fn get(&self) -> (u32, u32) {
        self.0
    }
}

/// ChunkPieces represents chunk wanted or already owned as a compact form.
pub struct ChunkPieces(Vec<ChunkRange>);

impl ChunkPieces {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, range: ChunkRange) {
        self.0.push(range);
    }

    pub fn iter(&self) -> impl Iterator<Item = &ChunkRange> {
        self.0.iter()
    }
}
