use super::ArtifactTreePathAllocator;

pub trait ArtifactTreeReadAllocator: ArtifactTreePathAllocator {
    fn allocate_read_buffer(&mut self, length: usize) -> Result<Vec<u8>, Self::Denial>;
}
