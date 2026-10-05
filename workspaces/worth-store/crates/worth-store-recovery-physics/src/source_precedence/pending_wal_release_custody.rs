//! C.8's post-checkpoint WAL-release claim. A selected NoRelease marker or
//! Batch/Accumulator base remains checkpoint-source truth, not a certificate
//! for the new published drop; TierEpoch is separately authenticated.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV3, BlobReclaimSourceKind, BlobRecordKind,
    BlobRecordV1, DurablePhysicalRootManifest, PersistedPhysicalRecoveryOperation,
    PersistedRecordIdentity, PhysicalInventoryTranscriptV1, PhysicalRecordFormatDeclaration,
    ReleaseCheckpointNoReleaseV1, ReleasedDropWalFateWitnessV1,
};
use worth_store_physical_integrity::{VerifiedCheckpointFacts, VerifiedCheckpointStream};

use super::no_release_custody::selected_checkpoint_marker;
use super::release_custody::{
    VerifiedAddressedCheckpointReleaseBase, VerifiedSelectedCheckpointCustody,
    VerifiedSelectedReleaseHeadCustodyV2, WitnessedSelectedControlFrame,
};
use super::released_v3_inventory_transition::VerifiedReleasedV3InventoryTransition;
use super::tier_custody::VerifiedSelectedTierEpochCustody;
use super::{ExceededPhysicsBound, PhysicalSourceSelection};
use crate::{
    ImmutablePhysicalRedoPlan, PhysicalRedoGroupBinding, PhysicalRedoProjection,
    ReconciledOperationFates, RecoveryOperationFate, VerifiedSelectedReleaseHeadReplayV14,
};

#[path = "pending_wal_release_custody/historical_batch.rs"]
mod historical_batch;
#[path = "pending_wal_release_custody/topology_rebind.rs"]
mod topology_rebind;
pub use historical_batch::VerifiedHistoricalPendingWalBatch;
#[path = "pending_wal_release_custody/ordered_batch.rs"]
mod ordered_batch;
pub use ordered_batch::VerifiedOrderedPendingWalReleaseBatch;
#[path = "pending_wal_release_custody/ordered_attach.rs"]
mod ordered_attach;
#[path = "pending_wal_release_custody/verification.rs"]
mod verification;
use verification::{verify_controls, verify_fate};
#[path = "pending_wal_release_custody/admission.rs"]
mod admission;
#[path = "pending_wal_release_custody/effective_heads.rs"]
mod effective_heads;
#[path = "pending_wal_release_custody/head_v2.rs"]
mod head_v2;
#[path = "pending_wal_release_custody/retained_storage.rs"]
mod retained_storage;
use effective_heads::PreparedEffectiveHeadRosterV14;
pub use effective_heads::{EffectiveReleaseHeadDenial, VerifiedEffectiveReleaseHeadRosterV14};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingWalReleaseCustodyDenial {
    CheckpointMarker,
    SourceBinding,
    ControlBinding,
    DurableWalFate,
    PublishedRoot,
    RetainedSizeOverflow,
    Limit(ExceededPhysicsBound),
}

#[derive(Debug)]
enum PendingReleaseCheckpointBase {
    NoRelease(ReleaseCheckpointNoReleaseV1),
    Released(Box<VerifiedSelectedCheckpointCustody>),
    ReleasedAddressed(Box<VerifiedAddressedCheckpointReleaseBase>),
    ReleasedHeadV2(Box<VerifiedSelectedReleaseHeadCustodyV2>),
}

/// Private-field C.8 transcript. Store must independently reread the exact
/// selected checkpoint, C.9 member, controls, and final root before sealing.
#[derive(Debug)]
pub struct VerifiedPendingWalReleaseCustody {
    checkpoint: VerifiedCheckpointFacts,
    base: PendingReleaseCheckpointBase,
    checkpoint_source_root_sha256: [u8; 32],
    source_root: DurablePhysicalRootManifest,
    source_root_sha256: [u8; 32],
    published_root: Option<DurablePhysicalRootManifest>,
    published_root_sha256: Option<[u8; 32]>,
    verified_transition: Option<VerifiedReleasedV3InventoryTransition>,
    historical_batches: Box<[VerifiedHistoricalPendingWalBatch]>,
    ordered_history: Option<Arc<crate::VerifiedOrderedRootHistory>>,
    ordered_released_batches: Box<[VerifiedOrderedPendingWalReleaseBatch]>,
    descriptor_record: PersistedRecordIdentity,
    descriptor_frame_sha256: [u8; 32],
    descriptor: BlobReclaimDescriptorV3,
    reservation_record: PersistedRecordIdentity,
    reservation_frame_sha256: [u8; 32],
    manifest_record: PersistedRecordIdentity,
    manifest_frame_sha256: [u8; 32],
    wal_fate: ReleasedDropWalFateWitnessV1,
    member_group: PhysicalRedoGroupBinding,
    member_redo_digest: [u8; 32],
    operation_fate: RecoveryOperationFate,
    selected_head_replay: Option<VerifiedSelectedReleaseHeadReplayV14>,
    prepared_effective_heads: Option<PreparedEffectiveHeadRosterV14>,
}

impl VerifiedPendingWalReleaseCustody {
    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        selected: &PhysicalSourceSelection,
        stream: &VerifiedCheckpointStream,
        tier: Option<&VerifiedSelectedTierEpochCustody>,
        projection: &PhysicalRedoProjection,
        redo: &ImmutablePhysicalRedoPlan,
        reservation_frame: &WitnessedSelectedControlFrame,
        manifest_frame: &WitnessedSelectedControlFrame,
        wal_fate: ReleasedDropWalFateWitnessV1,
        member_redo_digest: [u8; 32],
        fates: &ReconciledOperationFates,
        policy: [u8; 32],
    ) -> Result<Self, PendingWalReleaseCustodyDenial> {
        let marker = selected_checkpoint_marker(selected, stream)
            .map_err(|_| PendingWalReleaseCustodyDenial::CheckpointMarker)?;
        Self::admit_with_base(
            selected,
            PendingReleaseCheckpointBase::NoRelease(marker),
            tier,
            projection,
            redo,
            reservation_frame,
            manifest_frame,
            wal_fate,
            member_redo_digest,
            fates,
            policy,
            None,
        )
    }

    /// The selected Batch/Accumulator is a certified Store-wide starting
    /// point. A V3 predecessor, if any, remains scoped to its released object;
    /// C.8 separately verifies that object's selected chain and source holes.
    #[allow(clippy::too_many_arguments)]
    pub fn admit_from_selected_release(
        selected: &PhysicalSourceSelection,
        selected_release: VerifiedSelectedCheckpointCustody,
        tier: Option<&VerifiedSelectedTierEpochCustody>,
        projection: &PhysicalRedoProjection,
        redo: &ImmutablePhysicalRedoPlan,
        reservation_frame: &WitnessedSelectedControlFrame,
        manifest_frame: &WitnessedSelectedControlFrame,
        wal_fate: ReleasedDropWalFateWitnessV1,
        member_redo_digest: [u8; 32],
        fates: &ReconciledOperationFates,
        policy: [u8; 32],
    ) -> Result<Self, PendingWalReleaseCustodyDenial> {
        Self::admit_with_base(
            selected,
            PendingReleaseCheckpointBase::Released(Box::new(selected_release)),
            tier,
            projection,
            redo,
            reservation_frame,
            manifest_frame,
            wal_fate,
            member_redo_digest,
            fates,
            policy,
            None,
        )
    }

    /// Ordered postcheckpoint roots may retire the checkpoint tip controls
    /// from the final selected root. Their starting custody is addressed to
    /// the checkpoint source, independently of this pending V3 source root.
    #[allow(clippy::too_many_arguments)]
    pub fn admit_from_addressed_release(
        selected: &PhysicalSourceSelection,
        addressed: VerifiedAddressedCheckpointReleaseBase,
        tier: Option<&VerifiedSelectedTierEpochCustody>,
        projection: &PhysicalRedoProjection,
        redo: &ImmutablePhysicalRedoPlan,
        reservation_frame: &WitnessedSelectedControlFrame,
        manifest_frame: &WitnessedSelectedControlFrame,
        wal_fate: ReleasedDropWalFateWitnessV1,
        member_redo_digest: [u8; 32],
        fates: &ReconciledOperationFates,
        policy: [u8; 32],
    ) -> Result<Self, PendingWalReleaseCustodyDenial> {
        Self::admit_with_base(
            selected,
            PendingReleaseCheckpointBase::ReleasedAddressed(Box::new(addressed)),
            tier,
            projection,
            redo,
            reservation_frame,
            manifest_frame,
            wal_fate,
            member_redo_digest,
            fates,
            policy,
            None,
        )
    }

    pub fn rebind_published_root(
        &mut self,
        selected: &PhysicalSourceSelection,
        published: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), PendingWalReleaseCustodyDenial> {
        if self.published_root.is_some()
            || self.source_root != *selected.root().selected().manifest()
            || format != selected.root().selected().selector().format()
            || published.generation() != self.descriptor.base().candidate_root_generation()
            || published.tree_identity() != self.source_root.tree_identity()
            || published.tier_epoch_anchor() != self.source_root.tier_epoch_anchor()
            || self.selected_head_replay.as_ref().is_some_and(|replay| {
                published.release_custody_head_root() != Some(replay.result_root())
                    || published.next_release_custody_head_block() != replay.result_next_block()
            })
            || self.selected_head_replay.is_none()
                && (published.release_custody_head_root()
                    != self.source_root.release_custody_head_root()
                    || published.next_release_custody_head_block()
                        != self.source_root.next_release_custody_head_block())
        {
            return Err(PendingWalReleaseCustodyDenial::PublishedRoot);
        }
        self.published_root_sha256 = Some(Sha256::digest(published.encode(format)).into());
        self.published_root = Some(published.clone());
        Ok(())
    }

    pub fn checkpoint(&self) -> &VerifiedCheckpointFacts {
        &self.checkpoint
    }
    pub fn marker(&self) -> Option<ReleaseCheckpointNoReleaseV1> {
        match &self.base {
            PendingReleaseCheckpointBase::NoRelease(marker) => Some(*marker),
            PendingReleaseCheckpointBase::Released(_)
            | PendingReleaseCheckpointBase::ReleasedAddressed(_)
            | PendingReleaseCheckpointBase::ReleasedHeadV2(_) => None,
        }
    }
    pub fn selected_release(&self) -> Option<&VerifiedSelectedCheckpointCustody> {
        match &self.base {
            PendingReleaseCheckpointBase::NoRelease(_) => None,
            PendingReleaseCheckpointBase::Released(base) => Some(base),
            PendingReleaseCheckpointBase::ReleasedAddressed(_) => None,
            PendingReleaseCheckpointBase::ReleasedHeadV2(_) => None,
        }
    }
    pub fn addressed_release_base(&self) -> Option<&VerifiedAddressedCheckpointReleaseBase> {
        match &self.base {
            PendingReleaseCheckpointBase::ReleasedAddressed(base) => Some(base),
            _ => None,
        }
    }
    pub fn selected_head_v2(&self) -> Option<&VerifiedSelectedReleaseHeadCustodyV2> {
        match &self.base {
            PendingReleaseCheckpointBase::ReleasedHeadV2(base) => Some(base),
            _ => None,
        }
    }
    pub fn selected_head_replay(&self) -> Option<&VerifiedSelectedReleaseHeadReplayV14> {
        self.selected_head_replay.as_ref()
    }
    pub const fn checkpoint_source_root_sha256(&self) -> [u8; 32] {
        self.checkpoint_source_root_sha256
    }
    pub const fn source_root(&self) -> &DurablePhysicalRootManifest {
        &self.source_root
    }
    pub const fn source_root_sha256(&self) -> [u8; 32] {
        self.source_root_sha256
    }
    pub fn published_root(&self) -> Option<&DurablePhysicalRootManifest> {
        self.published_root.as_ref()
    }
    pub const fn published_root_sha256(&self) -> Option<[u8; 32]> {
        self.published_root_sha256
    }
    pub const fn topologies(
        &self,
    ) -> Option<(PhysicalInventoryTranscriptV1, PhysicalInventoryTranscriptV1)> {
        match &self.verified_transition {
            Some(transition) => Some((transition.source_topology(), transition.result_topology())),
            None => None,
        }
    }
    pub fn verified_transition(&self) -> Option<&VerifiedReleasedV3InventoryTransition> {
        self.verified_transition.as_ref()
    }
    pub fn historical_batches(&self) -> &[VerifiedHistoricalPendingWalBatch] {
        &self.historical_batches
    }
    pub fn ordered_history(&self) -> Option<&crate::VerifiedOrderedRootHistory> {
        self.ordered_history.as_deref()
    }
    pub fn ordered_released_batches(&self) -> &[VerifiedOrderedPendingWalReleaseBatch] {
        &self.ordered_released_batches
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
    pub const fn manifest_record(&self) -> PersistedRecordIdentity {
        self.manifest_record
    }
    pub const fn manifest_frame_sha256(&self) -> [u8; 32] {
        self.manifest_frame_sha256
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
