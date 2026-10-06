//! C4 typed-record reads with admitted address, cap-open and payload storage.

use worth_foundational::LimitDimension;
use worth_store_physical_format::RecordArtifactFile;

use super::super::ceiling::{ArtifactCeiling, CeilingExtent};
use super::super::grant::ReadGrant;
use super::super::refusal::{AllocatedReadOutcome, ArtifactDamage};
use super::artifact::{backed_checkpoint_artifact, backed_record_artifact};
use super::charged_read::{outcome, ReadStop};
use super::resident_read::{allocated, map_allocated_failure};
use super::{
    DiscoveryMediaBacking, FilesystemObservation, ObservedRecoveryArtifact,
    RecoveryDiscoveryAllocationFailure, RecoveryDiscoveryArtifact, RecoveryDiscoveryFailure,
};
use crate::filesystem_media::ArtifactTreeReadAllocator;

impl<M: DiscoveryMediaBacking> FilesystemObservation<M> {
    /// Reads the artifact `ceiling` names through the same backed
    /// address/open/read engine as WAL; no raw-path fallback. Storage admission
    /// occurs before each address/open scratch allocation, and payload
    /// allocation occurs only after the actual file length is known.
    pub fn read_with_storage<D: LimitDimension, S: ArtifactTreeReadAllocator>(
        &mut self,
        ceiling: ArtifactCeiling,
        grant: ReadGrant<D>,
        storage: &mut S,
    ) -> AllocatedReadOutcome<D, S::Denial> {
        let address = ceiling.file();
        let context = RecoveryDiscoveryArtifact::Record(address);
        if !self.parts.artifact_tree().path_storage_is_qualified() {
            return unqualified(context);
        }
        let caller = context.clone();
        outcome(match ceiling.extent() {
            CeilingExtent::Whole { bytes, fixed } => {
                self.read_whole_charged(context, bytes, fixed, &grant, |attempt, limit| {
                    let artifact = backed_record_artifact(address, storage)
                        .map_err(|failure| allocated(failure, &caller, 0))?;
                    attempt
                        .open()
                        .read_backed_bounded_with_allocator(artifact, limit, storage)
                        .map_err(|failure| allocated(failure, &caller, 0))
                })
            }
            CeilingExtent::Frame { offset, length } => {
                self.read_range_charged(context, offset, length, &grant, |attempt, capacity| {
                    let artifact = backed_record_artifact(address, storage)
                        .map_err(|failure| allocated(failure, &caller, offset))?;
                    attempt
                        .open()
                        .read_backed_exact_at_with_allocator(artifact, offset, capacity, storage)
                        .map_err(|failure| allocated(failure, &caller, offset))
                })
            }
        })
    }

    pub fn read_current_checkpoint_with_storage<S: ArtifactTreeReadAllocator>(
        &mut self,
        byte_limit: u64,
        storage: &mut S,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryAllocationFailure<S::Denial>> {
        let context = RecoveryDiscoveryArtifact::CurrentCheckpoint;
        self.admit_read()?;
        if !self.parts.artifact_tree().path_storage_is_qualified() {
            return Err(RecoveryDiscoveryFailure::invalid(context).into());
        }
        let artifact = backed_checkpoint_artifact(storage)
            .map_err(|failure| map_allocated_failure(failure, context.clone(), 0))?;
        self.read_whole_with(context, byte_limit, false, |tree, limit| {
            tree.read_backed_bounded_with_allocator(artifact, limit, storage)
        })
    }

    /// Reads exactly `length` bytes at `offset` of `address`, a range a
    /// verified parent declared, through the same opened-file range reader.
    pub fn read_record_artifact_range_with_storage<
        D: LimitDimension,
        S: ArtifactTreeReadAllocator,
    >(
        &mut self,
        address: RecordArtifactFile,
        offset: u64,
        length: u32,
        grant: ReadGrant<D>,
        storage: &mut S,
    ) -> AllocatedReadOutcome<D, S::Denial> {
        let context = RecoveryDiscoveryArtifact::Record(address);
        if !self.parts.artifact_tree().path_storage_is_qualified() {
            return unqualified(context);
        }
        let caller = context.clone();
        outcome(
            self.read_range_charged(context, offset, length, &grant, |attempt, capacity| {
                let artifact = backed_record_artifact(address, storage)
                    .map_err(|failure| allocated(failure, &caller, offset))?;
                attempt
                    .open()
                    .read_backed_exact_at_with_allocator(artifact, offset, capacity, storage)
                    .map_err(|failure| allocated(failure, &caller, offset))
            }),
        )
    }
}

/// A read through path storage the media never qualified addresses nothing.
fn unqualified<D: LimitDimension, E>(
    artifact: RecoveryDiscoveryArtifact,
) -> AllocatedReadOutcome<D, E> {
    outcome(Err(ReadStop::damage(ArtifactDamage::InvalidAddress {
        artifact,
    })))
}
