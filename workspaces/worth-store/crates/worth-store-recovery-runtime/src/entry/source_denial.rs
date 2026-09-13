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
    DirtyRecordLimit { observed: u64, admitted: u64 },
    BindingRecordLimit { observed: u64, admitted: u64 },
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
}
use worth_store_recovery_physics::{
    PhysicalCheckpointBaseDenial, PhysicalPageFactDenial, PhysicalRootCandidateDenial,
    PhysicalRootSelectionDenial, PhysicalSourceSelectionDenial, SelectedPhysicalWalTailDenial,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoverySourceDenial {
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
    WalIntegrity(PhysicalRecoveryWalIntegrityDenial),
    WalTail(SelectedPhysicalWalTailDenial),
    FinalSelection(PhysicalSourceSelectionDenial),
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
