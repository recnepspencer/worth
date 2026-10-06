use worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity;
use worth_store::physical_runtime::{ArtifactTreeFailureKind, RecoveryDiscoveryArtifact};
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, ManifestBlockReference, RootSelectorRole,
};
use worth_store_physical_integrity::PhysicalIntegrityRejection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryRootProtocolArtifact {
    BootstrapCatalog,
    CurrentSelector,
    PreviousSelector,
    StagedCurrentSelector { publication: u64 },
    CurrentRoot { generation: u64 },
    PreviousRoot { generation: u64 },
    CheckpointSourceRoot { generation: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryRootProtocolDenial {
    Absent,
    ConflictingDuplication { observed_sources: u64 },
    Integrity(PhysicalIntegrityRejection),
    NonCanonicalEncoding,
    ScopeMismatch,
    SourceIncarnationMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryCheckpointIntegrityDenial {
    AllocationRejected,
    Integrity(PhysicalIntegrityRejection),
    /// The block's cause names the limit and its counts.
    DirtyRecordLimit,
    /// The block's cause names the limit and its counts.
    BindingRecordLimit,
    NonCanonicalEncoding,
    ScopeMismatch,
    SourceIncarnationMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalManifestObservationDenial {
    DuplicateReference {
        reference: ManifestBlockReference,
    },
    MissingArtifact {
        reference: ManifestBlockReference,
    },
    Integrity {
        reference: ManifestBlockReference,
        denial: PhysicalRecoveryRootProtocolDenial,
    },
    /// The routing tree's leaves hold more entries than its verified
    /// manifest's record count. `admitted` is that count; recovery sets no
    /// such limit.
    RecordCountCeiling {
        observed: u64,
        admitted: u64,
    },
}
use worth_store_recovery_physics::{
    PhysicalCheckpointBaseDenial, PhysicalPageFactDenial, PhysicalRootCandidateDenial,
    PhysicalRootSelectionDenial, PhysicalSourceSelectionDenial, SelectedPhysicalWalTailDenial,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoverySourceReadAllocationBoundary {
    WindowAdmission,
    ReadBuffer,
    QualifiedPath(worth_store::physical_runtime::ArtifactTreePathAllocationBoundary),
    ParserRecords,
    BindingDecode,
    BindingBasis,
    CanonicalValidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoverySourceReadAllocationDenial {
    Admission(worth_store::physical_runtime::PhysicalRecoveryRejoinResidentAdmissionDenial),
    Residency(worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial),
    Observation(worth_store::physical_runtime::PhysicalRecoveryObservationAllocationDenial),
    BindingDecode(worth_store::physical_runtime::StoreRecoveryCheckpointBindingAllocationDenial),
    BindingBasis(worth_store::physical_runtime::StoreRecoveryCheckpointBindingAllocationDenial),
    AllocatorExceededReservation { requested: u64, actual: u64 },
    ReadBufferLengthMismatch { requested: usize, observed: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryWalInventoryAllocationBoundary {
    CanonicalOrdering,
    AdmittedSegments,
    IntegrityObservations,
    CandidateRoster,
    CandidateFrameFacts,
    TailPartition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoverySourceDenial {
    WalRead {
        failure: worth_store::physical_runtime::FundedRecoveryWalReadFailure,
    },
    WalReadAllocation {
        artifact: RecoveryDiscoveryArtifact,
        boundary: PhysicalRecoverySourceReadAllocationBoundary,
        requested: u64,
        cause: PhysicalRecoverySourceReadAllocationDenial,
    },
    WalAdmissionAllocation {
        cause: worth_store::physical_runtime::RecoveryWalAllocationDenial,
    },
    WalInventoryAllocation {
        boundary: PhysicalRecoveryWalInventoryAllocationBoundary,
        cause: worth_store::physical_runtime::RecoveryWalAllocationDenial,
    },
    CheckpointReadAllocation {
        artifact: RecoveryDiscoveryArtifact,
        boundary: PhysicalRecoverySourceReadAllocationBoundary,
        requested: u64,
        cause: PhysicalRecoverySourceReadAllocationDenial,
    },
    SourceReadAllocation {
        artifact: PhysicalRecoveryRootProtocolArtifact,
        boundary: PhysicalRecoverySourceReadAllocationBoundary,
        requested: u64,
        cause: PhysicalRecoverySourceReadAllocationDenial,
    },
    MediaObservation {
        artifact: RecoveryDiscoveryArtifact,
        failure: PhysicalRecoveryMediaObservationFailure,
    },
    RootSlot {
        slot: RootSelectorRole,
        denial: PhysicalRootCandidateDenial,
        observed_store: Option<StableStoreIdentity>,
        observed_role: Option<RootSelectorRole>,
        observed_generation: Option<u64>,
    },
    RootProtocol {
        artifact: PhysicalRecoveryRootProtocolArtifact,
        denial: PhysicalRecoveryRootProtocolDenial,
    },
    RootSelection(PhysicalRootSelectionDenial),
    ManifestObservation(PhysicalManifestObservationDenial),
    ManifestFacts(PhysicalPageFactDenial),
    CheckpointIntegrity(PhysicalRecoveryCheckpointIntegrityDenial),
    CheckpointBinding(PhysicalCheckpointBaseDenial),
    CheckpointBacking(worth_store::physical_runtime::SharedCheckpointAdmissionDenial),
    CheckpointInstallation(worth_store::physical_runtime::SelectedCheckpointInstallationDenial),
    WalIntegrity(PhysicalRecoveryWalIntegrityDenial),
    WalTail(SelectedPhysicalWalTailDenial),
    FinalSelection(PhysicalSourceSelectionDenial),
}

impl PhysicalRecoverySourceDenial {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        match self {
            // The shared Store diagnostic independently retains its native backing.
            Self::WalRead { .. } => Some(0),
            Self::WalIntegrity(denial) => u64::try_from(denial.artifact.capacity()).ok(),
            Self::MediaObservation { .. }
            | Self::CheckpointReadAllocation { .. }
            | Self::WalReadAllocation { .. }
            | Self::WalAdmissionAllocation { .. }
            | Self::WalInventoryAllocation { .. }
            | Self::SourceReadAllocation { .. }
            | Self::RootSlot { .. }
            | Self::RootProtocol { .. }
            | Self::RootSelection(_)
            | Self::ManifestObservation(_)
            | Self::ManifestFacts(_)
            | Self::CheckpointIntegrity(_)
            | Self::CheckpointBinding(_)
            | Self::CheckpointBacking(_)
            | Self::CheckpointInstallation(_)
            | Self::WalTail(_)
            | Self::FinalSelection(_) => Some(0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRecoveryWalIntegrityDenial {
    artifact: String,
    identity: WalSegmentArtifactIdentity,
    rejection: PhysicalIntegrityRejection,
}

impl PhysicalRecoveryWalIntegrityDenial {
    pub(crate) fn new(
        artifact: String,
        identity: WalSegmentArtifactIdentity,
        rejection: PhysicalIntegrityRejection,
    ) -> Self {
        Self {
            artifact,
            identity,
            rejection,
        }
    }

    pub fn artifact(&self) -> &str {
        &self.artifact
    }
    pub const fn identity(&self) -> WalSegmentArtifactIdentity {
        self.identity
    }
    pub const fn rejection(&self) -> PhysicalIntegrityRejection {
        self.rejection
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryMediaObservationFailure {
    InvalidAddress,
    Backend {
        kind: ArtifactTreeFailureKind,
        io_kind: Option<std::io::ErrorKind>,
    },
}
