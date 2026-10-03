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
    selected_rejoin, validate_construction_binding, PhysicalRecoveryConstructionPort,
    RecoveredPhysicalRuntimeConstructionDenial, RecoveredPhysicalRuntimeCore,
};

impl PhysicalRecoveryConstructionPort {
    pub fn construct_with_verified_ordered_historical_release(
        mut coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        historical: VerifiedOrderedHistoricalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
        tier: Option<VerifiedSelectedTierEpochCustody>,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        if cleanup.live_media_handle_delta() != 0
            || coordination
                .require_selected_checkpoint(historical.checkpoint())
                .is_err()
            || tier.as_ref().is_some_and(|claim| {
                coordination
                    .require_selected_checkpoint(claim.checkpoint())
                    .is_err()
            })
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
        // Tier custody requires its own source join; it may not enter the
        // completed-history owner, which joins NoRelease and HeadV2 bases.
        if tier.is_some() {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch);
        }
        let (media, selected_wal, selected_controls) =
            match selected_rejoin::completed_history::observe_claim(
                &mut coordination,
                media,
                &reopen,
                &historical,
                &effective,
                recovery_allocation,
                || {},
            ) {
                Ok(value) => value,
                Err(denial) => {
                    let _ = coordination.shutdown_is_quiescent();
                    return Err(super::rejoin_denial::construction_denial(denial));
                }
            };
        let mut core = Self::construct_inner(
            coordination,
            media,
            reopen,
            None,
            None,
            Some(selected_wal),
            Some(selected_controls),
        )?;
        core.historical_release_custody = Some(historical);
        core.effective_release_heads = Some(effective);
        Ok(core)
    }
}
