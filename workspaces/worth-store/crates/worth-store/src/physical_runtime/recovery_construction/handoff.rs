use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, DurablePhysicalRootManifest,
};
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
    VerifiedPendingWalReleaseCustody, VerifiedSelectedNoReleaseCustody,
    VerifiedSelectedReleaseHeadCustodyV2, VerifiedSelectedTierEpochCustody,
};

use crate::physical_runtime::{CompletedPhysicalRecoveryFreshReopen, RuntimeIdentity};

pub struct RecoveredPhysicalRuntimeCore {
    pub(super) residency: crate::physical_runtime::instance::PhysicalResidencyOwner,
    pub(super) checkpoint_ownership:
        crate::physical_runtime::recovery_coordination::RecoveryCheckpointOwnership,
    pub(super) store: StableStoreIdentity,
    pub(super) recovery_allocation: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
    pub(super) runtime: RuntimeIdentity,
    pub(super) recovery_runtime: RuntimeIdentity,
    pub(super) root: DurablePhysicalRootManifest,
    pub(super) media: AdmittedRecoveryFilesystemMedia,
    pub(super) reopen: CompletedPhysicalRecoveryFreshReopen,
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
    RejoinWalRead {
        boundary: Option<PhysicalRecoveryRejoinResidentBoundary>,
        cause: crate::physical_runtime::FundedRecoveryWalReadFailure,
    },
    RejoinResident(super::selected_rejoin::PhysicalRecoveryRejoinResidentDenial),
    RejoinWalAdmission {
        boundary: Option<PhysicalRecoveryRejoinResidentBoundary>,
        cause: crate::physical_runtime::RecoveryWalAllocationDenial,
    },
    RejoinBindingSampling {
        boundary: Option<PhysicalRecoveryRejoinResidentBoundary>,
        cause: crate::physical_runtime::StoreRecoveryBindingSampleAllocationDenial,
    },
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
    RejoinRecordReadAllocation {
        artifact: worth_store_physical_backend::RecoveryDiscoveryArtifact,
        offset: u64,
        requested: usize,
        cause: crate::physical_runtime::PhysicalRecoveryObservationAllocationDenial,
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
    pub fn checkpoint(&self) -> Option<&crate::physical_runtime::SharedRecoveryCheckpoint> {
        self.checkpoint_ownership.checkpoint()
    }

    /// Actual retained WAL fingerprint slot capacity. This observes storage,
    /// not pool accounting, and grants no media or allocation authority.
    pub fn selected_wal_owned_heap_bytes(&self) -> Option<u64> {
        self.selected_wal.as_ref()?.owned_heap_bytes()
    }

    /// Actual full-tree witness backing, excluding routing and effect slices.
    #[cfg(feature = "certification-test-authority")]
    pub fn selected_head_walk_owned_heap_bytes(&self) -> Option<u64> {
        self.selected_controls.as_ref()?.head_walk_heap_bytes()
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn selected_head_effect_owned_heap_bytes(&self) -> Option<u64> {
        self.selected_controls.as_ref()?.effect_heap_bytes()
    }

    pub const fn residency_policy(
        &self,
    ) -> crate::physical_runtime::record_serving::AdmittedPhysicalRecordResidencyPolicy {
        self.residency.admitted_policy()
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_residency_allocations(
        &self,
    ) -> worth_store_buffer_pool::PhysicalResidencyAllocationEventObserver {
        self.residency.ports().allocation_events()
    }

    /// Test-only pressure on the carried pool; grants no media or Serving authority.
    #[cfg(feature = "certification-test-authority")]
    pub fn certification_begin_recovery_allocation(
        &self,
        bytes: std::num::NonZeroU64,
    ) -> Result<
        worth_store_buffer_pool::OperationAllocationGrant,
        worth_store_buffer_pool::PhysicalResidencyDenial,
    > {
        self.residency.ports().begin_operation(
            worth_store_buffer_pool::PhysicalOperationAllocationScope::Recovery,
            bytes,
        )
    }

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
            residency,
            checkpoint_ownership,
            store,
            recovery_allocation,
            root,
            media,
            reopen,
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
        checkpoint_ownership.checkpoint()?;
        match (head_v2_custody, no_release_custody, pending_wal_release_custody, historical_release_custody, tier_custody, effective_release_heads) {
            (None, Some(no_release), None, None, tier, None) => Some(
                crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_no_release(
                    residency, checkpoint_ownership, store, recovery_allocation, root, no_release, tier, selected_wal?, selected_controls?,
                ),
            ),
            (None, None, Some(pending), None, tier, Some(effective)) => Some(
                crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_pending_wal_release(
                    residency, checkpoint_ownership, store, recovery_allocation, root, pending, effective, tier, selected_wal?, selected_controls?,
                ),
            ),
            (None, None, None, Some(historical), tier, Some(effective)) => Some(
                crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_ordered_historical_release(
                    residency, checkpoint_ownership, store, recovery_allocation, root, historical, effective, tier, selected_wal?, selected_controls?,
                ),
            ),
            (Some(head_v2), None, None, None, tier, None) => Some(
                crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_head_v2(
                    residency, checkpoint_ownership, store, recovery_allocation, root, head_v2, tier, selected_wal?, selected_controls?,
                ),
            ),
            _ => None,
        }
    }
}
