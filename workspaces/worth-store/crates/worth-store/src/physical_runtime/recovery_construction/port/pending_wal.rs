//! Store construction after independent post-checkpoint WAL release rejoin.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedPendingWalReleaseCustody,
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
    pub fn construct_with_verified_pending_wal_release(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        pending: VerifiedPendingWalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_pending_wal_release(
            coordination,
            media,
            cleanup,
            recovery_allocation,
            pending,
            effective,
            None,
            || {},
        )
    }

    pub fn construct_with_verified_tier_and_pending_wal_release(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        pending: VerifiedPendingWalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
        tier: VerifiedSelectedTierEpochCustody,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_pending_wal_release(
            coordination,
            media,
            cleanup,
            recovery_allocation,
            pending,
            effective,
            Some(tier),
            || {},
        )
    }

    /// Exercises the production pending-WAL rejoin with a pause between its
    /// first admitted-media observation and final bounded reread.
    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_with_verified_pending_wal_release_and_rejoin_pause(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        pending: VerifiedPendingWalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_pending_wal_release(
            coordination,
            media,
            cleanup,
            recovery_allocation,
            pending,
            effective,
            None,
            pause_before_final_reread,
        )
    }

    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_with_verified_tier_and_pending_wal_release_and_rejoin_pause(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        pending: VerifiedPendingWalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
        tier: VerifiedSelectedTierEpochCustody,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_pending_wal_release(
            coordination,
            media,
            cleanup,
            recovery_allocation,
            pending,
            effective,
            Some(tier),
            pause_before_final_reread,
        )
    }

    fn construct_pending_wal_release(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        pending: VerifiedPendingWalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        if cleanup.live_media_handle_delta() != 0
            || coordination
                .require_selected_checkpoint(pending.checkpoint())
                .is_err()
            || tier.as_ref().is_some_and(|claim| {
                coordination
                    .require_selected_checkpoint(claim.checkpoint())
                    .is_err()
            })
            || recovery_allocation.store_identity() != media.store_identity()
            || coordination.recovery_allocation_admission() != Some(recovery_allocation)
        {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::CleanupMediaNotQuiescent);
        }
        if effective.retained_bytes() > recovery_allocation.byte_limit() {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch);
        }
        let reopen = cleanup.into_reopen();
        if let Err(denial) = validate_construction_binding(&coordination, &media, &reopen) {
            let _ = coordination.shutdown_is_quiescent();
            return Err(denial);
        }
        let (media, selected_wal, selected_controls) =
            match selected_rejoin::pending_wal_release::observe_claim(
                &coordination,
                media,
                &reopen,
                &pending,
                recovery_allocation,
                &effective,
                tier.as_ref(),
                pause_before_final_reread,
            ) {
                Ok(value) => value,
                Err(_) => {
                    let _ = coordination.shutdown_is_quiescent();
                    return Err(
                        RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch,
                    );
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
        core.pending_wal_release_custody = Some(pending);
        core.effective_release_heads = Some(effective);
        core.tier_custody = tier;
        Ok(core)
    }
}
