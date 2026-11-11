//! Ordered historical release rejoins the same verified effective roster.
use super::*;
impl RejoinConstruction {
    pub(super) fn ordered_release(
        self,
        historical: VerifiedOrderedHistoricalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        recovery_allocation: Option<PhysicalRecoveryAllocationAdmission>,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        let Self {
            coordination,
            media,
            closed_cleanup,
        } = self;

        let allocation = recovery_allocation.expect("historical recovery allocation was checked");

        worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_ordered_historical_release(
                coordination,
                media,
                closed_cleanup,
                allocation,
                historical,
                effective,
                tier,
            )
    }
}
