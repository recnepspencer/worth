//! Exhaustive one-shot seal construction from Store-rejoined custody.

use super::*;

impl RecoveredPhysicalCheckpointCustody {
    #[cfg(feature = "recovery-runtime-owner")]
    #[inline(never)]
    pub(in crate::physical_runtime) fn from_verified_release(
        backing: PreparedRecoveredCustodyBacking,
        residency: crate::physical_runtime::instance::PhysicalResidencyOwner,
        checkpoint_ownership: crate::physical_runtime::recovery_coordination::RecoveryCheckpointOwnership,
        store: StableStoreIdentity,
        recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
        root: DurablePhysicalRootManifest,
        release: RecoveredReleaseCustody,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        selected_wal: crate::physical_runtime::recovery_construction::SelectedWalMediaFingerprint,
        selected_controls: crate::physical_runtime::recovery_construction::SelectedControlMediaFingerprint,
    ) -> Self {
        Self {
            residency,
            evidence: backing.finish(RecoveredCheckpointCustodyPayload {
                checkpoint_ownership,
                store,
                recovery_allocation,
                root,
                release,
                tier,
                selected_wal,
                selected_controls: Some(selected_controls),
            }),
        }
    }
}
