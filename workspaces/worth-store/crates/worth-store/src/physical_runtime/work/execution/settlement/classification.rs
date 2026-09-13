use worth_store_io_scheduler::QueueExecutionOutcome;
use worth_store_physical_backend::{
    ArtifactTreeFailureKind, CompletedArtifactRangeRead, CompletedArtifactRangeWrite,
    IndeterminateArtifactRangeWrite, MediaOperationRole,
};

use super::{
    PhysicalWorkEffectFate, PhysicalWorkHealthRevocation, PhysicalWorkNoEffectEvidence,
    PhysicalWorkPublicationResiduePosture, PhysicalWorkSchedulerPosture,
    PhysicalWorkSettlementEvidence, PhysicalWorkTerminalCause, PhysicalWorkTerminalFailure,
};
use crate::physical_runtime::work::{
    DispatchedPhysicalWork, PhysicalExecutorOutcome, PhysicalWorkOperationFamily,
    PhysicalWorkRecoveryDisposition, PhysicalWorkRecoveryTarget,
};

mod checkpoint;
pub(in crate::physical_runtime::work::execution::settlement) mod publication;
#[cfg(feature = "recovery-runtime-owner")]
mod recovery;
mod wal;
mod wal_reclamation;

pub(super) fn classify(
    dispatched: &DispatchedPhysicalWork,
    outcome: PhysicalExecutorOutcome,
) -> PhysicalWorkSettlementEvidence {
    match outcome {
        PhysicalExecutorOutcome::InspectionObserved {
            physical,
            bytes,
            scheduler,
        } if dispatched.matches_inspection(&physical)
            && bytes.len() == physical.range().length() as usize =>
        {
            PhysicalWorkSettlementEvidence::Inspection {
                physical,
                bytes,
                scheduler,
            }
        }
        PhysicalExecutorOutcome::InspectionDenied(failure)
            if dispatched.intent().scope().inspection_target().is_some() =>
        {
            PhysicalWorkSettlementEvidence::InspectionDenied(failure)
        }
        PhysicalExecutorOutcome::DeniedBeforeEffect { failure, retry } => {
            PhysicalWorkSettlementEvidence::NoEffect(PhysicalWorkNoEffectEvidence {
                failure,
                retry,
            })
        }
        PhysicalExecutorOutcome::MetadataCompleted {
            physical,
            scheduler,
        } if dispatched.matches_metadata(physical) => PhysicalWorkSettlementEvidence::Metadata {
            physical,
            scheduler,
        },
        PhysicalExecutorOutcome::ReadCompleted {
            physical,
            bytes,
            scheduler,
        } if dispatched.matches_read(physical) => {
            classify_completed_read(dispatched, physical, bytes, scheduler)
        }
        PhysicalExecutorOutcome::WriteCompleted {
            physical,
            scheduler,
        } if dispatched.matches_write(&physical) => {
            classify_completed_write(dispatched, physical, scheduler, false)
        }
        PhysicalExecutorOutcome::ResidencyWritebackCompleted {
            physical,
            scheduler,
        } if dispatched.matches_write(&physical) => PhysicalWorkSettlementEvidence::Write {
            physical,
            scheduler,
        },
        PhysicalExecutorOutcome::PublicationCompleted {
            physical,
            scheduler,
        } if dispatched.matches_write(&physical) => {
            classify_completed_write(dispatched, physical, scheduler, true)
        }
        PhysicalExecutorOutcome::NewArtifactCompleted {
            physical,
            coordinate,
            scheduler,
        } if dispatched.matches_new_artifact(&physical, coordinate) => {
            publication::classify_completed_new_artifact(
                dispatched, physical, coordinate, scheduler,
            )
        }
        #[cfg(feature = "recovery-runtime-owner")]
        PhysicalExecutorOutcome::RecoveryStagingCompleted {
            physical,
            scheduler,
        } if dispatched.matches_recovery_staging(&physical) => {
            PhysicalWorkSettlementEvidence::RecoveryStaging {
                physical,
                scheduler,
            }
        }
        PhysicalExecutorOutcome::PublicationEffectCompleted {
            physical,
            scheduler,
        } if dispatched.matches_publication_effect(&physical) => {
            publication::classify_completed_publication_effect(dispatched, physical, scheduler)
        }
        PhysicalExecutorOutcome::WalAppendCompleted {
            physical,
            scheduler,
        } if dispatched.matches_wal_append(&physical) => {
            wal::classify_completed_wal_append(dispatched, physical, scheduler)
        }
        PhysicalExecutorOutcome::WalSegmentCreateCompleted {
            physical,
            scheduler,
        } if dispatched.matches_wal_segment_create(&physical) => {
            wal::classify_completed_wal_segment_create(dispatched, physical, scheduler)
        }
        PhysicalExecutorOutcome::WalBarrierCompleted {
            physical,
            scheduler,
        } if dispatched.matches_wal_barrier(&physical) => {
            wal::classify_completed_wal_barrier(dispatched, physical, scheduler)
        }
        PhysicalExecutorOutcome::CheckpointCompleted {
            physical,
            scheduler,
        } if checkpoint::matches_completed(dispatched, &physical) => {
            checkpoint::classify_completed(dispatched, physical, scheduler)
        }
        PhysicalExecutorOutcome::WalReclamationCompleted {
            physical,
            scheduler,
        } if wal_reclamation::matches_completed(dispatched, &physical) => {
            wal_reclamation::classify_completed(dispatched, physical, scheduler)
        }
        PhysicalExecutorOutcome::Indeterminate(physical)
            if dispatched.matches_indeterminate(physical) =>
        {
            indeterminate_terminal(dispatched, physical)
        }
        PhysicalExecutorOutcome::NewArtifactIndeterminate {
            physical,
            coordinate,
        } if dispatched.matches_new_artifact_indeterminate(&physical, coordinate) => {
            publication::indeterminate_new_artifact(dispatched, physical, coordinate)
        }
        #[cfg(feature = "recovery-runtime-owner")]
        PhysicalExecutorOutcome::RecoveryStagingIndeterminate(physical)
            if dispatched.matches_recovery_staging_indeterminate(&physical) =>
        {
            let coordinate = dispatched
                .coordinate()
                .expect("recovery staging work has one exact coordinate");
            recovery::indeterminate_recovery_staging(dispatched, physical, coordinate)
        }
        PhysicalExecutorOutcome::PublicationEffectIndeterminate(physical)
            if dispatched.matches_publication_effect_indeterminate(&physical) =>
        {
            publication::indeterminate_publication_effect(dispatched, physical)
        }
        PhysicalExecutorOutcome::WalAppendIndeterminate(physical)
            if dispatched.matches_wal_append_indeterminate(&physical) =>
        {
            wal::indeterminate_wal_append(dispatched, physical)
        }
        PhysicalExecutorOutcome::WalSegmentCreateIndeterminate(physical)
            if dispatched.matches_wal_segment_create_indeterminate(&physical) =>
        {
            wal::indeterminate_wal_segment_create(dispatched, physical)
        }
        PhysicalExecutorOutcome::WalBarrierIndeterminate(physical)
            if dispatched.matches_wal_barrier_indeterminate(&physical) =>
        {
            wal::indeterminate_wal_barrier(dispatched, physical)
        }
        PhysicalExecutorOutcome::CheckpointIndeterminate(physical)
            if checkpoint::matches_indeterminate(dispatched, &physical) =>
        {
            checkpoint::classify_indeterminate(dispatched, physical)
        }
        PhysicalExecutorOutcome::WalReclamationIndeterminate(physical)
            if wal_reclamation::matches_indeterminate(dispatched, &physical) =>
        {
            wal_reclamation::classify_indeterminate(dispatched, physical)
        }
        _ => PhysicalWorkSettlementEvidence::StaleOrForeign,
    }
}

fn classify_completed_read(
    dispatched: &DispatchedPhysicalWork,
    physical: CompletedArtifactRangeRead,
    bytes: Box<[u8]>,
    scheduler: QueueExecutionOutcome,
) -> PhysicalWorkSettlementEvidence {
    let expected = u64::from(physical.coordinate().length());
    if physical.completed_bytes() == expected && bytes.len() as u64 == expected {
        return PhysicalWorkSettlementEvidence::Read {
            physical,
            bytes,
            scheduler,
        };
    }
    PhysicalWorkSettlementEvidence::TerminalFailure(PhysicalWorkTerminalFailure {
        identity: dispatched.intent().identity(),
        effect_fate: PhysicalWorkEffectFate::ReadIncomplete,
        target: PhysicalWorkRecoveryTarget::Range(physical.coordinate()),
        completed_bytes: physical.completed_bytes(),
        backend_operation: physical.operation(),
        backend_role: MediaOperationRole::PositionedRead,
        scheduler: scheduler_posture(scheduler),
        publication_residue: PhysicalWorkPublicationResiduePosture::NotApplicable,
        recovery: PhysicalWorkRecoveryDisposition::InspectionRequired,
        cause: PhysicalWorkTerminalCause::IncompleteRead {
            expected,
            completed: physical.completed_bytes(),
        },
    })
}

fn indeterminate_terminal(
    dispatched: &DispatchedPhysicalWork,
    physical: IndeterminateArtifactRangeWrite,
) -> PhysicalWorkSettlementEvidence {
    PhysicalWorkSettlementEvidence::TerminalFailure(PhysicalWorkTerminalFailure {
        identity: dispatched.intent().identity(),
        effect_fate: PhysicalWorkEffectFate::Indeterminate,
        target: PhysicalWorkRecoveryTarget::Range(physical.coordinate()),
        completed_bytes: physical.completed_bytes(),
        backend_operation: physical.operation(),
        backend_role: MediaOperationRole::PositionedWrite,
        scheduler: PhysicalWorkSchedulerPosture::NotObserved,
        publication_residue: indeterminate_publication_residue(dispatched),
        recovery: PhysicalWorkRecoveryDisposition::InspectionRequired,
        cause: PhysicalWorkTerminalCause::Backend(physical.failure()),
    })
}

pub(super) fn health_revocation(
    dispatched: &DispatchedPhysicalWork,
    evidence: &PhysicalWorkSettlementEvidence,
) -> Option<PhysicalWorkHealthRevocation> {
    let fate = evidence.fate();
    requires_health_revocation(evidence, fate).then_some(PhysicalWorkHealthRevocation {
        identity: dispatched.intent().identity(),
        fate,
        recovery: PhysicalWorkRecoveryDisposition::InspectionRequired,
    })
}

fn requires_health_revocation(
    evidence: &PhysicalWorkSettlementEvidence,
    fate: PhysicalWorkEffectFate,
) -> bool {
    matches!(
        fate,
        PhysicalWorkEffectFate::Indeterminate
            | PhysicalWorkEffectFate::WrittenButSchedulerRejected
            | PhysicalWorkEffectFate::StaleOrForeignOutcome
    ) || matches!(evidence, PhysicalWorkSettlementEvidence::TerminalFailure(_))
        || matches!(
            evidence,
            PhysicalWorkSettlementEvidence::NoEffect(evidence)
                if evidence.failure().kind() == ArtifactTreeFailureKind::Damaged
        )
}

fn classify_completed_write(
    dispatched: &DispatchedPhysicalWork,
    physical: CompletedArtifactRangeWrite,
    scheduler: QueueExecutionOutcome,
    publication: bool,
) -> PhysicalWorkSettlementEvidence {
    if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
        if publication {
            PhysicalWorkSettlementEvidence::Publication {
                physical,
                scheduler,
            }
        } else {
            PhysicalWorkSettlementEvidence::Write {
                physical,
                scheduler,
            }
        }
    } else {
        terminal_scheduler_failure(dispatched, physical, publication)
    }
}

fn terminal_scheduler_failure(
    dispatched: &DispatchedPhysicalWork,
    physical: CompletedArtifactRangeWrite,
    publication: bool,
) -> PhysicalWorkSettlementEvidence {
    PhysicalWorkSettlementEvidence::TerminalFailure(PhysicalWorkTerminalFailure {
        identity: dispatched.intent().identity(),
        effect_fate: PhysicalWorkEffectFate::WrittenButSchedulerRejected,
        target: PhysicalWorkRecoveryTarget::Range(physical.coordinate()),
        completed_bytes: physical.completed_bytes(),
        backend_operation: physical.operation(),
        backend_role: MediaOperationRole::PositionedWrite,
        scheduler: PhysicalWorkSchedulerPosture::RejectedAfterEffect,
        publication_residue: if publication {
            PhysicalWorkPublicationResiduePosture::MayExist
        } else {
            PhysicalWorkPublicationResiduePosture::NotApplicable
        },
        recovery: PhysicalWorkRecoveryDisposition::InspectionRequired,
        cause: PhysicalWorkTerminalCause::SchedulerRejectedAfterEffect,
    })
}

fn scheduler_posture(scheduler: QueueExecutionOutcome) -> PhysicalWorkSchedulerPosture {
    if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
        PhysicalWorkSchedulerPosture::Executed
    } else {
        PhysicalWorkSchedulerPosture::RejectedAfterEffect
    }
}

fn indeterminate_publication_residue(
    dispatched: &DispatchedPhysicalWork,
) -> PhysicalWorkPublicationResiduePosture {
    if dispatched.intent().operation() == PhysicalWorkOperationFamily::ArtifactPublication {
        PhysicalWorkPublicationResiduePosture::MayExist
    } else {
        PhysicalWorkPublicationResiduePosture::NotApplicable
    }
}
