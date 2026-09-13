use sha2::Digest;
use worth_store_physical_backend::{
    ArtifactTreeFailure, CompletedArtifactTreePublicationEffect, CompletedRecoveryStagingWrite,
    IndeterminateArtifactTreePublicationEffect, IndeterminateRecoveryStagingWrite,
};
use worth_store_physical_format::RecordArtifactFile;

use super::{
    PerformedRecoveryPhysicalEffect, PhysicalRecoveryCoordination,
    RecoveryStagingSynchronizationAction, RecoveryStagingWriteAction,
};

mod execution;

pub struct PhysicalRecoveryStagingCommand<'bytes> {
    ordinal: u64,
    plan: [u8; 32],
    staging_generation: u64,
    artifact: RecordArtifactFile,
    bytes: &'bytes [u8],
    payload_digest: [u8; 32],
}

pub enum PhysicalRecoveryStagingMaterialization {
    Created(PerformedRecoveryPhysicalEffect<RecoveryStagingWriteAction>),
    AlreadyMaterialized(CompletedRecoveryStagingWrite),
    CompletedFromExactPrefix(PerformedRecoveryPhysicalEffect<RecoveryStagingWriteAction>),
}

pub enum PhysicalRecoveryStagingMaterializationEvidence {
    Performed(PhysicalRecoveryStagingMaterialization),
    PhysicallyCompleted(CompletedRecoveryStagingWrite),
}

pub struct CompletedPhysicalRecoveryStagingCommand {
    materialization: PhysicalRecoveryStagingMaterialization,
    synchronization: PerformedRecoveryPhysicalEffect<RecoveryStagingSynchronizationAction>,
}

pub enum PhysicalRecoveryStagingCommandOutcome {
    Completed(CompletedPhysicalRecoveryStagingCommand),
    DeniedBeforeEffect(PhysicalRecoveryStagingCommandDenial),
    Indeterminate(PhysicalRecoveryStagingCommandIndeterminate),
}

pub struct PhysicalRecoveryStagingCommandDenial {
    stage: PhysicalRecoveryStagingCommandStage,
    denial: PhysicalRecoveryStagingCommandDenialKind,
    materialization: Option<PhysicalRecoveryStagingMaterialization>,
    scheduler: Option<crate::physical_runtime::PhysicalWorkSchedulerPosture>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryStagingCommandDenialKind {
    Submission,
    PreEffect(crate::physical_runtime::PhysicalWorkPreEffectDenial),
    Scheduler(crate::physical_runtime::PhysicalSchedulerDenial),
    Media(ArtifactTreeFailure),
}

pub enum PhysicalRecoveryStagingCommandIndeterminate {
    Materialization {
        physical: IndeterminateRecoveryStagingWrite,
        scheduler: Option<crate::physical_runtime::PhysicalWorkSchedulerPosture>,
    },
    Synchronization {
        physical: IndeterminateArtifactTreePublicationEffect,
        materialization: PhysicalRecoveryStagingMaterialization,
        scheduler: Option<crate::physical_runtime::PhysicalWorkSchedulerPosture>,
    },
    Scheduler {
        stage: PhysicalRecoveryStagingCommandStage,
        materialization: Option<PhysicalRecoveryStagingMaterializationEvidence>,
        synchronization: Option<CompletedArtifactTreePublicationEffect>,
        posture: crate::physical_runtime::PhysicalWorkSchedulerPosture,
    },
    Signal {
        stage: PhysicalRecoveryStagingCommandStage,
        materialization: Option<PhysicalRecoveryStagingMaterializationEvidence>,
        synchronization: Option<CompletedArtifactTreePublicationEffect>,
        outcome: crate::physical_runtime::PhysicalSignalSettlementOutcome,
    },
    Yieldpoint {
        stage: PhysicalRecoveryStagingCommandStage,
        materialization: Option<PhysicalRecoveryStagingMaterializationEvidence>,
        synchronization: Option<CompletedArtifactTreePublicationEffect>,
        wait: crate::physical_runtime::PhysicalRecoveryYieldpointWaitResult,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryStagingCommandStage {
    Materialization,
    Synchronization,
}

impl<'bytes> PhysicalRecoveryStagingCommand<'bytes> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ordinal: u64,
        plan: [u8; 32],
        staging_generation: u64,
        artifact: RecordArtifactFile,
        bytes: &'bytes [u8],
        payload_digest: [u8; 32],
    ) -> Option<Self> {
        (!bytes.is_empty()
            && staging_generation != 0
            && sha2::Sha256::digest(bytes).as_slice() == payload_digest)
            .then_some(Self {
                ordinal,
                plan,
                staging_generation,
                artifact,
                bytes,
                payload_digest,
            })
    }
}

impl PhysicalRecoveryCoordination {
    pub fn execute_staging_command(
        &self,
        media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
        command: PhysicalRecoveryStagingCommand<'_>,
    ) -> PhysicalRecoveryStagingCommandOutcome {
        execution::execute(self, media, command)
    }
}

impl CompletedPhysicalRecoveryStagingCommand {
    pub(super) const fn new(
        materialization: PhysicalRecoveryStagingMaterialization,
        synchronization: PerformedRecoveryPhysicalEffect<RecoveryStagingSynchronizationAction>,
    ) -> Self {
        Self {
            materialization,
            synchronization,
        }
    }
    pub const fn materialization(&self) -> &PhysicalRecoveryStagingMaterialization {
        &self.materialization
    }
    pub const fn synchronization(
        &self,
    ) -> &PerformedRecoveryPhysicalEffect<RecoveryStagingSynchronizationAction> {
        &self.synchronization
    }
}

impl PhysicalRecoveryStagingCommandDenial {
    pub(super) const fn new(
        stage: PhysicalRecoveryStagingCommandStage,
        denial: PhysicalRecoveryStagingCommandDenialKind,
        materialization: Option<PhysicalRecoveryStagingMaterialization>,
        scheduler: Option<crate::physical_runtime::PhysicalWorkSchedulerPosture>,
    ) -> Self {
        Self {
            stage,
            denial,
            materialization,
            scheduler,
        }
    }
    pub const fn stage(&self) -> PhysicalRecoveryStagingCommandStage {
        self.stage
    }
    pub const fn denial(&self) -> PhysicalRecoveryStagingCommandDenialKind {
        self.denial
    }
    pub const fn materialization(&self) -> Option<&PhysicalRecoveryStagingMaterialization> {
        self.materialization.as_ref()
    }
    pub const fn scheduler_posture(
        &self,
    ) -> Option<crate::physical_runtime::PhysicalWorkSchedulerPosture> {
        self.scheduler
    }
}

impl PhysicalRecoveryStagingCommandIndeterminate {
    pub const fn scheduler_posture(
        &self,
    ) -> Option<crate::physical_runtime::PhysicalWorkSchedulerPosture> {
        match self {
            Self::Materialization { scheduler, .. } | Self::Synchronization { scheduler, .. } => {
                *scheduler
            }
            Self::Scheduler { posture, .. } => Some(*posture),
            Self::Signal { .. } | Self::Yieldpoint { .. } => {
                Some(crate::physical_runtime::PhysicalWorkSchedulerPosture::Executed)
            }
        }
    }
}

impl PhysicalRecoveryStagingMaterialization {
    pub fn physical(&self) -> &CompletedRecoveryStagingWrite {
        match self {
            Self::Created(performed) | Self::CompletedFromExactPrefix(performed) => {
                match performed.occurrence() {
                    super::RecoveryPhysicalEffectOccurrence::StagingWrite(occurrence) => {
                        occurrence.physical()
                    }
                    _ => unreachable!("staging-write evidence has its exact action"),
                }
            }
            Self::AlreadyMaterialized(physical) => physical,
        }
    }
}

impl PhysicalRecoveryStagingMaterializationEvidence {
    pub fn physical(&self) -> &CompletedRecoveryStagingWrite {
        match self {
            Self::Performed(materialization) => materialization.physical(),
            Self::PhysicallyCompleted(physical) => physical,
        }
    }

    pub const fn is_performed(&self) -> bool {
        matches!(self, Self::Performed(_))
    }
}
