//! Construction after an independently rejoined, fully published V3 history.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
    VerifiedSelectedTierEpochCustody,
};

use crate::physical_runtime::{
    ClosedPhysicalRecoveryCleanup, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryCoordination,
};

use super::{
    validate_construction_binding, PhysicalRecoveryConstructionPort,
    RecoveredPhysicalRuntimeConstructionDenial, RecoveredPhysicalRuntimeCore,
};

impl PhysicalRecoveryConstructionPort {
    pub fn construct_with_verified_ordered_historical_release(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        historical: VerifiedOrderedHistoricalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
        tier: Option<VerifiedSelectedTierEpochCustody>,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        if cleanup.live_media_handle_delta() != 0
            || recovery_allocation.store_identity() != media.store_identity()
            || coordination.recovery_allocation_admission() != Some(recovery_allocation)
            || effective.retained_bytes() > recovery_allocation.byte_limit()
        {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::CleanupMediaNotQuiescent);
        }
        let reopen = cleanup.into_reopen();
        if let Err(denial) = validate_construction_binding(&coordination, &media, &reopen) {
            let _ = coordination.shutdown_is_quiescent();
            return Err(denial);
        }
        // The partial historical walker does not yet independently rejoin
        // every addressed V14 head path and final rooted roster. Retaining
        // its typed claim is not a substitute for that media proof.
        let _ = (media, reopen, historical, effective, tier);
        let _ = coordination.shutdown_is_quiescent();
        Err(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch)
    }
}
