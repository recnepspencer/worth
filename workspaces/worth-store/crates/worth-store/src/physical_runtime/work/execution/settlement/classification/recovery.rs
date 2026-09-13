use worth_store_physical_backend::{
    IndeterminateRecoveryStagingWrite, RecoveryStagingIndeterminatePhysical,
};
use worth_store_physical_format::RecordFrameCoordinate;

use super::super::{
    PhysicalWorkEffectFate, PhysicalWorkPublicationResiduePosture, PhysicalWorkSchedulerPosture,
    PhysicalWorkSettlementEvidence, PhysicalWorkTerminalCause, PhysicalWorkTerminalFailure,
};
use crate::physical_runtime::work::{
    DispatchedPhysicalWork, PhysicalWorkRecoveryDisposition, PhysicalWorkRecoveryTarget,
};

pub(super) fn indeterminate_recovery_staging(
    dispatched: &DispatchedPhysicalWork,
    physical: IndeterminateRecoveryStagingWrite,
    coordinate: RecordFrameCoordinate,
) -> PhysicalWorkSettlementEvidence {
    match physical.into_physical() {
        RecoveryStagingIndeterminatePhysical::NewArtifact(new_artifact) => {
            super::publication::indeterminate_new_artifact(dispatched, new_artifact, coordinate)
        }
        RecoveryStagingIndeterminatePhysical::Append {
            prefix_verified,
            append,
        } => {
            let completed_bytes = prefix_verified
                .map_or(0, |prefix| prefix.completed_bytes())
                .saturating_add(append.completed_bytes());
            PhysicalWorkSettlementEvidence::TerminalFailure(PhysicalWorkTerminalFailure {
                identity: dispatched.intent().identity(),
                effect_fate: PhysicalWorkEffectFate::Indeterminate,
                target: PhysicalWorkRecoveryTarget::Range(coordinate),
                completed_bytes,
                backend_operation: append.operation(),
                backend_role: worth_store_physical_backend::MediaOperationRole::Append,
                scheduler: PhysicalWorkSchedulerPosture::NotObserved,
                publication_residue: PhysicalWorkPublicationResiduePosture::MayExist,
                recovery: PhysicalWorkRecoveryDisposition::InspectionRequired,
                cause: PhysicalWorkTerminalCause::Backend(append.failure()),
            })
        }
    }
}
