use worth_store_physical_backend::{
    ArtifactTreeFailure, CompletedArtifactTreePublicationEffect,
    IndeterminateArtifactTreePublicationEffect, RecoveryRootProtocolPublicationPlan,
};
use worth_store_physical_format::RecordArtifactFile;

use super::{
    PerformedRecoveryPhysicalEffect, PhysicalRecoveryCoordination,
    RecoveryRecordNamespaceSynchronizationAction, RecoveryRootProtocolReplacementAction,
};

mod execution;

mod candidate;
pub use candidate::{
    CompletedPhysicalRecoveryPublicationCandidate, PhysicalRecoveryPublicationCandidate,
    PhysicalRecoveryPublicationCandidateMaterialization,
};

pub struct PhysicalRecoveryPublicationCommand {
    plan: [u8; 32],
    staging_generation: u64,
    candidates: Box<[PhysicalRecoveryPublicationCandidate]>,
    protocol: RecoveryRootProtocolPublicationPlan,
}

pub struct CompletedPhysicalRecoveryPublicationCommand {
    candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
    root_protocol: PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>,
    record_namespace: PerformedRecoveryPhysicalEffect<RecoveryRecordNamespaceSynchronizationAction>,
}

pub enum PhysicalRecoveryPublicationCommandOutcome {
    Completed(CompletedPhysicalRecoveryPublicationCommand),
    DeniedBeforeEffect(PhysicalRecoveryPublicationCommandDenial),
    Indeterminate(PhysicalRecoveryPublicationCommandIndeterminate),
}

pub struct PhysicalRecoveryPublicationCommandDenial {
    stage: PhysicalRecoveryPublicationCommandStage,
    denial: PhysicalRecoveryPublicationCommandDenialKind,
    candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
    candidate_materialization: Option<PhysicalRecoveryPublicationCandidateMaterialization>,
    root_protocol: Option<PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>>,
    scheduler: Option<crate::physical_runtime::PhysicalWorkSchedulerPosture>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryPublicationCommandDenialKind {
    Submission,
    PreEffect(crate::physical_runtime::PhysicalWorkPreEffectDenial),
    Scheduler(crate::physical_runtime::PhysicalSchedulerDenial),
    Media(ArtifactTreeFailure),
}

pub enum PhysicalRecoveryPublicationCommandIndeterminate {
    CandidateMaterialization {
        artifact: RecordArtifactFile,
        physical: worth_store_physical_backend::IndeterminateRecoveryStagingWrite,
        completed: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
        scheduler: Option<crate::physical_runtime::PhysicalWorkSchedulerPosture>,
    },
    CandidateSynchronization {
        artifact: RecordArtifactFile,
        physical: IndeterminateArtifactTreePublicationEffect,
        materialization: PhysicalRecoveryPublicationCandidateMaterialization,
        completed: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
        scheduler: Option<crate::physical_runtime::PhysicalWorkSchedulerPosture>,
    },
    CandidateMaterializationSettlement {
        artifact: RecordArtifactFile,
        physical: worth_store_physical_backend::CompletedRecoveryStagingWrite,
        completed: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
        failure: PhysicalRecoveryPublicationSettlementFailure,
    },
    CandidateMaterializationYieldpoint {
        artifact: RecordArtifactFile,
        physical: worth_store_physical_backend::CompletedRecoveryStagingWrite,
        completed: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
        wait: crate::physical_runtime::PhysicalRecoveryYieldpointWaitResult,
    },
    CandidateSynchronizationSettlement {
        artifact: RecordArtifactFile,
        physical: CompletedArtifactTreePublicationEffect,
        materialization: PhysicalRecoveryPublicationCandidateMaterialization,
        completed: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
        failure: PhysicalRecoveryPublicationSettlementFailure,
    },
    CandidateSynchronizationYieldpoint {
        artifact: RecordArtifactFile,
        physical: CompletedArtifactTreePublicationEffect,
        materialization: PhysicalRecoveryPublicationCandidateMaterialization,
        completed: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
        wait: crate::physical_runtime::PhysicalRecoveryYieldpointWaitResult,
    },
    Media {
        stage: PhysicalRecoveryPublicationCommandStage,
        physical: IndeterminateArtifactTreePublicationEffect,
        candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
        root_protocol:
            Option<PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>>,
    },
    Scheduler {
        stage: PhysicalRecoveryPublicationCommandStage,
        physical: CompletedArtifactTreePublicationEffect,
        candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
        root_protocol:
            Option<PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>>,
        posture: crate::physical_runtime::PhysicalWorkSchedulerPosture,
    },
    Signal {
        stage: PhysicalRecoveryPublicationCommandStage,
        physical: CompletedArtifactTreePublicationEffect,
        candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
        root_protocol:
            Option<PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>>,
        outcome: crate::physical_runtime::PhysicalSignalSettlementOutcome,
    },
    Yieldpoint {
        stage: PhysicalRecoveryPublicationCommandStage,
        physical: CompletedArtifactTreePublicationEffect,
        candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
        root_protocol:
            Option<PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>>,
        wait: crate::physical_runtime::PhysicalRecoveryYieldpointWaitResult,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryPublicationSettlementFailure {
    Scheduler(crate::physical_runtime::PhysicalWorkSchedulerPosture),
    Signal(crate::physical_runtime::PhysicalSignalSettlementOutcome),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryPublicationCommandStage {
    CandidateMaterialization,
    CandidateSynchronization,
    RootProtocolReplacement,
    RecordNamespaceSynchronization,
}

impl PhysicalRecoveryPublicationCommand {
    pub fn new(
        plan: [u8; 32],
        staging_generation: u64,
        candidates: Box<[PhysicalRecoveryPublicationCandidate]>,
        protocol: RecoveryRootProtocolPublicationPlan,
    ) -> Option<Self> {
        if staging_generation == 0
            || candidates.is_empty()
            || !candidate::is_complete_and_canonical(&candidates, staging_generation, protocol)
        {
            None
        } else {
            Some(Self {
                plan,
                staging_generation,
                candidates,
                protocol,
            })
        }
    }
}

impl PhysicalRecoveryCoordination {
    pub fn execute_publication_command(
        &self,
        media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
        command: PhysicalRecoveryPublicationCommand,
    ) -> PhysicalRecoveryPublicationCommandOutcome {
        execution::execute(self, media, command)
    }
}

impl CompletedPhysicalRecoveryPublicationCommand {
    pub(super) const fn new(
        candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
        root_protocol: PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>,
        record_namespace: PerformedRecoveryPhysicalEffect<
            RecoveryRecordNamespaceSynchronizationAction,
        >,
    ) -> Self {
        Self {
            candidates,
            root_protocol,
            record_namespace,
        }
    }

    pub fn candidates(&self) -> &[CompletedPhysicalRecoveryPublicationCandidate] {
        &self.candidates
    }

    pub const fn root_protocol(
        &self,
    ) -> &PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction> {
        &self.root_protocol
    }

    pub const fn record_namespace(
        &self,
    ) -> &PerformedRecoveryPhysicalEffect<RecoveryRecordNamespaceSynchronizationAction> {
        &self.record_namespace
    }
}

impl PhysicalRecoveryPublicationCommandDenial {
    pub(super) const fn new(
        stage: PhysicalRecoveryPublicationCommandStage,
        denial: PhysicalRecoveryPublicationCommandDenialKind,
        candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
        candidate_materialization: Option<PhysicalRecoveryPublicationCandidateMaterialization>,
        root_protocol: Option<
            PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>,
        >,
        scheduler: Option<crate::physical_runtime::PhysicalWorkSchedulerPosture>,
    ) -> Self {
        Self {
            stage,
            denial,
            candidates,
            candidate_materialization,
            root_protocol,
            scheduler,
        }
    }

    pub const fn stage(&self) -> PhysicalRecoveryPublicationCommandStage {
        self.stage
    }
    pub const fn denial(&self) -> PhysicalRecoveryPublicationCommandDenialKind {
        self.denial
    }
    pub fn candidates(&self) -> &[CompletedPhysicalRecoveryPublicationCandidate] {
        &self.candidates
    }
    pub const fn candidate_materialization(
        &self,
    ) -> Option<&PhysicalRecoveryPublicationCandidateMaterialization> {
        self.candidate_materialization.as_ref()
    }
    pub const fn root_protocol(
        &self,
    ) -> Option<&PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>> {
        self.root_protocol.as_ref()
    }
    pub const fn scheduler_posture(
        &self,
    ) -> Option<crate::physical_runtime::PhysicalWorkSchedulerPosture> {
        self.scheduler
    }
}
