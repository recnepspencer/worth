//! Pending V14 release from a selected V2 checkpoint-source head roster.
//! A keyed predecessor is proved from the actual selected WAL source path,
//! never from the Store-wide tip or a lifetime manifest ancestry scan.

use worth_store_physical_format::ReleasedDropWalFateWitnessV1;

use super::{
    PendingReleaseCheckpointBase, PendingWalReleaseCustodyDenial, VerifiedPendingWalReleaseCustody,
};
use crate::{
    ImmutablePhysicalRedoPlan, PhysicalRedoProjection, PhysicalSourceSelection,
    ReconciledOperationFates, VerifiedSelectedReleaseHeadCustodyV2,
    VerifiedSelectedReleaseHeadReplayV14, VerifiedSelectedTierEpochCustody,
    WitnessedSelectedControlFrame,
};

impl VerifiedPendingWalReleaseCustody {
    /// First release from NoRelease also requires the exact WAL-chosen V14
    /// path; a headless descriptor cannot mint this new claim.
    #[allow(clippy::too_many_arguments)]
    pub fn admit_with_head_replay(
        selected: &PhysicalSourceSelection,
        head_replay: VerifiedSelectedReleaseHeadReplayV14,
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
        let marker =
            crate::source_precedence::no_release_custody::selected_checkpoint_marker(selected)
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
            Some(head_replay),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn admit_from_head_v2(
        selected: &PhysicalSourceSelection,
        selected_head: VerifiedSelectedReleaseHeadCustodyV2,
        head_replay: VerifiedSelectedReleaseHeadReplayV14,
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
            PendingReleaseCheckpointBase::ReleasedHeadV2(Box::new(selected_head)),
            tier,
            projection,
            redo,
            reservation_frame,
            manifest_frame,
            wal_fate,
            member_redo_digest,
            fates,
            policy,
            Some(head_replay),
        )
    }
}
