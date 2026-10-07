use std::ffi::{OsStr, OsString};

use super::{
    ObservedWalArtifact, RecoveryDiscoveryAllocationFailure, RecoveryDiscoveryArtifact,
    RecoveryDiscoveryFailure,
};
use crate::filesystem_media::{
    ArtifactTreeDirectoryEntry, ArtifactTreeListingAllocator, ArtifactTreeListingStorageChange,
    ArtifactTreePathAllocationBoundary, ArtifactTreePathAllocator, ArtifactTreeReadAllocator,
    ArtifactTreeStorageAllocator,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryWalListingAllocationMode {
    CallerManaged,
    AdmissionCallbacks,
}

/// Storage mechanics only; C.4 retains authority over names and media reads.
pub trait RecoveryWalReadStorage: ArtifactTreeListingAllocator + ArtifactTreeReadAllocator {
    fn listing_allocation_mode(&self) -> RecoveryWalListingAllocationMode;
    fn allocate_wal_roster(
        &mut self,
        count: usize,
    ) -> Result<Vec<ObservedWalArtifact>, Self::Denial>;
    fn allocate_context(&mut self, count: usize) -> Result<OsString, Self::Denial>;
}

pub(super) struct CallbackWalStorage<R, P, C> {
    roster: Option<R>,
    payload: P,
    context: C,
}

impl<R, P, C> CallbackWalStorage<R, P, C> {
    pub(super) fn new(roster: R, payload: P, context: C) -> Self {
        Self {
            roster: Some(roster),
            payload,
            context,
        }
    }
}

impl<E, R, P, C> ArtifactTreeStorageAllocator for CallbackWalStorage<R, P, C>
where
    R: FnOnce(usize) -> Result<Vec<ObservedWalArtifact>, E>,
    P: FnMut(usize) -> Result<Vec<u8>, E>,
    C: FnMut(usize) -> Result<OsString, E>,
{
    type Denial = E;
}

impl<E, R, P, C> ArtifactTreePathAllocator for CallbackWalStorage<R, P, C>
where
    R: FnOnce(usize) -> Result<Vec<ObservedWalArtifact>, E>,
    P: FnMut(usize) -> Result<Vec<u8>, E>,
    C: FnMut(usize) -> Result<OsString, E>,
{
    type PathBacking = ();

    fn admit_path_backing(
        &mut self,
        _: ArtifactTreePathAllocationBoundary,
        _: u64,
    ) -> Result<(), E> {
        Ok(())
    }
}

impl<E, R, P, C> ArtifactTreeReadAllocator for CallbackWalStorage<R, P, C>
where
    R: FnOnce(usize) -> Result<Vec<ObservedWalArtifact>, E>,
    P: FnMut(usize) -> Result<Vec<u8>, E>,
    C: FnMut(usize) -> Result<OsString, E>,
{
    fn allocate_read_buffer(&mut self, length: usize) -> Result<Vec<u8>, E> {
        (self.payload)(length)
    }
}

impl<E, R, P, C> ArtifactTreeListingAllocator for CallbackWalStorage<R, P, C>
where
    R: FnOnce(usize) -> Result<Vec<ObservedWalArtifact>, E>,
    P: FnMut(usize) -> Result<Vec<u8>, E>,
    C: FnMut(usize) -> Result<OsString, E>,
{
    fn listing_storage_change(&mut self, _: ArtifactTreeListingStorageChange) -> Result<(), E> {
        Ok(())
    }

    fn allocate_listing_roster(
        &mut self,
        count: usize,
    ) -> Result<Vec<ArtifactTreeDirectoryEntry>, E> {
        Ok(Vec::with_capacity(count))
    }
}

impl<E, R, P, C> RecoveryWalReadStorage for CallbackWalStorage<R, P, C>
where
    R: FnOnce(usize) -> Result<Vec<ObservedWalArtifact>, E>,
    P: FnMut(usize) -> Result<Vec<u8>, E>,
    C: FnMut(usize) -> Result<OsString, E>,
{
    fn listing_allocation_mode(&self) -> RecoveryWalListingAllocationMode {
        RecoveryWalListingAllocationMode::CallerManaged
    }

    fn allocate_wal_roster(&mut self, count: usize) -> Result<Vec<ObservedWalArtifact>, E> {
        self.roster
            .take()
            .expect("the WAL roster is requested once")(count)
    }

    fn allocate_context(&mut self, count: usize) -> Result<OsString, E> {
        (self.context)(count)
    }
}

pub(super) fn allocate_wal_roster<E>(
    count: usize,
    allocate: impl FnOnce(usize) -> Result<Vec<ObservedWalArtifact>, E>,
) -> Result<Vec<ObservedWalArtifact>, RecoveryDiscoveryAllocationFailure<E>> {
    let requested = wal_roster_bytes(count)?;
    let roster =
        allocate(count).map_err(|cause| RecoveryDiscoveryAllocationFailure::Allocation {
            artifact: RecoveryDiscoveryArtifact::WalDirectory,
            offset: 0,
            requested,
            cause,
        })?;
    let mismatch = if !roster.is_empty() {
        Some((0, wal_roster_bytes(roster.len())?))
    } else if roster.capacity() < count {
        Some((requested, wal_roster_bytes(roster.capacity())?))
    } else {
        None
    };
    if let Some((requested, observed)) = mismatch {
        return Err(RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
            artifact: RecoveryDiscoveryArtifact::WalDirectory,
            offset: 0,
            requested,
            observed,
        });
    }
    Ok(roster)
}

fn wal_roster_bytes(count: usize) -> Result<usize, RecoveryDiscoveryFailure> {
    count
        .checked_mul(std::mem::size_of::<ObservedWalArtifact>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or_else(|| RecoveryDiscoveryFailure::invalid(RecoveryDiscoveryArtifact::WalDirectory))
}

pub(super) fn allocate_wal_context<E>(
    name: &OsStr,
    allocate: &mut impl FnMut(usize) -> Result<OsString, E>,
) -> Result<RecoveryDiscoveryArtifact, RecoveryDiscoveryAllocationFailure<E>> {
    let requested = name.as_encoded_bytes().len();
    let mut context =
        allocate(requested).map_err(|cause| RecoveryDiscoveryAllocationFailure::Allocation {
            artifact: RecoveryDiscoveryArtifact::WalDirectory,
            offset: 0,
            requested,
            cause,
        })?;
    let mismatch = if !context.is_empty() {
        Some((0, context.len()))
    } else if context.capacity() < requested {
        Some((requested, context.capacity()))
    } else {
        None
    };
    if let Some((requested, observed)) = mismatch {
        return Err(RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
            artifact: RecoveryDiscoveryArtifact::WalDirectory,
            offset: 0,
            requested,
            observed,
        });
    }
    context.push(name);
    Ok(RecoveryDiscoveryArtifact::WalArtifact(context))
}
