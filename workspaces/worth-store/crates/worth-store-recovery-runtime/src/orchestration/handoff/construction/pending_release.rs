//! Post-checkpoint pending WAL release retains its exact effective roster.
use super::*;
impl RejoinConstruction {
    pub(super) fn pending_release(
        self,
        pending: VerifiedPendingWalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
        recovery_allocation: Option<PhysicalRecoveryAllocationAdmission>,
        _selection: &crate::orchestration::ResidentSourceSelection,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        let Self {
            coordination,
            media,
            closed_cleanup,
        } = self;

        let allocation = recovery_allocation.expect("pending recovery allocation was checked");

        #[cfg(feature = "certification-test-authority")]
        if crate::certification::enabled() {
            let reservation = _selection
                .page_facts()
                .placements()
                .iter()
                .copied()
                .find(|route| route.record() == pending.reservation_record())
                .expect("pending-WAL reservation was already joined");
            crate::certification::pause_after_claim(reservation);
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_with_verified_pending_wal_release_and_rejoin_pause(
                    coordination, media, closed_cleanup, allocation, pending, effective,
                    crate::certification::take_rejoin_pause(),
                )
        } else {
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_pending_wal_release(
                    coordination, media, closed_cleanup, allocation, pending, effective,
                )
        }
        #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_pending_wal_release(
                coordination, media, closed_cleanup, allocation, pending, effective,
            )
    }
    pub(super) fn tier_pending_release(
        self,
        pending: VerifiedPendingWalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
        tier: VerifiedSelectedTierEpochCustody,
        recovery_allocation: Option<PhysicalRecoveryAllocationAdmission>,
        _selection: &crate::orchestration::ResidentSourceSelection,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        let Self {
            coordination,
            media,
            closed_cleanup,
        } = self;

        let allocation = recovery_allocation.expect("pending recovery allocation was checked");

        #[cfg(feature = "certification-test-authority")]
        if crate::certification::enabled() {
            let reservation = _selection
                .page_facts()
                .placements()
                .iter()
                .copied()
                .find(|route| route.record() == pending.reservation_record())
                .expect("pending-WAL reservation was already joined");
            crate::certification::pause_after_claim(reservation);
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_with_verified_tier_and_pending_wal_release_and_rejoin_pause(
                    coordination, media, closed_cleanup, allocation, pending, effective, tier,
                    crate::certification::take_rejoin_pause(),
                )
        } else {
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_tier_and_pending_wal_release(
                    coordination, media, closed_cleanup, allocation, pending, effective, tier,
                )
        }
        #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_tier_and_pending_wal_release(
                coordination, media, closed_cleanup, allocation, pending, effective, tier,
            )
    }
}
