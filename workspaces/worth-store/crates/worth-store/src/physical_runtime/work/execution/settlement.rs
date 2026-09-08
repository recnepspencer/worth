use worth_store_io_scheduler::QueueExecutionOutcome;
#[cfg(feature = "recovery-runtime-owner")]
use worth_store_physical_backend::CompletedRecoveryStagingWrite;
use worth_store_physical_backend::{
    ArtifactTreeFailure, CompletedArtifactAppend, CompletedArtifactMetadataRead,
    CompletedArtifactNewWrite, CompletedArtifactRangeRead, CompletedArtifactRangeWrite,
    MediaOperationRole,
};

use super::super::{
    PhysicalWorkRecoveryDisposition, PhysicalWorkRecoveryTarget, SettledPhysicalWork,
};
use super::CompletedPhysicalCheckpointAction;
use super::{
    CompletedPhysicalPublicationEffect, CompletedPhysicalWalBarrier,
    CompletedPhysicalWalReclamationAction,
};

mod backend_role;
mod classification;
mod durability;
mod result;

pub(in crate::physical_runtime::work) use durability::durability_satisfies;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkEffectFate {
    ProvenNoEffect,
    ReadCompleted,
    ReadIncomplete,
    WriteCompleted,
    PublicationCompleted,
    CheckpointCompleted,
    WalReclamationCompleted,
    WrittenButSchedulerRejected,
    Indeterminate,
    StaleOrForeignOutcome,
}

pub enum PhysicalWorkSettlementEvidence {
    /// Diagnostic acquisition only: never revokes serving health or grants repair.
    Inspection {
        physical: worth_store_physical_backend::ObservedArtifactInspectionRead,
        bytes: Box<[u8]>,
        scheduler: QueueExecutionOutcome,
    },
    InspectionDenied(ArtifactTreeFailure),
    NoEffect(PhysicalWorkNoEffectEvidence),
    Metadata {
        physical: CompletedArtifactMetadataRead,
        scheduler: QueueExecutionOutcome,
    },
    Read {
        physical: CompletedArtifactRangeRead,
        bytes: Box<[u8]>,
        scheduler: QueueExecutionOutcome,
    },
    Write {
        physical: CompletedArtifactRangeWrite,
        scheduler: QueueExecutionOutcome,
    },
    Publication {
        physical: CompletedArtifactRangeWrite,
        scheduler: QueueExecutionOutcome,
    },
    NewArtifact {
        physical: CompletedArtifactNewWrite,
        coordinate: worth_store_physical_format::RecordFrameCoordinate,
        scheduler: QueueExecutionOutcome,
    },
    #[cfg(feature = "recovery-runtime-owner")]
    RecoveryStaging {
        physical: CompletedRecoveryStagingWrite,
        scheduler: QueueExecutionOutcome,
    },
    PublicationEffect {
        physical: CompletedPhysicalPublicationEffect,
        scheduler: QueueExecutionOutcome,
    },
    WalAppend {
        physical: CompletedArtifactAppend,
        scheduler: QueueExecutionOutcome,
    },
    WalSegmentCreate {
        physical: CompletedArtifactNewWrite,
        scheduler: QueueExecutionOutcome,
    },
    WalBarrier {
        physical: CompletedPhysicalWalBarrier,
        scheduler: QueueExecutionOutcome,
    },
    Checkpoint {
        physical: CompletedPhysicalCheckpointAction,
        scheduler: QueueExecutionOutcome,
    },
    WalReclamation {
        physical: CompletedPhysicalWalReclamationAction,
        scheduler: QueueExecutionOutcome,
    },
    TerminalFailure(PhysicalWorkTerminalFailure),
    StaleOrForeign,
}

pub struct PhysicalWorkNoEffectEvidence {
    failure: ArtifactTreeFailure,
    pub(in crate::physical_runtime) retry: super::PhysicalRetryPayload,
}

impl PhysicalWorkNoEffectEvidence {
    pub const fn failure(&self) -> ArtifactTreeFailure {
        self.failure
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkSchedulerPosture {
    NotObserved,
    Executed,
    RejectedAfterEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkPublicationResiduePosture {
    NotApplicable,
    NoneObserved,
    MayExist,
    DeletionMayHaveOccurred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkTerminalCause {
    Backend(ArtifactTreeFailure),
    IncompleteRead { expected: u64, completed: u64 },
    SchedulerRejectedAfterEffect,
}

pub struct PhysicalWorkTerminalFailure {
    identity: super::super::PhysicalWorkIdentity,
    effect_fate: PhysicalWorkEffectFate,
    target: PhysicalWorkRecoveryTarget,
    completed_bytes: u64,
    backend_operation: worth_store_physical_backend::MediaOperationIdentity,
    backend_role: MediaOperationRole,
    scheduler: PhysicalWorkSchedulerPosture,
    publication_residue: PhysicalWorkPublicationResiduePosture,
    recovery: PhysicalWorkRecoveryDisposition,
    cause: PhysicalWorkTerminalCause,
}

pub struct PhysicalWorkHealthRevocation {
    identity: super::super::PhysicalWorkIdentity,
    fate: PhysicalWorkEffectFate,
    recovery: PhysicalWorkRecoveryDisposition,
}

pub(in crate::physical_runtime) struct PhysicalWorkSettlement;

pub(in crate::physical_runtime) struct PhysicalWorkSettlementResult {
    settled: SettledPhysicalWork,
    health_revocation: Option<PhysicalWorkHealthRevocation>,
    effect_activity: super::super::submission::PhysicalEffectActivity,
    residency_writeback: Option<super::PhysicalResidencyWritebackCompletion>,
}

impl PhysicalWorkSettlement {
    pub(in crate::physical_runtime) fn settle(
        dispatch: super::PhysicalExecutorDispatch,
    ) -> PhysicalWorkSettlementResult {
        let (mut dispatched, outcome, recovery_obligation, residency_writeback) =
            dispatch.into_parts();
        let effect_activity = dispatched.take_effect_activity();
        let evidence = classification::classify(&dispatched, outcome);
        let residency_writeback = match residency_writeback {
            Some(completion)
                if completion.identity() == dispatched.intent().identity()
                    && matches!(evidence, PhysicalWorkSettlementEvidence::Write { .. }) =>
            {
                Some(completion)
            }
            _ => None,
        };
        let health_revocation =
            classification::health_revocation(&dispatched, &evidence).or_else(|| {
                recovery_obligation
                    .is_retained()
                    .then_some(PhysicalWorkHealthRevocation {
                        identity: dispatched.intent().identity(),
                        fate: evidence.fate(),
                        recovery: PhysicalWorkRecoveryDisposition::InspectionRequired,
                    })
            });
        PhysicalWorkSettlementResult {
            settled: SettledPhysicalWork::from_settlement(
                dispatched,
                evidence,
                recovery_obligation,
            ),
            health_revocation,
            effect_activity,
            residency_writeback,
        }
    }
}

impl PhysicalWorkSettlementEvidence {
    pub const fn fate(&self) -> PhysicalWorkEffectFate {
        match self {
            Self::Inspection {
                physical,
                scheduler,
                ..
            } => {
                if physical.completed_bytes() == physical.range().length() as u64
                    && physical.stable()
                    && matches!(scheduler, QueueExecutionOutcome::Executed(_))
                {
                    PhysicalWorkEffectFate::ReadCompleted
                } else {
                    PhysicalWorkEffectFate::ReadIncomplete
                }
            }
            Self::InspectionDenied(_) => PhysicalWorkEffectFate::ProvenNoEffect,
            Self::NoEffect(_) => PhysicalWorkEffectFate::ProvenNoEffect,
            Self::Metadata { .. } => PhysicalWorkEffectFate::ReadCompleted,
            Self::Read { .. } => PhysicalWorkEffectFate::ReadCompleted,
            Self::Write { scheduler, .. } => {
                if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
                    PhysicalWorkEffectFate::WriteCompleted
                } else {
                    PhysicalWorkEffectFate::WrittenButSchedulerRejected
                }
            }
            Self::Publication { scheduler, .. } => {
                if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
                    PhysicalWorkEffectFate::PublicationCompleted
                } else {
                    PhysicalWorkEffectFate::WrittenButSchedulerRejected
                }
            }
            Self::NewArtifact { scheduler, .. } | Self::PublicationEffect { scheduler, .. } => {
                if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
                    PhysicalWorkEffectFate::PublicationCompleted
                } else {
                    PhysicalWorkEffectFate::WrittenButSchedulerRejected
                }
            }
            #[cfg(feature = "recovery-runtime-owner")]
            Self::RecoveryStaging { scheduler, .. } => {
                if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
                    PhysicalWorkEffectFate::PublicationCompleted
                } else {
                    PhysicalWorkEffectFate::WrittenButSchedulerRejected
                }
            }
            Self::WalAppend { scheduler, .. } | Self::WalSegmentCreate { scheduler, .. } => {
                if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
                    PhysicalWorkEffectFate::WriteCompleted
                } else {
                    PhysicalWorkEffectFate::WrittenButSchedulerRejected
                }
            }
            Self::WalBarrier { scheduler, .. } => {
                if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
                    PhysicalWorkEffectFate::PublicationCompleted
                } else {
                    PhysicalWorkEffectFate::WrittenButSchedulerRejected
                }
            }
            Self::Checkpoint { scheduler, .. } => {
                if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
                    PhysicalWorkEffectFate::CheckpointCompleted
                } else {
                    PhysicalWorkEffectFate::WrittenButSchedulerRejected
                }
            }
            Self::WalReclamation { scheduler, .. } => {
                if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
                    PhysicalWorkEffectFate::WalReclamationCompleted
                } else {
                    PhysicalWorkEffectFate::WrittenButSchedulerRejected
                }
            }
            Self::TerminalFailure(failure) => failure.effect_fate,
            Self::StaleOrForeign => PhysicalWorkEffectFate::StaleOrForeignOutcome,
        }
    }

    pub const fn completed_payload_bytes(&self) -> u64 {
        match self {
            Self::Inspection { physical, .. } => physical.completed_bytes(),
            Self::InspectionDenied(_) => 0,
            Self::NoEffect(_) | Self::Metadata { .. } | Self::StaleOrForeign => 0,
            Self::Read { physical, .. } => physical.completed_bytes(),
            Self::Write { physical, .. } | Self::Publication { physical, .. } => {
                physical.completed_bytes()
            }
            Self::NewArtifact { physical, .. } => physical.completed_bytes(),
            #[cfg(feature = "recovery-runtime-owner")]
            Self::RecoveryStaging { physical, .. } => physical.byte_count(),
            Self::WalAppend { physical, .. } => physical.range().byte_count(),
            Self::WalSegmentCreate { physical, .. } => physical.completed_bytes(),
            Self::WalBarrier { .. } => 0,
            Self::Checkpoint { physical, .. } => physical.completed_bytes(),
            Self::WalReclamation { .. } => 0,
            Self::PublicationEffect { .. } => 0,
            Self::TerminalFailure(failure) => failure.completed_bytes,
        }
    }

    pub(in crate::physical_runtime::work) const fn recovery_disposition(
        &self,
        declared: PhysicalWorkRecoveryDisposition,
    ) -> PhysicalWorkRecoveryDisposition {
        match self {
            Self::Inspection { .. } | Self::InspectionDenied(_) => {
                PhysicalWorkRecoveryDisposition::NoEffect
            }
            Self::TerminalFailure(failure) => failure.recovery,
            Self::StaleOrForeign => PhysicalWorkRecoveryDisposition::InspectionRequired,
            Self::NoEffect(_) => declared,
            Self::Metadata { .. } | Self::Read { .. } => PhysicalWorkRecoveryDisposition::NoEffect,
            Self::Write { .. }
            | Self::Publication { .. }
            | Self::NewArtifact { .. }
            | Self::PublicationEffect { .. }
            | Self::WalAppend { .. }
            | Self::WalSegmentCreate { .. }
            | Self::WalBarrier { .. } => PhysicalWorkRecoveryDisposition::ContinueSettlement,
            #[cfg(feature = "recovery-runtime-owner")]
            Self::RecoveryStaging { .. } => PhysicalWorkRecoveryDisposition::ContinueSettlement,
            Self::Checkpoint { .. } | Self::WalReclamation { .. } => {
                PhysicalWorkRecoveryDisposition::ContinueSettlement
            }
        }
    }
}

impl PhysicalWorkTerminalFailure {
    pub const fn identity(&self) -> super::super::PhysicalWorkIdentity {
        self.identity
    }

    pub const fn effect_fate(&self) -> PhysicalWorkEffectFate {
        self.effect_fate
    }

    pub const fn target(&self) -> PhysicalWorkRecoveryTarget {
        self.target
    }

    pub const fn coordinate(&self) -> Option<worth_store_physical_format::RecordFrameCoordinate> {
        match self.target {
            PhysicalWorkRecoveryTarget::Range(coordinate) => Some(coordinate),
            _ => None,
        }
    }

    pub const fn completed_bytes(&self) -> u64 {
        self.completed_bytes
    }

    pub const fn backend_operation(&self) -> worth_store_physical_backend::MediaOperationIdentity {
        self.backend_operation
    }

    pub const fn backend_role(&self) -> MediaOperationRole {
        self.backend_role
    }

    pub const fn recovery(&self) -> PhysicalWorkRecoveryDisposition {
        self.recovery
    }

    pub const fn scheduler(&self) -> PhysicalWorkSchedulerPosture {
        self.scheduler
    }

    pub const fn publication_residue(&self) -> PhysicalWorkPublicationResiduePosture {
        self.publication_residue
    }

    pub const fn cause(&self) -> PhysicalWorkTerminalCause {
        self.cause
    }
}

impl PhysicalWorkHealthRevocation {
    pub const fn identity(&self) -> super::super::PhysicalWorkIdentity {
        self.identity
    }

    pub const fn fate(&self) -> PhysicalWorkEffectFate {
        self.fate
    }

    pub const fn recovery(&self) -> PhysicalWorkRecoveryDisposition {
        self.recovery
    }
}
