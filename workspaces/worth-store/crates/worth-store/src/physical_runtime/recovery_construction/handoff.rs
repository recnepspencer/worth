use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, DurablePhysicalRootManifest,
};
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
    VerifiedPendingWalReleaseCustody, VerifiedSelectedCheckpointCustody,
    VerifiedSelectedNoReleaseCustody, VerifiedSelectedReleaseHeadCustodyV2,
    VerifiedSelectedTierEpochCustody,
};

use crate::physical_runtime::{CompletedPhysicalRecoveryFreshReopen, RuntimeIdentity};

pub struct RecoveredPhysicalRuntimeCore {
    pub(super) store: StableStoreIdentity,
    pub(super) recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
    pub(super) runtime: RuntimeIdentity,
    pub(super) recovery_runtime: RuntimeIdentity,
    pub(super) root: DurablePhysicalRootManifest,
    pub(super) media: AdmittedRecoveryFilesystemMedia,
    pub(super) reopen: CompletedPhysicalRecoveryFreshReopen,
    pub(super) checkpoint_custody: Option<VerifiedSelectedCheckpointCustody>,
    pub(super) head_v2_custody: Option<VerifiedSelectedReleaseHeadCustodyV2>,
    pub(super) no_release_custody: Option<VerifiedSelectedNoReleaseCustody>,
    pub(super) pending_wal_release_custody: Option<VerifiedPendingWalReleaseCustody>,
    pub(super) effective_release_heads: Option<VerifiedEffectiveReleaseHeadRosterV14>,
    pub(super) historical_release_custody: Option<VerifiedOrderedHistoricalReleaseCustody>,
    pub(super) tier_custody: Option<VerifiedSelectedTierEpochCustody>,
    pub(super) selected_wal: Option<super::selected_rejoin::SelectedWalMediaFingerprint>,
    pub(super) selected_controls: Option<super::selected_rejoin::SelectedControlMediaFingerprint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveredPhysicalRuntimeConstructionDenial {
    ResidentAdmission(crate::physical_runtime::PhysicalRecoveryRejoinResidentAdmissionDenial),
    RejoinResident(super::selected_rejoin::PhysicalRecoveryRejoinResidentDenial),
    RejoinResidentBoundary {
        boundary: PhysicalRecoveryRejoinResidentBoundary,
        cause: super::selected_rejoin::PhysicalRecoveryRejoinResidentDenial,
    },
    RejoinWalResident {
        /// `None` means the caller did not attach a first/final rejoin pass.
        boundary: Option<PhysicalRecoveryRejoinResidentBoundary>,
        stage: PhysicalRecoveryWalResidentStage,
        artifact_count: usize,
        /// Zero-based discovery order, not a stable WAL identity or sequence.
        artifact_ordinal: Option<usize>,
        frame_offset: Option<u64>,
        cause: super::selected_rejoin::PhysicalRecoveryRejoinResidentDenial,
    },
    RejoinResidentRead {
        artifact: worth_store_physical_backend::RecoveryDiscoveryArtifact,
        offset: u64,
        requested: usize,
        cause: super::selected_rejoin::PhysicalRecoveryRejoinResidentDenial,
    },
    RejoinReadBufferLengthMismatch {
        artifact: worth_store_physical_backend::RecoveryDiscoveryArtifact,
        offset: u64,
        requested: usize,
        observed: usize,
    },
    BindingMismatch,
    ConstructionAuthorityMismatch,
    AllocationStoreMismatch {
        allocation: StableStoreIdentity,
        selected: StableStoreIdentity,
    },
    CoordinationNotQuiescent,
    CleanupMediaNotQuiescent,
    RuntimeIdentityUnavailable,
    SelectedCustodyMismatch,
}

/// The production rejoin step that exhausted the carried resident admission.
/// This diagnostic context grants no media, recovery, or allocation authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryRejoinResidentBoundary {
    RetainedPlanningHandoff,
    FirstSelectedMediaObservation,
    FirstWalAdmission,
    FinalSelectedMediaObservation,
    FinalWalAdmission,
    FingerprintHandoff,
}

/// The allocation site within a complete, independently reread WAL inventory.
/// This describes a denial; it grants no WAL, media, or recovery authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryWalResidentStage {
    FingerprintRoster,
    FrameRosterGrowth,
    EncodedFrameCopy,
    AdmittedFrameRetain,
}

impl RecoveredPhysicalRuntimeCore {
    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.store
    }
    /// Original Store-issued recovery ceiling, preserved across the Serving handoff.
    pub const fn recovery_allocation_admission(
        &self,
    ) -> crate::physical_runtime::PhysicalRecoveryAllocationAdmission {
        self.recovery_allocation
    }
    pub const fn runtime_identity(&self) -> RuntimeIdentity {
        self.runtime
    }
    pub const fn recovery_runtime_identity(&self) -> RuntimeIdentity {
        self.recovery_runtime
    }
    pub const fn root(&self) -> &DurablePhysicalRootManifest {
        &self.root
    }
    pub const fn reopen(&self) -> &CompletedPhysicalRecoveryFreshReopen {
        &self.reopen
    }
    pub fn recovery_effect_count(&self) -> u64 {
        self.media.recovery_effect_count()
    }
    pub const fn backend_profile(
        &self,
    ) -> &worth_store_physical_backend::QualifiedPhysicalBackendProfile {
        self.media.backend_profile()
    }
    pub const fn media_generation(
        &self,
    ) -> worth_store_physical_backend::PhysicalRecoveryMediaGeneration {
        self.media.media_generation()
    }

    /// Consumes the recovered media owner before an ordinary serving media
    /// owner may be qualified. The returned capability cannot be minted from
    /// checkpoint bytes or used twice.
    pub fn into_checkpoint_custody(
        self,
    ) -> Option<crate::physical_runtime::RecoveredPhysicalCheckpointCustody> {
        let Self {
            store,
            recovery_allocation,
            root,
            media,
            reopen,
            checkpoint_custody,
            head_v2_custody,
            no_release_custody,
            pending_wal_release_custody,
            effective_release_heads,
            historical_release_custody,
            tier_custody,
            selected_wal,
            selected_controls,
            ..
        } = self;
        drop(media);
        drop(reopen);
        match (checkpoint_custody, head_v2_custody, no_release_custody, pending_wal_release_custody, historical_release_custody, tier_custody, effective_release_heads) {
            (Some(released), None, None, None, None, None, None) => Some(
                crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified(
                    store, recovery_allocation, root, released, selected_wal?, selected_controls?,
                ),
            ),
            (Some(released), None, None, None, None, Some(tier), None) => Some(
                crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_tier_and_release(
                    store, recovery_allocation, root, released, tier, selected_wal?, selected_controls?,
                ),
            ),
            (None, None, Some(no_release), None, None, tier, None) => Some(
                crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_no_release(
                    store, recovery_allocation, root, no_release, tier, selected_wal?, selected_controls?,
                ),
            ),
            (None, None, None, Some(pending), None, tier, Some(effective)) => Some(
                crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_pending_wal_release(
                    store, recovery_allocation, root, pending, effective, tier, selected_wal?, selected_controls?,
                ),
            ),
            (None, None, None, None, Some(historical), tier, Some(effective)) => Some(
                crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_ordered_historical_release(
                    store, recovery_allocation, root, historical, effective, tier, selected_wal?, selected_controls?,
                ),
            ),
            (None, Some(head_v2), None, None, None, tier, None) => Some(
                crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_head_v2(
                    store, recovery_allocation, root, head_v2, tier, selected_wal?, selected_controls?,
                ),
            ),
            _ => None,
        }
    }
}
