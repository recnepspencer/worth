//! Fully joined V2 checkpoint custody enters Store only through a second
//! bounded media rejoin under the concrete recovery allocation admission.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_recovery_physics::{
    VerifiedSelectedReleaseHeadCustodyV2, VerifiedSelectedTierEpochCustody,
};

use super::super::{
    selected_rejoin, RecoveredPhysicalRuntimeConstructionDenial, RecoveredPhysicalRuntimeCore,
};
use super::PhysicalRecoveryConstructionPort;
use crate::physical_runtime::{
    ClosedPhysicalRecoveryCleanup, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryCoordination,
};

impl PhysicalRecoveryConstructionPort {
    pub fn construct_with_verified_head_custody_v2(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        claim: VerifiedSelectedReleaseHeadCustodyV2,
        tier: Option<VerifiedSelectedTierEpochCustody>,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_head_v2(
            coordination,
            media,
            cleanup,
            recovery_allocation,
            claim,
            tier,
            || {},
        )
    }

    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_with_verified_head_custody_v2_and_rejoin_pause(
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        claim: VerifiedSelectedReleaseHeadCustodyV2,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_head_v2(
            coordination,
            media,
            cleanup,
            recovery_allocation,
            claim,
            tier,
            pause_before_final_reread,
        )
    }

    fn construct_head_v2(
        mut coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        claim: VerifiedSelectedReleaseHeadCustodyV2,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        if cleanup.live_media_handle_delta() != 0
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
        let mut core = Self::construct_inner(
            coordination,
            media,
            reopen,
            None,
            None,
            tier,
            Some(selected_wal),
            Some(selected_controls),
        )?;
        core.head_v2_custody = Some(claim);
        Ok(core)
    }
}
