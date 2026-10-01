//! Exhaustive one-shot seal construction from Store-rejoined custody.

use super::*;

impl RecoveredPhysicalCheckpointCustody {
    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn from_verified_no_release(
        store: StableStoreIdentity,
        recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
        root: DurablePhysicalRootManifest,
        verified: VerifiedSelectedNoReleaseCustody,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        selected_wal: crate::physical_runtime::recovery_construction::SelectedWalMediaFingerprint,
        selected_controls: crate::physical_runtime::recovery_construction::SelectedControlMediaFingerprint,
    ) -> Self {
        Self {
            store,
            recovery_allocation,
            root,
            head_v2: None,
            no_release: Some(verified),
            pending_wal_release: None,
            effective_release_heads: None,
            historical_release: None,
            tier,
            selected_wal,
            selected_controls: Some(selected_controls),
        }
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn from_verified_pending_wal_release(
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
            store,
            recovery_allocation,
            root,
            head_v2: None,
            no_release: None,
            pending_wal_release: Some(verified),
            effective_release_heads: Some(effective),
            historical_release: None,
            tier,
            selected_wal,
            selected_controls: Some(selected_controls),
        }
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn from_verified_ordered_historical_release(
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
            store,
            recovery_allocation,
            root,
            head_v2: None,
            no_release: None,
            pending_wal_release: None,
            effective_release_heads: Some(effective),
            historical_release: Some(verified),
            tier,
            selected_wal,
            selected_controls: Some(selected_controls),
        }
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn from_verified_head_v2(
        store: StableStoreIdentity,
        recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
        root: DurablePhysicalRootManifest,
        verified: VerifiedSelectedReleaseHeadCustodyV2,
        tier: Option<VerifiedSelectedTierEpochCustody>,
        selected_wal: crate::physical_runtime::recovery_construction::SelectedWalMediaFingerprint,
        selected_controls: crate::physical_runtime::recovery_construction::SelectedControlMediaFingerprint,
    ) -> Self {
        Self {
            store,
            recovery_allocation,
            root,
            head_v2: Some(verified),
            no_release: None,
            pending_wal_release: None,
            effective_release_heads: None,
            historical_release: None,
            tier,
            selected_wal,
            selected_controls: Some(selected_controls),
        }
    }
}
