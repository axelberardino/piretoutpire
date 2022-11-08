#[derive(Debug)]
pub struct Chunk<const COUNT: usize>([u8; COUNT]);

impl<const COUNT: usize> From<[u8; COUNT]> for Chunk<COUNT> {
    fn from(value: [u8; COUNT]) -> Self {
        Chunk(value)
    }
}

impl<const COUNT: usize> From<Chunk<COUNT>> for [u8; COUNT] {
    fn from(value: Chunk<COUNT>) -> Self {
        value.0
    }
}
