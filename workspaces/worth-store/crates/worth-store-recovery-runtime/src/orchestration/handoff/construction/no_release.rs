//! Selected checkpoint construction without released-head custody.
use super::*;
impl RejoinConstruction {
    pub(super) fn no_release(
        self,
        no_release: VerifiedSelectedNoReleaseCustody,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        let Self {
            coordination,
            media,
            closed_cleanup,
        } = self;

        #[cfg(feature = "certification-test-authority")]
        if crate::certification::enabled() {
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_with_verified_no_release(
                    coordination,
                    media,
                    closed_cleanup,
                    no_release,
                    crate::certification::take_rejoin_pause(),
                )
        } else {
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_no_release(
                    coordination, media, closed_cleanup, no_release,
                )
        }
        #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_no_release(
                coordination, media, closed_cleanup, no_release,
            )
    }
    pub(super) fn tier_no_release(
        self,
        no_release: VerifiedSelectedNoReleaseCustody,
        tier: VerifiedSelectedTierEpochCustody,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        let Self {
            coordination,
            media,
            closed_cleanup,
        } = self;

        #[cfg(feature = "certification-test-authority")]
        if crate::certification::enabled() {
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_with_verified_tier_no_release(
                    coordination,
                    media,
                    closed_cleanup,
                    tier,
                    no_release,
                    crate::certification::take_rejoin_pause(),
                )
        } else {
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_tier_no_release(
                    coordination, media, closed_cleanup, tier, no_release,
                )
        }
        #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_tier_no_release(
                coordination, media, closed_cleanup, tier, no_release,
            )
    }
}
