//! Selected head custody uses its exact original recovery allocation.
use super::*;
#[cfg(feature = "certification-test-authority")]
use worth_store_physical_format::{BlobRecordKind, SelectedRecordContentClass};
impl RejoinConstruction {
    pub(super) fn source_heads(
        self,
        claim: VerifiedSelectedReleaseHeadCustodyV2,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        recovery_allocation: Option<PhysicalRecoveryAllocationAdmission>,
        selection: &crate::orchestration::ResidentSourceSelection,
    ) -> Result<RecoveredPhysicalRuntimeCore, RecoveredPhysicalRuntimeConstructionDenial> {
        let Self {
            coordination,
            media,
            closed_cleanup,
        } = self;

        if let Some(allocation) = recovery_allocation {
            #[cfg(feature = "certification-test-authority")]
            if crate::certification::enabled() {
                let descriptor = claim.selected_heads().iter().find_map(|head| {
                    selection
                        .page_facts()
                        .placements()
                        .iter()
                        .copied()
                        .find(|route| {
                            route.record() == head.descriptor_record()
                                && route.content_class()
                                    == SelectedRecordContentClass::Blob(
                                        BlobRecordKind::ReclaimDescriptorV3,
                                    )
                        })
                });
                if let Some(descriptor) = descriptor {
                    crate::certification::pause_after_claim(descriptor);
                }
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::certification_construct_with_verified_head_custody_v2_and_rejoin_pause(
                    coordination,
                    media,
                    closed_cleanup,
                    allocation,
                    claim,
                    tier,
                    crate::certification::take_rejoin_pause(),
                )
            } else {
                worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_head_custody_v2(
                    coordination, media, closed_cleanup, allocation, claim, tier,
                )
            }
            #[cfg(not(feature = "certification-test-authority"))]
            worth_store::physical_runtime::PhysicalRecoveryConstructionPort::construct_with_verified_head_custody_v2(
                coordination, media, closed_cleanup, allocation, claim, tier,
            )
        } else {
            Err(worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::SelectedCustodyMismatch)
        }
    }
}
