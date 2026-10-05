use std::{convert::Infallible, ffi::OsString, num::NonZeroU64};

use worth_store_physical_format::store_namespace::NamespaceEntryType;

use super::resident_read::map_allocated_failure;
use super::wal_storage::{allocate_wal_context, allocate_wal_roster, CallbackWalStorage};
use super::{
    ArtifactTreeDirectory, ArtifactTreeFailure, FilesystemObservation,
    FilesystemObservationAllowance, ObservedWalArtifact, RecoveryDiscoveryAllocationFailure,
    RecoveryDiscoveryArtifact, RecoveryDiscoveryCount, RecoveryDiscoveryFailure,
    RecoveryWalObservationIdentity,
};
use super::{
    RecoverySelectedWalReadOutcome as Outcome, RecoveryWalReadSelection,
    RecoveryWalSelectionMismatch,
};
use super::{RecoveryWalListingAllocationMode, RecoveryWalReadStorage};
use crate::filesystem_media::{
    backed_directory, backed_file, ArtifactTreeAllocatedListingFailure, ArtifactTreeBackedPath,
};

mod selected_file;

impl<M: super::DiscoveryMediaBacking> FilesystemObservation<M> {
    pub fn read_wal_artifacts(
        &mut self,
        maximum_segments: NonZeroU64,
        byte_limit: u64,
    ) -> Result<Vec<ObservedWalArtifact>, RecoveryDiscoveryFailure> {
        self.read_wal_artifacts_with_payload_allocator(
            maximum_segments,
            byte_limit,
            |length| -> Result<Vec<u8>, Infallible> { Ok(vec![0; length]) },
        )
        .map_err(|denial| match denial {
            RecoveryDiscoveryAllocationFailure::Discovery(denial) => denial,
            RecoveryDiscoveryAllocationFailure::Allocation { cause, .. } => match cause {},
            RecoveryDiscoveryAllocationFailure::BufferLengthMismatch { .. } => {
                unreachable!("the built-in WAL allocators return the requested storage")
            }
        })
    }

    /// Allocate each WAL payload after the admitted file length is known and
    /// before reading its bytes. Directory names and the result roster remain
    /// outside this payload-only allocator boundary.
    pub fn read_wal_artifacts_with_payload_allocator<E>(
        &mut self,
        maximum_segments: NonZeroU64,
        byte_limit: u64,
        allocate: impl FnMut(usize) -> Result<Vec<u8>, E>,
    ) -> Result<Vec<ObservedWalArtifact>, RecoveryDiscoveryAllocationFailure<E>> {
        self.read_wal_artifacts_with_allocators(
            maximum_segments,
            byte_limit,
            |count| Ok(Vec::with_capacity(count)),
            allocate,
            |count| Ok(OsString::with_capacity(count)),
        )
    }

    /// Allocate the empty result roster after listing establishes its count,
    /// before reading any payload. The roster allocator receives a slot count;
    /// allocation-failure requests and buffer mismatches report backing bytes.
    ///
    /// The context allocator supplies empty storage for one diagnostic name;
    /// C.4 fills it with the listed name before address validation or reading.
    /// A successful read disposes that context
    /// before the next callback; a failed read retains it in the failure.
    /// Provider, listing, original-name, and path allocations remain outside
    /// this boundary. Listed names move into their observations without a copy.
    pub fn read_wal_artifacts_with_allocators<E>(
        &mut self,
        maximum_segments: NonZeroU64,
        byte_limit: u64,
        allocate_roster: impl FnOnce(usize) -> Result<Vec<ObservedWalArtifact>, E>,
        allocate_payload: impl FnMut(usize) -> Result<Vec<u8>, E>,
        allocate_context: impl FnMut(usize) -> Result<OsString, E>,
    ) -> Result<Vec<ObservedWalArtifact>, RecoveryDiscoveryAllocationFailure<E>> {
        let mut storage =
            CallbackWalStorage::new(allocate_roster, allocate_payload, allocate_context);
        self.read_wal_artifacts_with_storage(maximum_segments, byte_limit, &mut storage)
    }

    pub fn wal_listing_storage_is_qualified(&self) -> bool {
        self.parts
            .artifact_tree()
            .listing_storage_requirement(1)
            .is_some()
    }

    pub fn wal_path_storage_is_qualified(&self) -> bool {
        self.parts.artifact_tree().path_storage_is_qualified()
    }

    /// One read algorithm for caller-managed or staged admitted listing storage.
    /// Admitted mode requires qualified path and listing mechanics before the
    /// first address allocation. Caller-managed mode retains its raw behavior.
    pub fn read_wal_artifacts_with_storage<S: RecoveryWalReadStorage>(
        &mut self,
        maximum_segments: NonZeroU64,
        byte_limit: u64,
        storage: &mut S,
    ) -> Result<Vec<ObservedWalArtifact>, RecoveryDiscoveryAllocationFailure<S::Denial>> {
        match self.read_wal_inventory(maximum_segments, byte_limit, storage, None)? {
            Outcome::Observed(observed) => Ok(observed),
            Outcome::Mismatch(_) => unreachable!("unconstrained reads have no selected inventory"),
        }
    }

    pub fn read_selected_wal_artifacts_with_storage<S: RecoveryWalReadStorage>(
        &mut self,
        maximum_segments: NonZeroU64,
        byte_limit: u64,
        selection: &dyn RecoveryWalReadSelection,
        storage: &mut S,
    ) -> Result<Outcome<Vec<ObservedWalArtifact>>, RecoveryDiscoveryAllocationFailure<S::Denial>>
    {
        self.read_wal_inventory(maximum_segments, byte_limit, storage, Some(selection))
    }

    fn read_wal_inventory<S: RecoveryWalReadStorage>(
        &mut self,
        maximum_segments: NonZeroU64,
        byte_limit: u64,
        storage: &mut S,
        selection: Option<&dyn RecoveryWalReadSelection>,
    ) -> Result<Outcome<Vec<ObservedWalArtifact>>, RecoveryDiscoveryAllocationFailure<S::Denial>>
    {
        // A ceiling past what this platform can list admits all it can list.
        let listed = usize::try_from(maximum_segments.get()).unwrap_or(usize::MAX);
        let directory_context = RecoveryDiscoveryArtifact::WalDirectory;
        let native = storage.listing_allocation_mode()
            == RecoveryWalListingAllocationMode::AdmissionCallbacks;
        if native && !self.wal_path_storage_is_qualified() {
            return Err(RecoveryDiscoveryFailure::invalid(directory_context).into());
        }
        let directory = backed_directory(&ArtifactTreeDirectory::families(), "wal", storage)
            .map_err(|failure| map_allocated_failure(failure, directory_context.clone(), 0))?;
        let exists = if native {
            self.parts
                .artifact_tree()
                .directory_exists_with_allocator(directory.get(), storage)
                .map_err(|failure| map_allocated_failure(failure, directory_context.clone(), 0))?
        } else {
            self.parts
                .artifact_tree()
                .directory_exists(directory.get())
                .map_err(|failure| RecoveryDiscoveryFailure::Media {
                    artifact: directory_context.clone(),
                    failure,
                })?
        };
        if !exists {
            return Ok(
                if selection.is_some_and(|selected| !selected.matches_listing(&mut [])) {
                    Outcome::Mismatch(RecoveryWalSelectionMismatch::Listing)
                } else {
                    Outcome::Observed(Vec::new())
                },
            );
        }
        let mut entries = match storage.listing_allocation_mode() {
            RecoveryWalListingAllocationMode::CallerManaged => self
                .parts
                .artifact_tree()
                .list_bounded(directory.get(), listed)
                .map_err(|failure| listing_refused(failure, maximum_segments))?,
            RecoveryWalListingAllocationMode::AdmissionCallbacks => self
                .parts
                .artifact_tree()
                .list_bounded_with_allocator(directory.get(), listed, storage)
                .map_err(|failure| map_listing_failure(failure, maximum_segments))?,
        };
        self.counters.directory_entries_observed += entries.len() as u64;
        if selection.is_some_and(|selected| !selected.matches_listing(&mut entries)) {
            return Ok(Outcome::Mismatch(RecoveryWalSelectionMismatch::Listing));
        }
        let mut observed =
            allocate_wal_roster(entries.len(), |count| storage.allocate_wal_roster(count))?;
        let mut remaining_wal_bytes = byte_limit;
        for entry in entries {
            let (name, entry_type) = entry.into_parts();
            let expected = match selection {
                Some(selected) => match selected.expected_file_length(&name) {
                    Some(length) => Some(length),
                    None => return Ok(Outcome::Mismatch(RecoveryWalSelectionMismatch::Listing)),
                },
                None => None,
            };
            let bytes = if entry_type == NamespaceEntryType::RegularFile {
                let context =
                    allocate_wal_context(&name, &mut |count| storage.allocate_context(count))?;
                let file = name.to_str().ok_or(()).and_then(|name| {
                    ArtifactTreeDirectory::validate_file_component(name)
                        .map(|()| name)
                        .map_err(|_| ())
                });
                let file_name = match file {
                    Ok(name) => name,
                    Err(()) => return Err(RecoveryDiscoveryFailure::invalid(context).into()),
                };
                let file = match backed_file(directory.get(), file_name, storage) {
                    Ok(file) => file,
                    Err(failure) => return Err(map_allocated_failure(failure, context, 0)),
                };
                // Context is created once; successful reads dispose it before
                // the next callback. The file address drops before metadata.
                let bytes = match self.read_wal_artifact_with_storage(
                    file,
                    context,
                    remaining_wal_bytes,
                    byte_limit,
                    native,
                    expected,
                    storage,
                )? {
                    Outcome::Observed(bytes) => bytes,
                    Outcome::Mismatch(mismatch) => return Ok(Outcome::Mismatch(mismatch)),
                };
                // The read was left these bytes and returned no more of them.
                remaining_wal_bytes = remaining_wal_bytes.saturating_sub(byte_count(&bytes));
                self.count_wal_bytes(byte_count(&bytes))?;
                bytes
            } else {
                None
            };
            observed.push(ObservedWalArtifact {
                store: self.parts.store_identity(),
                observation: self.issue_wal_observation_identity()?,
                name,
                entry_type,
                bytes,
            });
        }
        Ok(Outcome::Observed(observed))
    }

    fn issue_wal_observation_identity(
        &mut self,
    ) -> Result<RecoveryWalObservationIdentity, RecoveryDiscoveryFailure> {
        self.count_wal_observation()?;
        Ok(RecoveryWalObservationIdentity {
            media_generation: self.parts.media_generation(),
            discovery_incarnation: self.discovery_incarnation,
            sequence: self.wal_observations_issued,
        })
    }
}

impl<M> FilesystemObservation<M> {
    /// Counts the WAL bytes one read returned, for the caller.
    pub(super) fn count_wal_bytes(&mut self, bytes: u64) -> Result<(), RecoveryDiscoveryFailure> {
        self.counters.wal_bytes_read = self.counters.wal_bytes_read.checked_add(bytes).ok_or(
            RecoveryDiscoveryFailure::CountOverflow(RecoveryDiscoveryCount::WalBytesRead),
        )?;
        Ok(())
    }

    /// Counts one more issued WAL observation.
    pub(super) fn count_wal_observation(&mut self) -> Result<(), RecoveryDiscoveryFailure> {
        self.wal_observations_issued = self.wal_observations_issued.checked_add(1).ok_or(
            RecoveryDiscoveryFailure::CountOverflow(RecoveryDiscoveryCount::WalObservations),
        )?;
        Ok(())
    }
}

/// A listing the tree refused past `maximum_segments` is this inventory's
/// entries limit: its own allowance refuses the count the tree reported.
/// Every other failure, a refusal with no count past that ceiling among
/// them, stays the tree's.
fn listing_refused(
    failure: ArtifactTreeFailure,
    maximum_segments: NonZeroU64,
) -> RecoveryDiscoveryFailure {
    let refused = failure.access_limit().and_then(|limit| {
        FilesystemObservationAllowance::entries(maximum_segments.get())
            .admit(limit.observed)
            .err()
    });
    refused.map_or(
        RecoveryDiscoveryFailure::Media {
            artifact: RecoveryDiscoveryArtifact::WalDirectory,
            failure,
        },
        RecoveryDiscoveryFailure::Limit,
    )
}

fn byte_count(bytes: &Option<Vec<u8>>) -> u64 {
    bytes.as_ref().map_or(0, |bytes| bytes.len() as u64)
}

fn map_listing_failure<E>(
    failure: ArtifactTreeAllocatedListingFailure<E>,
    maximum_segments: NonZeroU64,
) -> RecoveryDiscoveryAllocationFailure<E> {
    let artifact = RecoveryDiscoveryArtifact::WalDirectory;
    match failure {
        ArtifactTreeAllocatedListingFailure::Media(failure) => {
            listing_refused(failure, maximum_segments).into()
        }
        ArtifactTreeAllocatedListingFailure::Allocation { requested, cause } => {
            match usize::try_from(requested) {
                Ok(requested) => RecoveryDiscoveryAllocationFailure::Allocation {
                    artifact,
                    offset: 0,
                    requested,
                    cause,
                },
                Err(_) => RecoveryDiscoveryFailure::invalid(artifact).into(),
            }
        }
        ArtifactTreeAllocatedListingFailure::BufferLengthMismatch {
            requested,
            observed,
        } => RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
            artifact,
            offset: 0,
            requested,
            observed,
        },
    }
}

#[cfg(all(test, feature = "recovery-runtime-owner"))]
mod tests {
    use super::*;

    /// A listing the tree failed without a count past the segment ceiling
    /// is the tree's failure on the WAL directory, on either listing path:
    /// no allowance of this inventory refused anything.
    #[test]
    fn a_listing_failure_without_a_count_is_the_trees() {
        for failure in [
            ArtifactTreeFailure::recovery_damaged(),
            ArtifactTreeFailure::recovery_denial(),
        ] {
            let tree = RecoveryDiscoveryFailure::Media {
                artifact: RecoveryDiscoveryArtifact::WalDirectory,
                failure,
            };
            assert_eq!(listing_refused(failure, NonZeroU64::MIN), tree);
            assert_eq!(
                map_listing_failure::<Infallible>(
                    ArtifactTreeAllocatedListingFailure::Media(failure),
                    NonZeroU64::MIN,
                ),
                RecoveryDiscoveryAllocationFailure::Discovery(tree),
            );
        }
    }
}
