use super::{ArtifactTreeDirectoryEntry, ArtifactTreeFailure, ArtifactTreeStorageAllocator};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactTreeListingAllocationBoundary {
    ProviderPath,
    ProviderIterator,
    EntryName,
    EntryRoster,
}

/// Mechanical live-storage changes; these confer no filesystem authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactTreeListingStorageChange {
    Admit {
        boundary: ArtifactTreeListingAllocationBoundary,
        required_bytes: u64,
    },
    Settle {
        retained_bytes: u64,
    },
}

pub trait ArtifactTreeListingAllocator: ArtifactTreeStorageAllocator {
    fn listing_storage_change(
        &mut self,
        change: ArtifactTreeListingStorageChange,
    ) -> Result<(), Self::Denial>;

    fn allocate_listing_roster(
        &mut self,
        count: usize,
    ) -> Result<Vec<ArtifactTreeDirectoryEntry>, Self::Denial>;
}

#[derive(Debug)]
pub(crate) enum ArtifactTreeAllocatedListingFailure<E> {
    Media(ArtifactTreeFailure),
    Allocation { requested: u64, cause: E },
    BufferLengthMismatch { requested: usize, observed: usize },
}

impl<E> From<ArtifactTreeFailure> for ArtifactTreeAllocatedListingFailure<E> {
    fn from(failure: ArtifactTreeFailure) -> Self {
        Self::Media(failure)
    }
}

pub(super) fn change<A: ArtifactTreeListingAllocator>(
    allocator: &mut A,
    change: ArtifactTreeListingStorageChange,
) -> Result<(), ArtifactTreeAllocatedListingFailure<A::Denial>> {
    let requested = match change {
        ArtifactTreeListingStorageChange::Admit { required_bytes, .. } => required_bytes,
        ArtifactTreeListingStorageChange::Settle { retained_bytes } => retained_bytes,
    };
    allocator
        .listing_storage_change(change)
        .map_err(|cause| ArtifactTreeAllocatedListingFailure::Allocation { requested, cause })
}
