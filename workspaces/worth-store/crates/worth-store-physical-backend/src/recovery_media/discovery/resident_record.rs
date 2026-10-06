//! C4 typed-record reads with admitted address, cap-open and payload storage.

use worth_foundational::LimitDimension;
use worth_store_physical_format::RecordArtifactFile;

use super::super::ceiling::{ArtifactCeiling, CeilingExtent};
use super::super::grant::{GrantShare, ReadGrant};
use super::super::refusal::{AllocatedReadOutcome, ArtifactDamage};
use super::artifact::{backed_ceiling_artifact, backed_record_artifact};
use super::charged_read::{outcome, ReadStop};
use super::resident_read::allocated;
use super::{DiscoveryMediaBacking, FilesystemObservation, RecoveryDiscoveryArtifact};
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
        let address = ceiling.address();
        let context = ceiling.artifact();
        if !self.parts.artifact_tree().path_storage_is_qualified() {
            return unqualified(context);
        }
        let caller = context.clone();
        outcome(match ceiling.extent() {
            CeilingExtent::Whole { bytes, fixed } => {
                let grant = GrantShare::of(&grant);
                self.read_whole_charged(context, bytes, fixed, &grant, |attempt, limit| {
                    let artifact = backed_ceiling_artifact(address, storage)
                        .map_err(|failure| allocated(failure, &caller, 0))?;
                    attempt
                        .open()
                        .read_backed_bounded_with_allocator(artifact, limit, storage)
                        .map_err(|failure| allocated(failure, &caller, 0))
                })
            }
            CeilingExtent::Frame { offset, length } => {
                self.read_range_charged(context, offset, length, &grant, |attempt, capacity| {
                    let artifact = backed_ceiling_artifact(address, storage)
                        .map_err(|failure| allocated(failure, &caller, offset))?;
                    attempt
                        .open()
                        .read_backed_exact_at_with_allocator(artifact, offset, capacity, storage)
                        .map_err(|failure| allocated(failure, &caller, offset))
                })
            }
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
