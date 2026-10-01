//! An earlier completed V3 batch, checked against selected controls, its
//! semantics-admitted C.9 member, and a checkpoint-anchored root chain.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV3, BlobRecordKind, BlobRecordV1, DropSetManifestV3,
    OriginalDropReservedV1, PersistedPhysicalRecoveryOperation, PersistedRecordIdentity,
    ReleasedDropWalFateWitnessV1,
};

use super::{
    verification::{verify_controls, verify_fate},
    PendingReleaseCheckpointBase, PendingWalReleaseCustodyDenial as Denial,
    VerifiedPendingWalReleaseCustody,
};
use crate::{
    ImmutablePhysicalRedoPlan, PhysicalRedoGroupBinding, PhysicalSourceSelection,
    ReconciledOperationFates, RecoveryOperationFate, VerifiedHistoricalReleaseRootChain,
    WitnessedSelectedControlFrame,
};

#[derive(Debug)]
pub struct VerifiedHistoricalPendingWalBatch {
    chain: VerifiedHistoricalReleaseRootChain,
    descriptor_record: PersistedRecordIdentity,
    descriptor_frame_sha256: [u8; 32],
    descriptor: BlobReclaimDescriptorV3,
    reservation_record: PersistedRecordIdentity,
    reservation_frame_sha256: [u8; 32],
    reservation: OriginalDropReservedV1,
    manifest_record: PersistedRecordIdentity,
    manifest_frame_sha256: [u8; 32],
    manifest: DropSetManifestV3,
    wal_fate: ReleasedDropWalFateWitnessV1,
    member_group: PhysicalRedoGroupBinding,
    member_redo_digest: [u8; 32],
    operation_fate: RecoveryOperationFate,
}

impl VerifiedHistoricalPendingWalBatch {
    /// Outer batch storage belongs to the containing roster.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.chain.owned_heap_bytes()?.checked_add(
            u64::try_from(self.manifest.dropped().len())
                .ok()?
                .checked_mul(u64::try_from(std::mem::size_of::<PersistedRecordIdentity>()).ok()?)?,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn admit(
        selected: &PhysicalSourceSelection,
        chain: VerifiedHistoricalReleaseRootChain,
        descriptor_frame: &WitnessedSelectedControlFrame,
        reservation_frame: &WitnessedSelectedControlFrame,
        manifest_frame: &WitnessedSelectedControlFrame,
        wal_fate: ReleasedDropWalFateWitnessV1,
        redo: &ImmutablePhysicalRedoPlan,
        fates: &ReconciledOperationFates,
        policy: [u8; 32],
    ) -> Result<Self, Denial> {
        let descriptor_bytes = descriptor_frame
            .selected(selected, BlobRecordKind::ReclaimDescriptorV3)
            .map_err(|_| Denial::ControlBinding)?;
        let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
            decode_blob_record(descriptor_bytes).map_err(|_| Denial::ControlBinding)?
        else {
            return Err(Denial::ControlBinding);
        };
        let reservation_bytes = reservation_frame
            .selected(selected, BlobRecordKind::OriginalDropReserved)
            .map_err(|_| Denial::ControlBinding)?;
        let BlobRecordV1::OriginalDropReserved(reservation) =
            decode_blob_record(reservation_bytes).map_err(|_| Denial::ControlBinding)?
        else {
            return Err(Denial::ControlBinding);
        };
        let manifest_bytes = manifest_frame
            .selected(selected, BlobRecordKind::DropSetManifestV3)
            .map_err(|_| Denial::ControlBinding)?;
        let BlobRecordV1::DropSetManifestV3(manifest) =
            decode_blob_record(manifest_bytes).map_err(|_| Denial::ControlBinding)?
        else {
            return Err(Denial::ControlBinding);
        };
        verify_controls(
            descriptor,
            reservation,
            &manifest,
            reservation_frame,
            manifest_frame,
            None,
            None,
            None,
        )?;
        let base = descriptor.base();
        let mut projections = redo
            .projections()
            .iter()
            .filter(|projection| projection.operation() == chain.descriptor_operation());
        let projection = projections.next().ok_or(Denial::DurableWalFate)?;
        let PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. } =
            projection.materialization().operation()
        else {
            return Err(Denial::DurableWalFate);
        };
        let descriptor_sha: [u8; 32] = Sha256::digest(descriptor_bytes).into();
        let first_lsn = chain.first_lsn();
        if projections.next().is_some()
            || !redo.admits_exact_member_redo_digest(projection, chain.first_redo_sha256())
            || projection.group() != chain.first_group()
            || projection.fate() != chain.first_fate()
            || projection.materialization().source_root_generation()
                != chain.source_root_generation()
            || binding.record() != descriptor_frame.selected_placement().record()
            || binding.record_payload_sha256() != descriptor_sha
            || binding.candidate_root_generation() != chain.first_result_generation()
            || base.predecessor().is_some()
            || base.source_root_generation() != chain.source_root_generation()
            || base.candidate_root_generation() != chain.first_result_generation()
            || descriptor.custody().source_root_frame_sha256() != chain.source_root_frame_sha256()
            || descriptor.custody().request().idempotency() != chain.descriptor_operation()
            || !chain
                .first_transition()
                .projected()
                .iter()
                .any(|placement| *placement == descriptor_frame.selected_placement())
            || wal_fate.lsn_start() != first_lsn.start().get()
            || wal_fate.lsn_end_exclusive() != first_lsn.end_exclusive().get()
        {
            return Err(Denial::ControlBinding);
        }
        let checkpoint = selected.checkpoint().ok_or(Denial::CheckpointMarker)?;
        let operation_fate = verify_fate(
            descriptor,
            wal_fate,
            fates,
            policy,
            chain.descriptor_operation(),
            checkpoint
                .checkpoint()
                .compaction_cutover()
                .wal_cutoff_lsn_exclusive(),
        )?;
        Ok(Self {
            chain,
            descriptor_record: binding.record(),
            descriptor_frame_sha256: descriptor_sha,
            descriptor,
            reservation_record: reservation_frame.selected_placement().record(),
            reservation_frame_sha256: reservation_frame.selected_payload_sha256(),
            reservation,
            manifest_record: manifest_frame.selected_placement().record(),
            manifest_frame_sha256: manifest_frame.selected_payload_sha256(),
            manifest,
            wal_fate,
            member_group: projection.group(),
            member_redo_digest: projection
                .semantics_admitted_redo_sha256()
                .ok_or(Denial::DurableWalFate)?,
            operation_fate,
        })
    }

    pub const fn chain(&self) -> &VerifiedHistoricalReleaseRootChain {
        &self.chain
    }
    pub const fn descriptor_record(&self) -> PersistedRecordIdentity {
        self.descriptor_record
    }
    pub const fn descriptor_frame_sha256(&self) -> [u8; 32] {
        self.descriptor_frame_sha256
    }
    pub const fn descriptor(&self) -> BlobReclaimDescriptorV3 {
        self.descriptor
    }
    pub const fn reservation_record(&self) -> PersistedRecordIdentity {
        self.reservation_record
    }
    pub const fn reservation_frame_sha256(&self) -> [u8; 32] {
        self.reservation_frame_sha256
    }
    pub const fn reservation(&self) -> OriginalDropReservedV1 {
        self.reservation
    }
    pub const fn manifest_record(&self) -> PersistedRecordIdentity {
        self.manifest_record
    }
    pub const fn manifest_frame_sha256(&self) -> [u8; 32] {
        self.manifest_frame_sha256
    }
    pub const fn manifest(&self) -> &DropSetManifestV3 {
        &self.manifest
    }
    pub const fn wal_fate(&self) -> ReleasedDropWalFateWitnessV1 {
        self.wal_fate
    }
    pub const fn member_group(&self) -> PhysicalRedoGroupBinding {
        self.member_group
    }
    pub const fn member_redo_digest(&self) -> [u8; 32] {
        self.member_redo_digest
    }
    pub const fn operation_fate(&self) -> RecoveryOperationFate {
        self.operation_fate
    }
}

impl VerifiedPendingWalReleaseCustody {
    /// A selected first V3 result under NoRelease is not a checkpoint batch.
    /// It may be attached only after C8 has checked the complete checkpoint
    /// prefix, the first released transition, and every ordinary successor to
    /// this pending batch's exact selected source root.
    #[allow(clippy::too_many_arguments)]
    pub fn attach_historical_first_batch(
        &mut self,
        selected: &PhysicalSourceSelection,
        chain: VerifiedHistoricalReleaseRootChain,
        descriptor_frame: &WitnessedSelectedControlFrame,
        reservation_frame: &WitnessedSelectedControlFrame,
        manifest_frame: &WitnessedSelectedControlFrame,
        wal_fate: ReleasedDropWalFateWitnessV1,
        redo: &ImmutablePhysicalRedoPlan,
        fates: &ReconciledOperationFates,
        policy: [u8; 32],
    ) -> Result<(), Denial> {
        if !matches!(self.base, PendingReleaseCheckpointBase::NoRelease(_))
            || !self.historical_batches.is_empty()
            || self.source_root != *selected.root().selected().manifest()
            || self.checkpoint_source_root_sha256
                != chain
                    .checkpoint_root_frame_sha256()
                    .ok_or(Denial::CheckpointMarker)?
            || self.source_root_sha256 != chain.selected_root_frame_sha256()
            || self.descriptor.custody().source_free_space_frame_sha256()
                != chain.selected_free_frame_sha256()
            || chain.first_result_generation() >= self.source_root.generation()
            || chain.first_lsn().end_exclusive().get() >= self.wal_fate.lsn_start()
            || chain.descriptor_operation() == self.descriptor.custody().request().idempotency()
        {
            return Err(Denial::SourceBinding);
        }
        let batch = VerifiedHistoricalPendingWalBatch::admit(
            selected,
            chain,
            descriptor_frame,
            reservation_frame,
            manifest_frame,
            wal_fate,
            redo,
            fates,
            policy,
        )?;
        self.historical_batches = vec![batch].into_boxed_slice();
        Ok(())
    }
}
