use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, DurablePhysicalRootManifest,
};
use worth_store_recovery_physics::VerifiedSelectedTierEpochCustody;

use crate::physical_runtime::record_serving::RecoveredReleaseCustody;
use crate::physical_runtime::{CompletedPhysicalRecoveryFreshReopen, RuntimeIdentity};

/// Prepaid one-slot backing for `RecoveredPhysicalRuntimeCore`, reserved by the
/// recovery owner before recovery effects so Store construction never allocates
/// the core body after publication.
pub struct PreparedRecoveredRuntimeCoreBacking {
    inner: Vec<RecoveredPhysicalRuntimeCoreBody>,
}

impl PreparedRecoveredRuntimeCoreBacking {
    pub const fn required_bytes() -> u64 {
        std::mem::size_of::<RecoveredPhysicalRuntimeCoreBody>() as u64
    }

    pub fn try_prepare() -> Result<Self, std::collections::TryReserveError> {
        let mut inner = Vec::new();
        inner.try_reserve_exact(1)?;
        Ok(Self { inner })
    }

    pub fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.inner.capacity())
            .ok()?
            .checked_mul(Self::required_bytes())
    }

    pub(super) fn prepare_direct() -> Self {
        Self {
            inner: Vec::with_capacity(1),
        }
    }

    #[inline(never)]
    pub(super) fn finish(
        mut self,
        body: RecoveredPhysicalRuntimeCoreBody,
    ) -> RecoveredPhysicalRuntimeCore {
        assert!(self.inner.is_empty() && self.inner.capacity() >= 1);
        self.inner.push(body);
        RecoveredPhysicalRuntimeCore { inner: self.inner }
    }
}

pub struct RecoveredPhysicalRuntimeCore {
    inner: Vec<RecoveredPhysicalRuntimeCoreBody>,
}

pub(super) struct RecoveredPhysicalRuntimeCoreBody {
    pub(super) serving_custody_backing:
        crate::physical_runtime::record_serving::PreparedRecoveredCustodyBacking,
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
    pub(super) release_custody: Option<RecoveredReleaseCustody>,
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
    /// The complete WAL inventory passed its budget before its first effect.
    RejoinWalBytes {
        boundary: Option<PhysicalRecoveryRejoinResidentBoundary>,
        cause: super::selected_rejoin::ExceededSelectedWalInventoryBound,
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
    RejoinSelectedMedia(PhysicalRecoverySelectedRejoinMismatch),
    RejoinUnsupportedSelectedControl {
        record: worth_store_physical_format::PersistedRecordIdentity,
        content_class: worth_store_physical_format::SelectedRecordContentClass,
    },
    RejoinQualification(worth_store_physical_backend::RecoveryFilesystemQualificationError),
    RejoinDiscovery(crate::physical_runtime::RecoveryDiscoveryFailure),
    RejoinCanonicalRedo(worth_store_physical_format::CanonicalRedoWireDenial),
    RejoinRecoveryProjection(worth_store_physical_format::PhysicalRecoveryProjectionDenial),
}

/// The independent Store predicate that denied selected custody. This is
/// diagnostic truth only and cannot authorize recovery or Serving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoverySelectedRejoinMismatch {
    MissingSelector,
    MissingRoot,
    MissingCheckpoint,
    MissingRoute,
    MissingFrame,
    RootBinding,
    CheckpointBinding,
    CertificateRoster,
    RoutingFrame,
    UnsupportedSelectedPlacement,
    ControlFrame,
    BoundExceeded,
    WalFate,
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
    pub(super) const fn body(&self) -> &RecoveredPhysicalRuntimeCoreBody {
        &self.inner.as_slice()[0]
    }

    pub(super) fn into_body(mut self) -> RecoveredPhysicalRuntimeCoreBody {
        self.inner
            .pop()
            .expect("recovered physical runtime core holds one body")
    }

    /// Actual capacity of the original-funded serving evidence backing.
    /// This observation grants no allocation or serving authority.
    pub fn serving_evidence_backing_bytes(&self) -> Option<u64> {
        self.body().serving_custody_backing.owned_heap_bytes()
    }

    pub fn checkpoint(&self) -> Option<&crate::physical_runtime::SharedRecoveryCheckpoint> {
        self.body().checkpoint_ownership.checkpoint()
    }

    /// Actual retained WAL fingerprint slot capacity. This observes storage,
    /// not pool accounting, and grants no media or allocation authority.
    pub fn selected_wal_owned_heap_bytes(&self) -> Option<u64> {
        self.body().selected_wal.as_ref()?.owned_heap_bytes()
    }

    /// Actual full-tree witness backing, excluding routing and effect slices.
    #[cfg(feature = "certification-test-authority")]
    pub fn selected_head_walk_owned_heap_bytes(&self) -> Option<u64> {
        self.body().selected_controls.as_ref()?.head_walk_heap_bytes()
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn selected_head_effect_owned_heap_bytes(&self) -> Option<u64> {
        self.body().selected_controls.as_ref()?.effect_heap_bytes()
    }

    pub const fn residency_policy(
        &self,
    ) -> crate::physical_runtime::record_serving::AdmittedPhysicalRecordResidencyPolicy {
        self.body().residency.admitted_policy()
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_residency_allocations(
        &self,
    ) -> worth_store_buffer_pool::PhysicalResidencyAllocationEventObserver {
        self.body().residency.ports().allocation_events()
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
        self.body().residency.ports().begin_operation(
            worth_store_buffer_pool::PhysicalOperationAllocationScope::Recovery,
            bytes,
        )
    }

    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.body().store
    }
    /// Original Store-issued recovery ceiling, preserved across the Serving handoff.
    pub const fn recovery_allocation_admission(
        &self,
    ) -> crate::physical_runtime::PhysicalRecoveryAllocationAdmission {
        self.body().recovery_allocation
    }
    pub const fn runtime_identity(&self) -> RuntimeIdentity {
        self.body().runtime
    }
    pub const fn recovery_runtime_identity(&self) -> RuntimeIdentity {
        self.body().recovery_runtime
    }
    pub const fn root(&self) -> &DurablePhysicalRootManifest {
        &self.body().root
    }
    pub const fn reopen(&self) -> &CompletedPhysicalRecoveryFreshReopen {
        &self.body().reopen
    }
    pub fn recovery_effect_count(&self) -> u64 {
        self.body().media.recovery_effect_count()
    }
    pub const fn backend_profile(
        &self,
    ) -> &worth_store_physical_backend::QualifiedPhysicalBackendProfile {
        self.body().media.backend_profile()
    }
    pub const fn media_generation(
        &self,
    ) -> worth_store_physical_backend::PhysicalRecoveryMediaGeneration {
        self.body().media.media_generation()
    }

    /// Release media for certification that deliberately changes selected
    /// bytes before independently qualifying another owner. Ordinary homes
    /// must use the continuous `into_serving_handoff` transfer.
    #[cfg(feature = "certification-test-authority")]
    pub fn certification_release_media_into_checkpoint_custody(
        self,
    ) -> Option<crate::physical_runtime::RecoveredPhysicalCheckpointCustody> {
        self.into_serving_handoff().map(|handoff| handoff.custody)
    }
}
