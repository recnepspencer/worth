//! Store construction after independent post-checkpoint WAL release rejoin.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedPendingWalReleaseCustody,
    VerifiedSelectedTierEpochCustody,
};

use crate::physical_runtime::record_serving::RecoveredReleaseCustody;
use crate::physical_runtime::{
    ClosedPhysicalRecoveryCleanup, CompletedPhysicalRecoveryFreshReopen,
    PhysicalRecoveryAllocationAdmission, PhysicalRecoveryCoordination,
};

use super::{
    selected_rejoin, validate_construction_binding, PhysicalRecoveryConstructionPort,
    PreparedRecoveredRuntimeCoreBacking, RecoveredPhysicalRuntimeConstructionDenial,
    RecoveredPhysicalRuntimeCore,
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
            PreparedRecoveredRuntimeCoreBacking::prepare_direct(),
            coordination,
            media,
            cleanup,
            recovery_allocation,
            Box::new(pending),
            vec![effective],
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
            PreparedRecoveredRuntimeCoreBacking::prepare_direct(),
            coordination,
            media,
            cleanup,
            recovery_allocation,
            Box::new(pending),
            vec![effective],
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
            PreparedRecoveredRuntimeCoreBacking::prepare_direct(),
            coordination,
            media,
            cleanup,
            recovery_allocation,
            Box::new(pending),
            vec![effective],
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
            PreparedRecoveredRuntimeCoreBacking::prepare_direct(),
            coordination,
            media,
            cleanup,
            recovery_allocation,
            Box::new(pending),
            vec![effective],
            Some(tier),
            pause_before_final_reread,
        )
    }

    pub(super) fn construct_pending_wal_release(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        pending: Box<VerifiedPendingWalReleaseCustody>,
        effective: Vec<VerifiedEffectiveReleaseHeadRosterV14>,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        let Some(effective_ref) = effective.first() else {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch);
        };
        if effective.len() != 1
            || cleanup.live_media_handle_delta() != 0
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
        if effective_ref.retained_bytes() > recovery_allocation.byte_limit() {
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
                effective_ref,
                tier.as_ref(),
                pause_before_final_reread,
            ) {
                Ok(value) => value,
                Err(denial) => {
                    let _ = coordination.shutdown_is_quiescent();
                    return Err(super::rejoin_denial::construction_denial(denial));
                }
            };
        Self::finish_pending_wal_release(
            backing,
            coordination,
            media,
            reopen,
            pending,
            effective,
            tier,
            selected_wal,
            selected_controls,
        )
    }

    #[inline(never)]
    fn finish_pending_wal_release(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        reopen: CompletedPhysicalRecoveryFreshReopen,
        pending: Box<VerifiedPendingWalReleaseCustody>,
        mut effective: Vec<VerifiedEffectiveReleaseHeadRosterV14>,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        selected_wal: selected_rejoin::SelectedWalMediaFingerprint,
        selected_controls: selected_rejoin::SelectedControlMediaFingerprint,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        let effective = effective
            .pop()
            .expect("prepaid effective roster holds one value");
        Self::construct_inner(
            backing,
            coordination,
            media,
            reopen,
            Some(RecoveredReleaseCustody::PendingWal {
                claim: *pending,
                effective,
            }),
            tier,
            Some(selected_wal),
            Some(selected_controls),
        )
    }
}
