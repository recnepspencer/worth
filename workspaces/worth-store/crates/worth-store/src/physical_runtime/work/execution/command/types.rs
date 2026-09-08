use worth_store_buffer_pool::PhysicalWritebackClaim;
use worth_store_physical_format::RecordFrameCoordinate;

use super::super::super::{PhysicalWorkOperationFamily, ResourceAdmittedPhysicalWork};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalExecutorCommandDenial {
    OperationFamilyMismatch,
    ExactCommandRequiresOneRange,
    PayloadLengthMismatch,
    RetryIdentityMismatch,
    RetryRangeMismatch,
    ResidencyRetryRequiresClaim,
    ArtifactCommandRequiresArtifactScope,
    WalAppendCommandRequiresWalScope,
    WalSegmentCreateCommandRequiresWalScope,
    WalBarrierCommandRequiresWalScope,
    CheckpointCommandRequiresCheckpointScope,
    CheckpointPayloadPostureMismatch,
    WalReclamationCommandRequiresWalScope,
    RootPublicationCommandRequiresRootScope,
}

pub enum PhysicalExecutorCommand {
    Inspection(PhysicalInspectionExecutorCommand),
    Metadata(PhysicalMetadataExecutorCommand),
    Read(PhysicalReadExecutorCommand),
    ExactWrite(PhysicalWriteExecutorCommand),
    Publication(PhysicalWriteExecutorCommand),
    NewArtifact(PhysicalWriteExecutorCommand),
    PublicationEffect(PhysicalPublicationExecutorCommand),
    RootPublicationEffect(PhysicalPublicationExecutorCommand),
    ResidencyWriteback(PhysicalResidencyWritebackExecutorCommand),
    WalSegmentCreate(PhysicalWalSegmentCreateExecutorCommand),
    WalAppend(PhysicalWalAppendExecutorCommand),
    WalBarrier(PhysicalWalBarrierExecutorCommand),
    Checkpoint(PhysicalCheckpointExecutorCommand),
    WalReclamation(PhysicalWalReclamationExecutorCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalPublicationEffect {
    SynchronizeArtifact,
    SynchronizeArtifactParent,
    ReplaceCatalog,
    SynchronizeRecordFamily,
}

pub struct PhysicalRetryCommand {
    identity: super::super::super::PhysicalWorkIdentity,
    payload: PhysicalRetryPayload,
}

pub(in crate::physical_runtime) enum PhysicalRetryPayload {
    Metadata,
    Read,
    ExactWrite(Box<[u8]>),
    Publication(Box<[u8]>),
    NewArtifact(Box<[u8]>),
    PublicationEffect(PhysicalPublicationEffect),
    RootPublicationEffect,
    ResidencyWriteback,
    WalSegmentCreate {
        artifact: worth_store_physical_backend::ArtifactTreeFile,
        range: worth_store_physical_backend::ArtifactNewWriteRange,
        payload: Box<[u8]>,
    },
    WalAppend {
        artifact: worth_store_physical_backend::ArtifactTreeFile,
        range: worth_store_physical_backend::ArtifactAppendRange,
        payload: Box<[u8]>,
    },
    WalBarrier {
        artifact: worth_store_physical_backend::ArtifactTreeFile,
        binding_digest: [u8; 32],
    },
    Checkpoint {
        payload: Option<Box<[u8]>>,
    },
    WalReclamation,
}

pub struct PhysicalMetadataExecutorCommand {
    pub(in crate::physical_runtime) work: ResourceAdmittedPhysicalWork,
    pub(in crate::physical_runtime) artifact: worth_store_physical_format::RecordArtifactFile,
}

pub struct PhysicalReadExecutorCommand {
    pub(in crate::physical_runtime) work: ResourceAdmittedPhysicalWork,
    pub(in crate::physical_runtime) coordinate: RecordFrameCoordinate,
    pub(in crate::physical_runtime) destination: Box<[u8]>,
}

pub struct PhysicalInspectionExecutorCommand {
    pub(in crate::physical_runtime) work: ResourceAdmittedPhysicalWork,
    pub(in crate::physical_runtime) range: worth_store_physical_format::PhysicalArtifactReadRange,
    pub(in crate::physical_runtime) destination: Box<[u8]>,
}

pub struct PhysicalWriteExecutorCommand {
    pub(in crate::physical_runtime) work: ResourceAdmittedPhysicalWork,
    pub(in crate::physical_runtime) coordinate: RecordFrameCoordinate,
    pub(in crate::physical_runtime) payload: Box<[u8]>,
    pub(in crate::physical_runtime) payload_digest: [u8; 32],
}

pub struct PhysicalResidencyWritebackExecutorCommand {
    pub(in crate::physical_runtime) work: ResourceAdmittedPhysicalWork,
    pub(in crate::physical_runtime) claim: PhysicalWritebackClaim,
}

pub struct PhysicalPublicationExecutorCommand {
    pub(in crate::physical_runtime) work: ResourceAdmittedPhysicalWork,
    pub(in crate::physical_runtime) artifact: worth_store_physical_format::RecordArtifactFile,
    pub(in crate::physical_runtime) effect: PhysicalPublicationEffect,
}

pub struct PhysicalWalSegmentCreateExecutorCommand {
    pub(in crate::physical_runtime) work: ResourceAdmittedPhysicalWork,
    pub(in crate::physical_runtime) artifact: worth_store_physical_backend::ArtifactTreeFile,
    pub(in crate::physical_runtime) range: worth_store_physical_backend::ArtifactNewWriteRange,
    pub(in crate::physical_runtime) payload: Box<[u8]>,
    pub(in crate::physical_runtime) payload_digest: [u8; 32],
}

pub struct PhysicalWalAppendExecutorCommand {
    pub(in crate::physical_runtime) work: ResourceAdmittedPhysicalWork,
    pub(in crate::physical_runtime) artifact: worth_store_physical_backend::ArtifactTreeFile,
    pub(in crate::physical_runtime) range: worth_store_physical_backend::ArtifactAppendRange,
    pub(in crate::physical_runtime) payload: Box<[u8]>,
    pub(in crate::physical_runtime) payload_digest: [u8; 32],
}

pub struct PhysicalWalBarrierExecutorCommand {
    pub(in crate::physical_runtime) work: ResourceAdmittedPhysicalWork,
    pub(in crate::physical_runtime) artifact: worth_store_physical_backend::ArtifactTreeFile,
    pub(in crate::physical_runtime) binding_digest: [u8; 32],
}

pub struct PhysicalCheckpointExecutorCommand {
    pub(in crate::physical_runtime) work: ResourceAdmittedPhysicalWork,
    pub(in crate::physical_runtime) payload: Option<Box<[u8]>>,
    pub(in crate::physical_runtime) payload_digest: Option<[u8; 32]>,
}

pub struct PhysicalWalReclamationExecutorCommand {
    pub(in crate::physical_runtime) work: ResourceAdmittedPhysicalWork,
}

impl PhysicalExecutorCommand {
    pub const fn identity(&self) -> super::super::super::PhysicalWorkIdentity {
        self.intent().identity()
    }

    pub(in crate::physical_runtime) const fn intent(
        &self,
    ) -> &super::super::super::PhysicalWorkIntent {
        match self {
            Self::Inspection(command) => command.work.intent(),
            Self::Metadata(command) => command.work.intent(),
            Self::Read(command) => command.work.intent(),
            Self::ExactWrite(command) | Self::Publication(command) => command.work.intent(),
            Self::NewArtifact(command) => command.work.intent(),
            Self::PublicationEffect(command) => command.work.intent(),
            Self::RootPublicationEffect(command) => command.work.intent(),
            Self::ResidencyWriteback(command) => command.work.intent(),
            Self::WalSegmentCreate(command) => command.work.intent(),
            Self::WalAppend(command) => command.work.intent(),
            Self::WalBarrier(command) => command.work.intent(),
            Self::Checkpoint(command) => command.work.intent(),
            Self::WalReclamation(command) => command.work.intent(),
        }
    }

    pub(in crate::physical_runtime) fn is_cancelled(&self) -> bool {
        match self {
            Self::Inspection(command) => command.work.is_cancelled(),
            Self::Metadata(command) => command.work.is_cancelled(),
            Self::Read(command) => command.work.is_cancelled(),
            Self::ExactWrite(command) | Self::Publication(command) => command.work.is_cancelled(),
            Self::NewArtifact(command) => command.work.is_cancelled(),
            Self::PublicationEffect(command) => command.work.is_cancelled(),
            Self::RootPublicationEffect(command) => command.work.is_cancelled(),
            Self::ResidencyWriteback(command) => command.work.is_cancelled(),
            Self::WalSegmentCreate(command) => command.work.is_cancelled(),
            Self::WalAppend(command) => command.work.is_cancelled(),
            Self::WalBarrier(command) => command.work.is_cancelled(),
            Self::Checkpoint(command) => command.work.is_cancelled(),
            Self::WalReclamation(command) => command.work.is_cancelled(),
        }
    }

    pub(in crate::physical_runtime) const fn wal_barrier_completion_binding(
        &self,
    ) -> Option<(super::super::super::PhysicalWalBarrierScope, [u8; 32])> {
        match self {
            Self::WalBarrier(command) => Some((
                command
                    .work
                    .intent()
                    .scope()
                    .wal_barrier_target()
                    .expect("WAL barrier commands retain exact WAL barrier scope"),
                command.binding_digest,
            )),
            _ => None,
        }
    }

    pub(in crate::physical_runtime) fn residency_writeback(
        work: ResourceAdmittedPhysicalWork,
        claim: PhysicalWritebackClaim,
    ) -> Self {
        Self::ResidencyWriteback(PhysicalResidencyWritebackExecutorCommand { work, claim })
    }
}

impl PhysicalRetryCommand {
    pub(in crate::physical_runtime) const fn new(
        identity: super::super::super::PhysicalWorkIdentity,
        payload: PhysicalRetryPayload,
    ) -> Self {
        Self { identity, payload }
    }

    pub const fn identity(&self) -> super::super::super::PhysicalWorkIdentity {
        self.identity
    }

    pub fn bind(
        self,
        work: ResourceAdmittedPhysicalWork,
    ) -> Result<PhysicalExecutorCommand, PhysicalExecutorCommandDenial> {
        if work.intent().identity() != self.identity {
            return Err(PhysicalExecutorCommandDenial::RetryIdentityMismatch);
        }
        match self.payload {
            PhysicalRetryPayload::Metadata => PhysicalExecutorCommand::metadata(work),
            PhysicalRetryPayload::Read => PhysicalExecutorCommand::read(work),
            PhysicalRetryPayload::ExactWrite(payload) => {
                PhysicalExecutorCommand::exact_write(work, payload)
            }
            PhysicalRetryPayload::Publication(payload) => {
                PhysicalExecutorCommand::publication(work, payload)
            }
            PhysicalRetryPayload::NewArtifact(payload) => {
                PhysicalExecutorCommand::new_artifact(work, payload)
            }
            PhysicalRetryPayload::PublicationEffect(effect) => {
                PhysicalExecutorCommand::publication_effect(work, effect)
            }
            PhysicalRetryPayload::RootPublicationEffect => {
                PhysicalExecutorCommand::root_publication_effect(work)
            }
            PhysicalRetryPayload::ResidencyWriteback => {
                Err(PhysicalExecutorCommandDenial::ResidencyRetryRequiresClaim)
            }
            PhysicalRetryPayload::WalSegmentCreate {
                artifact,
                range,
                payload,
            } => PhysicalExecutorCommand::retry_wal_segment_create(work, artifact, range, payload),
            PhysicalRetryPayload::WalAppend {
                artifact,
                range,
                payload,
            } => PhysicalExecutorCommand::retry_wal_append(work, artifact, range, payload),
            PhysicalRetryPayload::WalBarrier {
                artifact,
                binding_digest,
            } => PhysicalExecutorCommand::wal_barrier(work, artifact, binding_digest),
            PhysicalRetryPayload::Checkpoint { payload } => {
                PhysicalExecutorCommand::checkpoint(work, payload)
            }
            PhysicalRetryPayload::WalReclamation => PhysicalExecutorCommand::wal_reclamation(work),
        }
    }

    pub(in crate::physical_runtime) fn admit_residency_retry(
        self,
        work: &ResourceAdmittedPhysicalWork,
    ) -> Result<(), PhysicalExecutorCommandDenial> {
        if work.intent().identity() != self.identity {
            return Err(PhysicalExecutorCommandDenial::RetryIdentityMismatch);
        }
        matches!(self.payload, PhysicalRetryPayload::ResidencyWriteback)
            .then_some(())
            .ok_or(PhysicalExecutorCommandDenial::ResidencyRetryRequiresClaim)
    }
}

pub(super) fn require_family(
    work: &ResourceAdmittedPhysicalWork,
    expected: PhysicalWorkOperationFamily,
) -> Result<(), PhysicalExecutorCommandDenial> {
    (work.intent().operation() == expected)
        .then_some(())
        .ok_or(PhysicalExecutorCommandDenial::OperationFamilyMismatch)
}
