use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, RecordArtifactFile, RecordFrameCoordinate,
};

use super::super::{PhysicalWorkOperationFamily, PhysicalWorkRecoveryDisposition};
use super::format_mapping::{operation_from_format, target_from_format};
use super::integrity_admission::IntegrityAdmittedPhysicalWorkProjection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkRecoveryTarget {
    Range(RecordFrameCoordinate),
    WalArtifactInterval {
        segment: u64,
        generation: u64,
        offset: u64,
        byte_count: u64,
    },
    Checkpoint {
        sequence: u64,
        action: PhysicalCheckpointRecoveryAction,
    },
    WalSegmentReclamation {
        segment: u64,
        generation: u64,
    },
    ArtifactFileSynchronization(RecordArtifactFile),
    ArtifactParentSynchronization(RecordArtifactFile),
    CatalogReplacement(RecordArtifactFile),
    RecordNamespaceSynchronization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalWorkRecoveryLocator {
    store: StableStoreIdentity,
    runtime: u64,
    generation: u64,
    operation: u64,
    family: PhysicalWorkOperationFamily,
    target: PhysicalWorkRecoveryTarget,
    payload_digest: Option<[u8; 32]>,
    recovery: PhysicalWorkRecoveryDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalCheckpointRecoveryAction {
    CreateCandidate { byte_count: u64 },
    AppendCandidate { offset: u64, byte_count: u64 },
    SynchronizeCandidate,
    RemoveCandidate,
    PublishCandidate,
    SynchronizeNamespace,
}

impl From<super::super::PhysicalCheckpointWorkAction> for PhysicalCheckpointRecoveryAction {
    fn from(action: super::super::PhysicalCheckpointWorkAction) -> Self {
        match action {
            super::super::PhysicalCheckpointWorkAction::CreateCandidate { byte_count } => {
                Self::CreateCandidate { byte_count }
            }
            super::super::PhysicalCheckpointWorkAction::AppendCandidate { offset, byte_count } => {
                Self::AppendCandidate { offset, byte_count }
            }
            super::super::PhysicalCheckpointWorkAction::SynchronizeCandidate => {
                Self::SynchronizeCandidate
            }
            super::super::PhysicalCheckpointWorkAction::RemoveCandidate => Self::RemoveCandidate,
            super::super::PhysicalCheckpointWorkAction::PublishCandidate => Self::PublishCandidate,
            super::super::PhysicalCheckpointWorkAction::SynchronizeNamespace => {
                Self::SynchronizeNamespace
            }
        }
    }
}

impl PhysicalWorkRecoveryLocator {
    pub(super) fn from_integrity_admitted(
        admitted: IntegrityAdmittedPhysicalWorkProjection,
    ) -> Option<Self> {
        let family = operation_from_format(admitted.operation());
        if matches!(
            family,
            PhysicalWorkOperationFamily::ArtifactMetadataRead
                | PhysicalWorkOperationFamily::ArtifactRangeRead
        ) {
            return None;
        }
        let identity = admitted.identity();
        Some(Self {
            store: admitted.scope().store_identity(),
            runtime: identity.runtime().get(),
            generation: identity.generation().get(),
            operation: identity.operation().get(),
            family,
            target: target_from_format(admitted.target())?,
            payload_digest: admitted.payload_digest(),
            recovery: PhysicalWorkRecoveryDisposition::InspectionRequired,
        })
    }

    pub const fn store(self) -> StableStoreIdentity {
        self.store
    }

    pub const fn runtime(self) -> u64 {
        self.runtime
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn operation(self) -> u64 {
        self.operation
    }

    pub const fn family(self) -> PhysicalWorkOperationFamily {
        self.family
    }

    pub const fn target(self) -> PhysicalWorkRecoveryTarget {
        self.target
    }

    pub const fn coordinate(self) -> Option<RecordFrameCoordinate> {
        match self.target {
            PhysicalWorkRecoveryTarget::Range(coordinate) => Some(coordinate),
            _ => None,
        }
    }

    pub const fn payload_digest(self) -> Option<[u8; 32]> {
        self.payload_digest
    }

    pub const fn recovery_disposition(self) -> PhysicalWorkRecoveryDisposition {
        self.recovery
    }
}
