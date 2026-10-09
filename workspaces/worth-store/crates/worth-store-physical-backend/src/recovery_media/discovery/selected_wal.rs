//! Mechanical constraints for rereading a caller's already selected inventory.

use crate::filesystem_media::{
    ArtifactTreeDirectoryEntry, ArtifactTreePathAllocationBoundary, ArtifactTreePathAllocator,
    ArtifactTreeReadAllocator, ArtifactTreeStorageAllocator,
};
use std::ffi::OsStr;

/// Membership and lengths are supplied by the caller; this confers no selection authority.
pub trait RecoveryWalReadSelection {
    fn matches_listing(&self, entries: &mut [ArtifactTreeDirectoryEntry]) -> bool;
    fn expected_file_length(&self, name: &OsStr) -> Option<u64>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryWalSelectionMismatch {
    Listing,
    FileLength { expected: u64, observed: u64 },
}

#[derive(Debug)]
pub enum RecoverySelectedWalReadOutcome<T> {
    Observed(T),
    Mismatch(RecoveryWalSelectionMismatch),
}

pub(super) enum SelectedReadDenial<E> {
    Storage(E),
    Length(RecoveryWalSelectionMismatch),
}

/// Forwards the actual native issuer and path custody, intercepting metadata length only.
pub(super) struct SelectedReadStorage<'storage, S> {
    pub(super) storage: &'storage mut S,
    pub(super) expected: Option<u64>,
}

impl<S: ArtifactTreeStorageAllocator> ArtifactTreeStorageAllocator for SelectedReadStorage<'_, S> {
    type Denial = SelectedReadDenial<S::Denial>;
}

impl<S: ArtifactTreePathAllocator> ArtifactTreePathAllocator for SelectedReadStorage<'_, S> {
    type PathBacking = S::PathBacking;

    fn admit_path_backing(
        &mut self,
        boundary: ArtifactTreePathAllocationBoundary,
        bytes: u64,
    ) -> Result<Self::PathBacking, Self::Denial> {
        self.storage
            .admit_path_backing(boundary, bytes)
            .map_err(SelectedReadDenial::Storage)
    }
}

impl<S: ArtifactTreeReadAllocator> ArtifactTreeReadAllocator for SelectedReadStorage<'_, S> {
    fn allocate_read_buffer(&mut self, length: usize) -> Result<Vec<u8>, Self::Denial> {
        if let Some(expected) = self.expected {
            if length as u64 != expected {
                return Err(SelectedReadDenial::Length(
                    RecoveryWalSelectionMismatch::FileLength {
                        expected,
                        observed: length as u64,
                    },
                ));
            }
        }
        self.storage
            .allocate_read_buffer(length)
            .map_err(SelectedReadDenial::Storage)
    }
}
