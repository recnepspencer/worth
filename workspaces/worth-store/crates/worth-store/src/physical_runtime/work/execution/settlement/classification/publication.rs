use worth_store_io_scheduler::QueueExecutionOutcome;
use worth_store_physical_backend::{
    CompletedArtifactNewWrite, IndeterminateArtifactNewWrite, MediaOperationRole,
};
use worth_store_physical_format::RecordFrameCoordinate;

use super::super::{
    PhysicalWorkEffectFate, PhysicalWorkPublicationResiduePosture, PhysicalWorkSchedulerPosture,
    PhysicalWorkSettlementEvidence, PhysicalWorkTerminalCause, PhysicalWorkTerminalFailure,
};
use crate::physical_runtime::work::{
    DispatchedPhysicalWork, PhysicalWorkRecoveryDisposition, PhysicalWorkRecoveryTarget,
};

pub(super) fn classify_completed_new_artifact(
    dispatched: &DispatchedPhysicalWork,
    physical: CompletedArtifactNewWrite,
    coordinate: RecordFrameCoordinate,
    scheduler: QueueExecutionOutcome,
) -> PhysicalWorkSettlementEvidence {
    if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
        PhysicalWorkSettlementEvidence::NewArtifact {
            physical,
            coordinate,
            scheduler,
        }
    } else {
        PhysicalWorkSettlementEvidence::TerminalFailure(PhysicalWorkTerminalFailure {
            identity: dispatched.intent().identity(),
            effect_fate: PhysicalWorkEffectFate::WrittenButSchedulerRejected,
            target: PhysicalWorkRecoveryTarget::Range(coordinate),
            completed_bytes: physical.completed_bytes(),
            backend_operation: physical.write_operation(),
            backend_role: MediaOperationRole::PositionedWrite,
            scheduler: PhysicalWorkSchedulerPosture::RejectedAfterEffect,
            publication_residue: PhysicalWorkPublicationResiduePosture::MayExist,
            recovery: PhysicalWorkRecoveryDisposition::InspectionRequired,
            cause: PhysicalWorkTerminalCause::SchedulerRejectedAfterEffect,
        })
    }
}

pub(super) fn classify_completed_publication_effect(
    dispatched: &DispatchedPhysicalWork,
    physical: crate::physical_runtime::work::CompletedPhysicalPublicationEffect,
    scheduler: QueueExecutionOutcome,
) -> PhysicalWorkSettlementEvidence {
    if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
        PhysicalWorkSettlementEvidence::PublicationEffect {
            physical,
            scheduler,
        }
    } else {
        PhysicalWorkSettlementEvidence::TerminalFailure(PhysicalWorkTerminalFailure {
            identity: dispatched.intent().identity(),
            effect_fate: PhysicalWorkEffectFate::WrittenButSchedulerRejected,
            target: physical.recovery_target(),
            completed_bytes: 0,
            backend_operation: physical.physical().operation(),
            backend_role: effect_role(physical.effect()),
            scheduler: PhysicalWorkSchedulerPosture::RejectedAfterEffect,
            publication_residue: PhysicalWorkPublicationResiduePosture::MayExist,
            recovery: PhysicalWorkRecoveryDisposition::InspectionRequired,
            cause: PhysicalWorkTerminalCause::SchedulerRejectedAfterEffect,
        })
    }
}

pub(super) fn indeterminate_new_artifact(
    dispatched: &DispatchedPhysicalWork,
    physical: IndeterminateArtifactNewWrite,
    coordinate: RecordFrameCoordinate,
) -> PhysicalWorkSettlementEvidence {
    let backend_role = if physical.write_operation().is_some() {
        MediaOperationRole::PositionedWrite
    } else {
        MediaOperationRole::CreateNew
    };
    PhysicalWorkSettlementEvidence::TerminalFailure(PhysicalWorkTerminalFailure {
        identity: dispatched.intent().identity(),
        effect_fate: PhysicalWorkEffectFate::Indeterminate,
        target: PhysicalWorkRecoveryTarget::Range(coordinate),
        completed_bytes: physical.completed_bytes(),
        backend_operation: physical
            .write_operation()
            .unwrap_or_else(|| physical.create_operation()),
        backend_role,
        scheduler: PhysicalWorkSchedulerPosture::NotObserved,
        publication_residue: PhysicalWorkPublicationResiduePosture::MayExist,
        recovery: PhysicalWorkRecoveryDisposition::InspectionRequired,
        cause: PhysicalWorkTerminalCause::Backend(physical.failure()),
    })
}

pub(super) fn indeterminate_publication_effect(
    dispatched: &DispatchedPhysicalWork,
    physical: crate::physical_runtime::work::IndeterminatePhysicalPublicationEffect,
) -> PhysicalWorkSettlementEvidence {
    PhysicalWorkSettlementEvidence::TerminalFailure(PhysicalWorkTerminalFailure {
        identity: dispatched.intent().identity(),
        effect_fate: PhysicalWorkEffectFate::Indeterminate,
        target: physical.recovery_target(),
        completed_bytes: 0,
        backend_operation: physical.physical().operation(),
        backend_role: effect_role(physical.effect()),
        scheduler: PhysicalWorkSchedulerPosture::NotObserved,
        publication_residue: PhysicalWorkPublicationResiduePosture::MayExist,
        recovery: PhysicalWorkRecoveryDisposition::InspectionRequired,
        cause: PhysicalWorkTerminalCause::Backend(physical.physical().failure()),
    })
}

pub(in crate::physical_runtime::work::execution::settlement) const fn effect_role(
    effect: crate::physical_runtime::work::PhysicalPublicationEffect,
) -> MediaOperationRole {
    match effect {
        crate::physical_runtime::work::PhysicalPublicationEffect::SynchronizeArtifact => {
            MediaOperationRole::SynchronizeFileState
        }
        crate::physical_runtime::work::PhysicalPublicationEffect::SynchronizeArtifactParent
        | crate::physical_runtime::work::PhysicalPublicationEffect::SynchronizeRecordFamily => {
            MediaOperationRole::SynchronizeDirectoryPublication
        }
        crate::physical_runtime::work::PhysicalPublicationEffect::ReplaceCatalog => {
            MediaOperationRole::AtomicReplace
        }
        crate::physical_runtime::work::PhysicalPublicationEffect::RemoveArtifact => {
            MediaOperationRole::Delete
        }
    }
}
