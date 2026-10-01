//! A tag-7 starting point at the checkpoint source, which may precede the
//! final selected root after ordinary and released WAL publications.

use std::sync::Arc;

use worth_store_physical_format::{
    BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1, DropSetManifestV3,
    DurablePhysicalRootManifest, OriginalDropReservedV1, PersistedRecordIdentity,
    ReleaseCheckpointAccumulatorV1, ReleaseCheckpointBatchV1, ReleasedGenerationReclaimBasisV1,
};
use worth_store_physical_integrity::VerifiedCheckpointStream;

use super::WitnessedSelectedControlFrame;

#[path = "addressed_base/admission.rs"]
mod admission;
#[path = "addressed_base/checks.rs"]
mod checks;
#[path = "addressed_base/prior.rs"]
mod prior;

/// C.8's private, checkpoint-source-addressed Batch/Accumulator starting
/// point. Store must independently read this root and all three tip frames.
#[derive(Debug)]
pub struct VerifiedAddressedCheckpointReleaseBase {
    checkpoint: Arc<VerifiedCheckpointStream>,
    checkpoint_root: DurablePhysicalRootManifest,
    checkpoint_root_frame_sha256: [u8; 32],
    checkpoint_free_space_frame_sha256: [u8; 32],
    batches: Box<[ReleaseCheckpointBatchV1]>,
    prior_controls: Box<[AddressedCheckpointBatchControl]>,
    lineage: Box<[AddressedReleaseLineage]>,
    accumulator: ReleaseCheckpointAccumulatorV1,
    tip_descriptor_frame: WitnessedSelectedControlFrame,
    tip_reservation_frame: WitnessedSelectedControlFrame,
    tip_manifest_frame: WitnessedSelectedControlFrame,
    tip_descriptor: BlobReclaimDescriptorV3,
    tip_reservation: OriginalDropReservedV1,
    tip_manifest: DropSetManifestV3,
    retained_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AddressedReleaseLineage {
    pub(super) record: PersistedRecordIdentity,
    pub(super) sha256: [u8; 32],
    pub(super) descriptor: BlobReclaimDescriptorV3,
    pub(super) source: ReleasedGenerationReclaimBasisV1,
}

/// Checkpoint-source routed frames for a non-tip Batch. The admission below
/// binds each to its exact tag-7 record before any predecessor use.
#[derive(Debug)]
pub struct AddressedCheckpointBatchControl {
    descriptor: WitnessedSelectedControlFrame,
    reservation: WitnessedSelectedControlFrame,
    manifest: WitnessedSelectedControlFrame,
}

impl AddressedCheckpointBatchControl {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.descriptor
            .owned_heap_bytes()?
            .checked_add(self.reservation.owned_heap_bytes()?)?
            .checked_add(self.manifest.owned_heap_bytes()?)
    }

    pub fn new(
        descriptor: WitnessedSelectedControlFrame,
        reservation: WitnessedSelectedControlFrame,
        manifest: WitnessedSelectedControlFrame,
    ) -> Self {
        Self {
            descriptor,
            reservation,
            manifest,
        }
    }

    pub const fn descriptor(&self) -> &WitnessedSelectedControlFrame {
        &self.descriptor
    }
    pub const fn reservation(&self) -> &WitnessedSelectedControlFrame {
        &self.reservation
    }
    pub const fn manifest(&self) -> &WitnessedSelectedControlFrame {
        &self.manifest
    }
}

impl VerifiedAddressedCheckpointReleaseBase {
    /// Excludes shared checkpoint backing and this value's inline storage.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let batches = u64::try_from(self.batches.len())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<ReleaseCheckpointBatchV1>()).ok()?)?;
        let controls = u64::try_from(self.prior_controls.len()).ok()?.checked_mul(
            u64::try_from(std::mem::size_of::<AddressedCheckpointBatchControl>()).ok()?,
        )?;
        let lineage = u64::try_from(self.lineage.len())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<AddressedReleaseLineage>()).ok()?)?;
        self.prior_controls
            .iter()
            .try_fold(
                batches.checked_add(controls)?.checked_add(lineage)?,
                |sum, control| sum.checked_add(control.owned_heap_bytes()?),
            )?
            .checked_add(self.tip_descriptor_frame.owned_heap_bytes()?)?
            .checked_add(self.tip_reservation_frame.owned_heap_bytes()?)?
            .checked_add(self.tip_manifest_frame.owned_heap_bytes()?)?
            .checked_add(
                u64::try_from(self.tip_manifest.dropped().len())
                    .ok()?
                    .checked_mul(
                        u64::try_from(std::mem::size_of::<PersistedRecordIdentity>()).ok()?,
                    )?,
            )
    }

    pub fn checkpoint(&self) -> &VerifiedCheckpointStream {
        &self.checkpoint
    }
    pub const fn checkpoint_root(&self) -> &DurablePhysicalRootManifest {
        &self.checkpoint_root
    }
    pub const fn checkpoint_root_frame_sha256(&self) -> [u8; 32] {
        self.checkpoint_root_frame_sha256
    }
    pub const fn checkpoint_free_space_frame_sha256(&self) -> [u8; 32] {
        self.checkpoint_free_space_frame_sha256
    }
    pub fn batches(&self) -> &[ReleaseCheckpointBatchV1] {
        &self.batches
    }
    pub fn prior_controls(&self) -> &[AddressedCheckpointBatchControl] {
        &self.prior_controls
    }
    pub fn matches_predecessor(
        &self,
        descriptor: BlobReclaimDescriptorV3,
        manifest: &DropSetManifestV3,
    ) -> bool {
        let Some(predecessor) = descriptor.base().predecessor() else {
            return false;
        };
        let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
            return false;
        };
        self.lineage
            .iter()
            .rev()
            .find(|prior| {
                prior.source.object() == source.object()
                    && prior.source.generation() == source.generation()
            })
            .is_some_and(|prior| {
                predecessor.descriptor_record() == prior.record
                    && predecessor.descriptor_frame_sha256() == prior.sha256
                    && !prior.descriptor.base().terminal()
                    && prior.descriptor.base().source_basis_digest()
                        == descriptor.base().source_basis_digest()
                    && prior
                        .descriptor
                        .base()
                        .cumulative_dropped()
                        .checked_add(u64::from(manifest.count()))
                        == Some(descriptor.base().cumulative_dropped())
            })
    }
    pub const fn accumulator(&self) -> ReleaseCheckpointAccumulatorV1 {
        self.accumulator
    }
    pub const fn tip_descriptor_frame(&self) -> &WitnessedSelectedControlFrame {
        &self.tip_descriptor_frame
    }
    pub const fn tip_reservation_frame(&self) -> &WitnessedSelectedControlFrame {
        &self.tip_reservation_frame
    }
    pub const fn tip_manifest_frame(&self) -> &WitnessedSelectedControlFrame {
        &self.tip_manifest_frame
    }
    pub const fn tip_descriptor(&self) -> BlobReclaimDescriptorV3 {
        self.tip_descriptor
    }
    pub const fn tip_reservation(&self) -> OriginalDropReservedV1 {
        self.tip_reservation
    }
    pub const fn tip_manifest(&self) -> &DropSetManifestV3 {
        &self.tip_manifest
    }
    pub const fn tip_descriptor_record(&self) -> PersistedRecordIdentity {
        self.accumulator.tip().descriptor_record()
    }
    pub const fn tip_descriptor_frame_sha256(&self) -> [u8; 32] {
        self.accumulator.tip().descriptor_frame_sha256()
    }
    pub const fn tip_reservation_record(&self) -> PersistedRecordIdentity {
        self.accumulator.tip().reservation_record()
    }
    pub const fn tip_reservation_frame_sha256(&self) -> [u8; 32] {
        self.accumulator.tip().reservation_frame_sha256()
    }
    pub const fn tip_manifest_record(&self) -> PersistedRecordIdentity {
        self.tip_descriptor.base().manifest_record()
    }
    pub const fn tip_manifest_frame_sha256(&self) -> [u8; 32] {
        self.tip_descriptor.base().manifest_frame_sha256()
    }
    pub const fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }
}
