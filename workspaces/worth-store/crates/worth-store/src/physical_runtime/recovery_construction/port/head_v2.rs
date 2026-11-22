//! Fully joined V2 checkpoint custody enters Store only through a second
//! bounded media rejoin under the concrete recovery allocation admission.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_recovery_physics::{
    VerifiedSelectedReleaseHeadCustodyV2, VerifiedSelectedTierEpochCustody,
};

use super::super::{
    selected_rejoin, PreparedRecoveredRuntimeCoreBacking,
    RecoveredPhysicalRuntimeConstructionDenial, RecoveredPhysicalRuntimeCore,
};
use super::PhysicalRecoveryConstructionPort;
use crate::physical_runtime::record_serving::RecoveredReleaseCustody;
use crate::physical_runtime::{
    ClosedPhysicalRecoveryCleanup, CompletedPhysicalRecoveryFreshReopen,
    PhysicalRecoveryAllocationAdmission, PhysicalRecoveryCoordination,
};

impl PhysicalRecoveryConstructionPort {
    pub(super) fn construct_head_v2(
        backing: PreparedRecoveredRuntimeCoreBacking,
        mut coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        claim: Box<VerifiedSelectedReleaseHeadCustodyV2>,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        if cleanup.live_media_handle_delta() != 0
            || coordination
                .require_selected_checkpoint(claim.checkpoint())
                .is_err()
            || recovery_allocation.store_identity() != media.store_identity()
            || coordination.recovery_allocation_admission() != Some(recovery_allocation)
        {
            let _ = coordination.shutdown_is_quiescent();
            return Err(RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch);
        }
        let reopen = cleanup.into_reopen();
        if let Err(denial) = super::validate_construction_binding(&coordination, &media, &reopen) {
            let _ = coordination.shutdown_is_quiescent();
            return Err(denial);
        }
        let mut resident =
            match selected_rejoin::resident::StoreRejoinResidentLedger::from_coordination(
                &mut coordination,
            ) {
                Ok(resident) => resident,
                Err(cause) => {
                    let _ = coordination.shutdown_is_quiescent();
                    return Err(
                        RecoveredPhysicalRuntimeConstructionDenial::RejoinResidentBoundary {
                            boundary: crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary::RetainedPlanningHandoff,
                            cause,
                        },
                    );
                }
            };
        let (media, selected_wal, selected_controls) = match selected_rejoin::head_v2::observe_claim(
            &coordination,
            media,
            &reopen,
            recovery_allocation,
            &mut resident,
            &claim,
            tier.as_ref(),
            pause_before_final_reread,
        ) {
            Ok(value) => value,
            Err(denial) => {
                let _ = coordination.shutdown_is_quiescent();
                return Err(super::rejoin_denial::construction_denial(denial));
            }
        };
        Self::finish_head_v2(
            backing,
            coordination,
            media,
            reopen,
            claim,
            tier,
            selected_wal,
            selected_controls,
        )
    }

    #[inline(never)]
    fn finish_head_v2(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        reopen: CompletedPhysicalRecoveryFreshReopen,
        claim: Box<VerifiedSelectedReleaseHeadCustodyV2>,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        selected_wal: selected_rejoin::SelectedWalMediaFingerprint,
        selected_controls: selected_rejoin::SelectedControlMediaFingerprint,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_inner(
            backing,
            coordination,
            media,
            reopen,
            Some(RecoveredReleaseCustody::HeadV2(*claim)),
            tier,
            Some(selected_wal),
            Some(selected_controls),
        )
    }
}
