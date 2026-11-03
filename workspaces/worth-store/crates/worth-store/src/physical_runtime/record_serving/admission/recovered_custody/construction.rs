//! Exhaustive one-shot seal construction from Store-rejoined custody.

use super::*;

impl RecoveredPhysicalCheckpointCustody {
    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn from_verified_no_release(
        residency: crate::physical_runtime::instance::PhysicalResidencyOwner,
        checkpoint_ownership: crate::physical_runtime::recovery_coordination::RecoveryCheckpointOwnership,
        store: StableStoreIdentity,
        recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
        root: DurablePhysicalRootManifest,
        verified: RecoveredNoReleaseCustody,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        selected_wal: crate::physical_runtime::recovery_construction::SelectedWalMediaFingerprint,
        selected_controls: crate::physical_runtime::recovery_construction::SelectedControlMediaFingerprint,
    ) -> Self {
        Self {
            residency,
            evidence: RecoveredCheckpointCustodyEvidence {
                checkpoint_ownership,
                store,
                recovery_allocation,
                root,
                release: RecoveredReleaseCustody::NoRelease(verified),
                tier,
                selected_wal,
                selected_controls: Some(selected_controls),
            },
        }
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn from_verified_pending_wal_release(
        residency: crate::physical_runtime::instance::PhysicalResidencyOwner,
        checkpoint_ownership: crate::physical_runtime::recovery_coordination::RecoveryCheckpointOwnership,
        store: StableStoreIdentity,
        recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
        root: DurablePhysicalRootManifest,
        verified: VerifiedPendingWalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        selected_wal: crate::physical_runtime::recovery_construction::SelectedWalMediaFingerprint,
        selected_controls: crate::physical_runtime::recovery_construction::SelectedControlMediaFingerprint,
    ) -> Self {
        Self {
            residency,
            evidence: RecoveredCheckpointCustodyEvidence {
                checkpoint_ownership,
                store,
                recovery_allocation,
                root,
                release: RecoveredReleaseCustody::PendingWal {
                    claim: verified,
                    effective,
                },
                tier,
                selected_wal,
                selected_controls: Some(selected_controls),
            },
        }
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn from_verified_ordered_historical_release(
        residency: crate::physical_runtime::instance::PhysicalResidencyOwner,
        checkpoint_ownership: crate::physical_runtime::recovery_coordination::RecoveryCheckpointOwnership,
        store: StableStoreIdentity,
        recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
        root: DurablePhysicalRootManifest,
        verified: VerifiedOrderedHistoricalReleaseCustody,
        effective: VerifiedEffectiveReleaseHeadRosterV14,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        selected_wal: crate::physical_runtime::recovery_construction::SelectedWalMediaFingerprint,
        selected_controls: crate::physical_runtime::recovery_construction::SelectedControlMediaFingerprint,
    ) -> Self {
        Self {
            residency,
            evidence: RecoveredCheckpointCustodyEvidence {
                checkpoint_ownership,
                store,
                recovery_allocation,
                root,
                release: RecoveredReleaseCustody::OrderedHistorical {
                    effective,
                    claim: verified,
                },
                tier,
                selected_wal,
                selected_controls: Some(selected_controls),
            },
        }
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn from_verified_head_v2(
        residency: crate::physical_runtime::instance::PhysicalResidencyOwner,
        checkpoint_ownership: crate::physical_runtime::recovery_coordination::RecoveryCheckpointOwnership,
        store: StableStoreIdentity,
        recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
        root: DurablePhysicalRootManifest,
        verified: VerifiedSelectedReleaseHeadCustodyV2,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        selected_wal: crate::physical_runtime::recovery_construction::SelectedWalMediaFingerprint,
        selected_controls: crate::physical_runtime::recovery_construction::SelectedControlMediaFingerprint,
    ) -> Self {
        Self {
            residency,
            evidence: RecoveredCheckpointCustodyEvidence {
                checkpoint_ownership,
                store,
                recovery_allocation,
                root,
                release: RecoveredReleaseCustody::HeadV2(verified),
                tier,
                selected_wal,
                selected_controls: Some(selected_controls),
            },
        }
    }
}
