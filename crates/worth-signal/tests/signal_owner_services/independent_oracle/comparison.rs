use worth_foundational::FoundationalBranchTarget;
use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, SignalBranchAdvanceDenial,
    SignalBranchBasisObservationDenial, SignalBranchBasisReadmissionDenial,
    SignalBranchRetentionAcquisitionDenial, SignalBranchRetentionReleaseDenial,
};

use super::state::{ModelObservation, ModelSnapshot};
use super::transition::ModelDenial;

pub(crate) fn neutral_basis(basis: &AdmittedSignalBranchBasis) -> ModelObservation {
    let target = match basis.observation().target() {
        FoundationalBranchTarget::Basis(target) => target,
        FoundationalBranchTarget::Empty => panic!("Signal basis observation cannot be empty"),
    };
    ModelObservation {
        branch: basis.branch_id().0,
        graph_instance: target.graph_instance_id().to_owned(),
        definition_basis: target.definition_basis(),
        snapshot: target.snapshot_id(),
        restore_snapshot: target.restore_snapshot_id(),
        generation: basis.observation().generation().get(),
    }
}

pub(crate) fn neutral_snapshot(snapshot: &AdmittedSignalBranchSnapshot) -> ModelSnapshot {
    ModelSnapshot {
        branch: snapshot.snapshot().meta.branch_id.0,
        snapshot: snapshot.snapshot().meta.snapshot_id.0,
    }
}

pub(crate) fn basis_observation_denial(denial: &SignalBranchBasisObservationDenial) -> ModelDenial {
    match denial {
        SignalBranchBasisObservationDenial::OwnerUnavailable(_) => ModelDenial::OwnerUnavailable,
        SignalBranchBasisObservationDenial::ManagedReferenceDenied { denial } => match denial {
            worth_signal::facade::branch::ManagedSignalBranchReferenceAdmissionDenial::BranchLifecycleEnded => {
                ModelDenial::RetiredBranch
            }
            worth_signal::facade::branch::ManagedSignalBranchReferenceAdmissionDenial::OwnerUnavailable(_) => {
                ModelDenial::OwnerUnavailable
            }
            other => panic!("unexpected managed-reference denial in oracle trace: {other:?}"),
        },
        SignalBranchBasisObservationDenial::UnknownBranch { .. } => ModelDenial::UnknownBranch,
        SignalBranchBasisObservationDenial::RetiredBranch { .. } => ModelDenial::RetiredBranch,
        SignalBranchBasisObservationDenial::RetentionUnavailable { denial } => {
            retention_denial(denial)
        }
        other => panic!("unexpected observation denial in oracle trace: {other:?}"),
    }
}

pub(crate) fn readmission_denial(denial: &SignalBranchBasisReadmissionDenial) -> ModelDenial {
    match denial {
        SignalBranchBasisReadmissionDenial::OwnerUnavailable(_) => ModelDenial::OwnerUnavailable,
        SignalBranchBasisReadmissionDenial::UnknownBranch { .. } => ModelDenial::UnknownBranch,
        SignalBranchBasisReadmissionDenial::RetiredBranch { .. } => ModelDenial::RetiredBranch,
        SignalBranchBasisReadmissionDenial::ReferenceMismatch { .. } => ModelDenial::StaleBasis,
        SignalBranchBasisReadmissionDenial::RetentionIdentityExhausted
        | SignalBranchBasisReadmissionDenial::UnavailableRetention { .. } => {
            panic!("retention capacity is not part of the focused oracle trace")
        }
        other => panic!("unexpected readmission denial in oracle trace: {other:?}"),
    }
}

pub(crate) fn retention_denial(denial: &SignalBranchRetentionAcquisitionDenial) -> ModelDenial {
    match denial {
        SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(_) => {
            ModelDenial::OwnerUnavailable
        }
        SignalBranchRetentionAcquisitionDenial::UnknownBranch { .. } => ModelDenial::UnknownBranch,
        SignalBranchRetentionAcquisitionDenial::RetiredBranch { .. } => ModelDenial::RetiredBranch,
        other => panic!("unexpected retention denial in oracle trace: {other:?}"),
    }
}

pub(crate) fn release_denial(denial: &SignalBranchRetentionReleaseDenial) -> ModelDenial {
    match denial {
        SignalBranchRetentionReleaseDenial::OwnerUnavailable(_) => ModelDenial::OwnerUnavailable,
        SignalBranchRetentionReleaseDenial::ForeignRuntime => ModelDenial::ForeignSnapshot,
        other => panic!("unexpected release denial in oracle trace: {other:?}"),
    }
}

pub(crate) fn advance_denial(denial: &SignalBranchAdvanceDenial) -> ModelDenial {
    match denial {
        SignalBranchAdvanceDenial::OwnerUnavailable(_) => ModelDenial::OwnerUnavailable,
        SignalBranchAdvanceDenial::CancelledNoMovement => ModelDenial::Cancelled,
        SignalBranchAdvanceDenial::UnknownBranch { .. } => ModelDenial::UnknownBranch,
        SignalBranchAdvanceDenial::RetiredBranch { .. } => ModelDenial::RetiredBranch,
        SignalBranchAdvanceDenial::BasisMismatch { .. }
        | SignalBranchAdvanceDenial::MutationDeniedNoMovement {
            denial:
                worth_signal::facade::branch::SignalBranchAdvanceEngineDenial::CanonicalBasisMismatch,
        } => ModelDenial::StaleBasis,
        other => panic!("unexpected advance denial in oracle trace: {other:?}"),
    }
}

pub(crate) fn fork_denial(
    denial: &worth_signal::facade::branch::SignalBranchForkOperationDenial,
) -> ModelDenial {
    use worth_signal::facade::branch::SignalBranchForkOperationDenial;

    match denial {
        SignalBranchForkOperationDenial::OwnerUnavailable(_) => ModelDenial::OwnerUnavailable,
        SignalBranchForkOperationDenial::CancelledNoMovement => ModelDenial::Cancelled,
        SignalBranchForkOperationDenial::UnknownBranch { .. } => ModelDenial::UnknownBranch,
        SignalBranchForkOperationDenial::RetiredBranch { .. } => ModelDenial::RetiredBranch,
        SignalBranchForkOperationDenial::BasisMismatch { .. } => ModelDenial::StaleBasis,
        other => panic!("unexpected fork denial in adjacency oracle: {other:?}"),
    }
}

pub(crate) fn capture_denial(
    denial: &worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial,
) -> ModelDenial {
    use worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial;

    match denial {
        SignalBranchSnapshotCaptureDenial::OwnerUnavailable(_) => ModelDenial::OwnerUnavailable,
        SignalBranchSnapshotCaptureDenial::CancelledNoMovement => ModelDenial::Cancelled,
        SignalBranchSnapshotCaptureDenial::UnknownBranch { .. } => ModelDenial::UnknownBranch,
        SignalBranchSnapshotCaptureDenial::RetiredBranch { .. } => ModelDenial::RetiredBranch,
        SignalBranchSnapshotCaptureDenial::BasisMismatch { .. } => ModelDenial::StaleBasis,
        other => panic!("unexpected capture denial in adjacency oracle: {other:?}"),
    }
}

pub(crate) fn restore_denial(
    denial: &worth_signal::facade::branch::SignalBranchRestoreDenial,
) -> ModelDenial {
    use worth_signal::facade::branch::SignalBranchRestoreDenial;

    match denial {
        SignalBranchRestoreDenial::OwnerUnavailable(_) => ModelDenial::OwnerUnavailable,
        SignalBranchRestoreDenial::CancelledNoMovement => ModelDenial::Cancelled,
        SignalBranchRestoreDenial::UnknownBranch { .. } => ModelDenial::UnknownBranch,
        SignalBranchRestoreDenial::RetiredBranch { .. } => ModelDenial::RetiredBranch,
        SignalBranchRestoreDenial::BasisMismatch { .. } => ModelDenial::StaleBasis,
        SignalBranchRestoreDenial::CrossBranchSnapshot { .. }
        | SignalBranchRestoreDenial::UnavailableSnapshot { .. }
        | SignalBranchRestoreDenial::ForeignSnapshotOwner { .. } => ModelDenial::ForeignSnapshot,
        other => panic!("unexpected restore denial in adjacency oracle: {other:?}"),
    }
}
