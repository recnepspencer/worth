//! Prepaid recovery-runtime entry points that consume caller-funded core and
//! custody storage without re-boxing or allocating after publication.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
    VerifiedPendingWalReleaseCustody, VerifiedSelectedNoReleaseCustody,
    VerifiedSelectedReleaseHeadCustodyV2, VerifiedSelectedTierEpochCustody,
};

use super::{
    PhysicalRecoveryConstructionPort, PreparedRecoveredRuntimeCoreBacking,
    RecoveredPhysicalRuntimeConstructionDenial, RecoveredPhysicalRuntimeCore,
};
use crate::physical_runtime::{
    AbsentCheckpointWitness, ClosedPhysicalRecoveryCleanup,
    ConfiguredPhysicalDurabilityDeclaration, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryCoordination,
};

impl PhysicalRecoveryConstructionPort {
    pub fn construct_with_backing(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_direct(backing, coordination, media, cleanup)
    }

    pub fn construct_generation_zero_with_backing(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        declaration: ConfiguredPhysicalDurabilityDeclaration,
        absent: AbsentCheckpointWitness,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_generation_zero_inner(
            backing,
            coordination,
            media,
            cleanup,
            declaration,
            absent,
            || {},
        )
    }

    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_generation_zero_with_backing_and_rejoin_pause(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        declaration: ConfiguredPhysicalDurabilityDeclaration,
        absent: AbsentCheckpointWitness,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_generation_zero_inner(
            backing,
            coordination,
            media,
            cleanup,
            declaration,
            absent,
            pause_before_final_reread,
        )
    }

    pub fn construct_with_verified_no_release_and_backing(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        no_release: Box<VerifiedSelectedNoReleaseCustody>,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_no_release(backing, coordination, media, cleanup, no_release, || {})
    }

    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_with_verified_no_release_and_backing(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        no_release: Box<VerifiedSelectedNoReleaseCustody>,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_no_release(
            backing,
            coordination,
            media,
            cleanup,
            no_release,
            pause_before_final_reread,
        )
    }

    pub fn construct_with_verified_tier_no_release_and_backing(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        tier: VerifiedSelectedTierEpochCustody,
        no_release: Box<VerifiedSelectedNoReleaseCustody>,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_tier_no_release(
            backing,
            coordination,
            media,
            cleanup,
            tier,
            no_release,
            || {},
        )
    }

    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_with_verified_tier_no_release_and_backing(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        tier: VerifiedSelectedTierEpochCustody,
        no_release: Box<VerifiedSelectedNoReleaseCustody>,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_tier_no_release(
            backing,
            coordination,
            media,
            cleanup,
            tier,
            no_release,
            pause_before_final_reread,
        )
    }

    pub fn construct_with_verified_head_custody_v2_and_backing(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        claim: Box<VerifiedSelectedReleaseHeadCustodyV2>,
        tier: Option<VerifiedSelectedTierEpochCustody>,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_head_v2(
            backing,
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
    pub fn certification_construct_with_verified_head_custody_v2_and_backing_and_rejoin_pause(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        claim: Box<VerifiedSelectedReleaseHeadCustodyV2>,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_head_v2(
            backing,
            coordination,
            media,
            cleanup,
            recovery_allocation,
            claim,
            tier,
            pause_before_final_reread,
        )
    }

    pub fn construct_with_verified_ordered_historical_release_and_backing(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        historical: Box<VerifiedOrderedHistoricalReleaseCustody>,
        effective: Box<VerifiedEffectiveReleaseHeadRosterV14>,
        tier: Option<VerifiedSelectedTierEpochCustody>,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_ordered_historical_inner(
            backing,
            coordination,
            media,
            cleanup,
            recovery_allocation,
            historical,
            effective,
            tier,
        )
    }

    pub fn construct_with_verified_pending_wal_release_and_backing(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        pending: Box<VerifiedPendingWalReleaseCustody>,
        effective: Vec<VerifiedEffectiveReleaseHeadRosterV14>,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_pending_wal_release(
            backing,
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

    pub fn construct_with_verified_tier_and_pending_wal_release_and_backing(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        pending: Box<VerifiedPendingWalReleaseCustody>,
        effective: Vec<VerifiedEffectiveReleaseHeadRosterV14>,
        tier: VerifiedSelectedTierEpochCustody,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_pending_wal_release(
            backing,
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

    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_construct_with_verified_pending_wal_release_and_backing_and_rejoin_pause(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        pending: Box<VerifiedPendingWalReleaseCustody>,
        effective: Vec<VerifiedEffectiveReleaseHeadRosterV14>,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_pending_wal_release(
            backing,
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
    pub fn certification_construct_with_verified_tier_and_pending_wal_release_and_backing_and_rejoin_pause(
        backing: PreparedRecoveredRuntimeCoreBacking,
        coordination: PhysicalRecoveryCoordination,
        media: AdmittedRecoveryFilesystemMedia,
        cleanup: ClosedPhysicalRecoveryCleanup,
        recovery_allocation: PhysicalRecoveryAllocationAdmission,
        pending: Box<VerifiedPendingWalReleaseCustody>,
        effective: Vec<VerifiedEffectiveReleaseHeadRosterV14>,
        tier: VerifiedSelectedTierEpochCustody,
        pause_before_final_reread: impl FnOnce(),
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        Self::construct_pending_wal_release(
            backing,
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
}
