use super::ArtifactTreeStorageAllocator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactTreePathAllocationBoundary {
    DirectoryAddress,
    FileAddress,
    DirectoryOpen,
    FileOpen,
}

pub trait ArtifactTreePathAllocator: ArtifactTreeStorageAllocator {
    type PathBacking;

    fn admit_path_backing(
        &mut self,
        boundary: ArtifactTreePathAllocationBoundary,
        requested_bytes: u64,
    ) -> Result<Self::PathBacking, Self::Denial>;
}
