//! C4 typed-record reads with admitted address, cap-open and payload storage.

use super::artifact::{backed_checkpoint_artifact, backed_record_artifact};
use super::resident_read::map_allocated_failure;
use super::{
    DiscoveryMediaBacking, FilesystemObservation, ObservedRecoveryArtifact,
    RecoveryDiscoveryAllocationFailure, RecoveryDiscoveryArtifact, RecoveryDiscoveryFailure,
};
use crate::filesystem_media::{ArtifactTreeFailureKind, ArtifactTreeReadAllocator};
use worth_store_physical_format::RecordArtifactFile;

impl<M: DiscoveryMediaBacking> FilesystemObservation<M> {
    /// The same backed address/open/read engine as WAL; no raw-path fallback.
    /// Storage admission occurs before each address/open scratch allocation,
    /// and payload allocation occurs only after the actual file length is known.
    pub fn read_record_artifact_with_storage<S: ArtifactTreeReadAllocator>(
        &mut self,
        address: RecordArtifactFile,
        byte_limit: u64,
        storage: &mut S,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<S::Denial>> {
        let context = RecoveryDiscoveryArtifact::Record(address);
        self.admit_backed_record_read(&context)?;
        let artifact = backed_record_artifact(address, storage)
            .map_err(|failure| map_allocated_failure(failure, context.clone(), 0))?;
        let fixed = matches!(
            address,
            RecordArtifactFile::BootstrapCatalog
                | RecordArtifactFile::CurrentRootSelector
                | RecordArtifactFile::PreviousRootSelector
        );
        let observed = self.read_whole_with(context, byte_limit, fixed, |tree, limit| {
            tree.read_backed_bounded_with_allocator(artifact, limit, storage)
        })?;
        if fixed {
            self.counters.fixed_slots_read += 1;
        }
        Ok(observed)
    }

    pub fn read_current_checkpoint_with_storage<S: ArtifactTreeReadAllocator>(
        &mut self,
        byte_limit: u64,
        storage: &mut S,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<S::Denial>> {
        let context = RecoveryDiscoveryArtifact::CurrentCheckpoint;
        self.admit_backed_record_read(&context)?;
        let artifact = backed_checkpoint_artifact(storage)
            .map_err(|failure| map_allocated_failure(failure, context.clone(), 0))?;
        self.read_whole_with(context, byte_limit, false, |tree, limit| {
            tree.read_backed_bounded_with_allocator(artifact, limit, storage)
        })
    }

    /// Exact selected extent ranges use the same opened-file range reader.
    pub fn read_record_artifact_range_with_storage<S: ArtifactTreeReadAllocator>(
        &mut self,
        address: RecordArtifactFile,
        offset: u64,
        length: u32,
        byte_limit: u64,
        storage: &mut S,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<S::Denial>> {
        let context = RecoveryDiscoveryArtifact::Record(address);
        let length = u64::from(length);
        offset
            .checked_add(length)
            .ok_or_else(|| RecoveryDiscoveryFailure::invalid(context.clone()))?;
        self.admit_range(&context, length, byte_limit)?;
        self.admit_backed_record_read(&context)?;
        let capacity = usize::try_from(length)
            .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?;
        let artifact = backed_record_artifact(address, storage)
            .map_err(|failure| map_allocated_failure(failure, context.clone(), offset))?;
        self.remaining_entries -= 1;
        match self
            .parts
            .artifact_tree()
            .read_backed_exact_at_with_allocator(artifact, offset, capacity, storage)
        {
            Ok(bytes) => {
                self.remaining_bytes -= length;
                self.counters.bytes_read += length;
                self.counters.addressed_artifacts_read += 1;
                Ok(ObservedRecoveryArtifact::new(
                    self.parts.store_identity(),
                    context,
                    offset,
                    Some(bytes),
                ))
            }
            Err(crate::filesystem_media::ArtifactTreeAllocatedReadFailure::Media(failure))
                if failure.kind() == ArtifactTreeFailureKind::Absent =>
            {
                Ok(ObservedRecoveryArtifact::new(
                    self.parts.store_identity(),
                    context,
                    offset,
                    None,
                ))
            }
            Err(failure) => Err(map_allocated_failure(failure, context, offset)),
        }
    }

    fn admit_backed_record_read(
        &self,
        context: &RecoveryDiscoveryArtifact,
    ) -> Result<(), RecoveryDiscoveryFailure> {
        if self.remaining_entries == 0 {
            return Err(RecoveryDiscoveryFailure::EntryLimitExceeded {
                observed: 1,
                admitted: 0,
            });
        }
        if !self.parts.artifact_tree().path_storage_is_qualified() {
            return Err(RecoveryDiscoveryFailure::invalid(context.clone()));
        }
        Ok(())
    }
}
