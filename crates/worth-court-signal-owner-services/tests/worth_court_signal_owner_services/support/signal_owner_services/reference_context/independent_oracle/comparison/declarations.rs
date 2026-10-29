pub(crate) use {worth_foundational::FoundationalBranchTarget};
pub(crate) use {worth_signal::facade::branch::AdmittedSignalBranchBasis, worth_signal::facade::branch::AdmittedSignalBranchSnapshot, worth_signal::facade::branch::SignalBranchAdvanceDenial, worth_signal::facade::branch::SignalBranchBasisObservationDenial, worth_signal::facade::branch::SignalBranchBasisReadmissionDenial, worth_signal::facade::branch::SignalBranchRetentionAcquisitionDenial, worth_signal::facade::branch::SignalBranchRetentionReleaseDenial};

pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::state::ModelObservation, crate::signal_owner_services::reference_context::independent_oracle::state::ModelSnapshot};
pub(crate) use {crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial};

pub(crate) fn neutral_basis(basis: &AdmittedSignalBranchBasis) -> ModelObservation {
    let target = match basis.observation().target() {
        worth_foundational::FoundationalBranchTarget::Basis(target) => target,
        worth_foundational::FoundationalBranchTarget::Empty => panic!("Signal basis observation cannot be empty"),
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
        worth_signal::facade::branch::SignalBranchBasisObservationDenial::OwnerUnavailable(_) => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable,
        worth_signal::facade::branch::SignalBranchBasisObservationDenial::ManagedReferenceDenied { denial } => match denial {
            worth_signal::facade::branch::ManagedSignalBranchReferenceAdmissionDenial::BranchLifecycleEnded => {
                crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch
            }
            worth_signal::facade::branch::ManagedSignalBranchReferenceAdmissionDenial::OwnerUnavailable(_) => {
                crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable
            }
            other => panic!("unexpected managed-reference denial in oracle trace: {other:?}"),
        },
        worth_signal::facade::branch::SignalBranchBasisObservationDenial::UnknownBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch,
        worth_signal::facade::branch::SignalBranchBasisObservationDenial::RetiredBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch,
        worth_signal::facade::branch::SignalBranchBasisObservationDenial::RetentionUnavailable { denial } => {
            retention_denial(denial)
        }
        other => panic!("unexpected observation denial in oracle trace: {other:?}"),
    }
}

pub(crate) fn readmission_denial(denial: &SignalBranchBasisReadmissionDenial) -> ModelDenial {
    match denial {
        worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::OwnerUnavailable(_) => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable,
        worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::UnknownBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch,
        worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::RetiredBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch,
        worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::ReferenceMismatch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::StaleBasis,
        worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::RetentionIdentityExhausted
        | worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::UnavailableRetention { .. } => {
            panic!("retention capacity is not part of the focused oracle trace")
        }
        other => panic!("unexpected readmission denial in oracle trace: {other:?}"),
    }
}

pub(crate) fn retention_denial(denial: &SignalBranchRetentionAcquisitionDenial) -> ModelDenial {
    match denial {
        worth_signal::facade::branch::SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(_) => {
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable
        }
        worth_signal::facade::branch::SignalBranchRetentionAcquisitionDenial::UnknownBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch,
        worth_signal::facade::branch::SignalBranchRetentionAcquisitionDenial::RetiredBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch,
        other => panic!("unexpected retention denial in oracle trace: {other:?}"),
    }
}

pub(crate) fn release_denial(denial: &SignalBranchRetentionReleaseDenial) -> ModelDenial {
    match denial {
        worth_signal::facade::branch::SignalBranchRetentionReleaseDenial::OwnerUnavailable(_) => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable,
        worth_signal::facade::branch::SignalBranchRetentionReleaseDenial::ForeignRuntime => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::ForeignSnapshot,
        other => panic!("unexpected release denial in oracle trace: {other:?}"),
    }
}

pub(crate) fn advance_denial(denial: &SignalBranchAdvanceDenial) -> ModelDenial {
    match denial {
        worth_signal::facade::branch::SignalBranchAdvanceDenial::OwnerUnavailable(_) => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable,
        worth_signal::facade::branch::SignalBranchAdvanceDenial::CancelledNoMovement => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::Cancelled,
        worth_signal::facade::branch::SignalBranchAdvanceDenial::UnknownBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch,
        worth_signal::facade::branch::SignalBranchAdvanceDenial::RetiredBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch,
        worth_signal::facade::branch::SignalBranchAdvanceDenial::BasisMismatch { .. }
        | worth_signal::facade::branch::SignalBranchAdvanceDenial::MutationDeniedNoMovement {
            denial:
                worth_signal::facade::branch::SignalBranchAdvanceEngineDenial::CanonicalBasisMismatch,
        } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::StaleBasis,
        other => panic!("unexpected advance denial in oracle trace: {other:?}"),
    }
}

pub(crate) fn fork_denial(
    denial: &worth_signal::facade::branch::SignalBranchForkOperationDenial,
) -> ModelDenial {
    use {worth_signal::facade::branch::SignalBranchForkOperationDenial};

    match denial {
        SignalBranchForkOperationDenial::OwnerUnavailable(_) => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable,
        SignalBranchForkOperationDenial::CancelledNoMovement => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::Cancelled,
        SignalBranchForkOperationDenial::UnknownBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch,
        SignalBranchForkOperationDenial::RetiredBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch,
        SignalBranchForkOperationDenial::BasisMismatch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::StaleBasis,
        other => panic!("unexpected fork denial in adjacency oracle: {other:?}"),
    }
}

pub(crate) fn capture_denial(
    denial: &worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial,
) -> ModelDenial {
    use {worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial};

    match denial {
        SignalBranchSnapshotCaptureDenial::OwnerUnavailable(_) => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable,
        SignalBranchSnapshotCaptureDenial::CancelledNoMovement => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::Cancelled,
        SignalBranchSnapshotCaptureDenial::UnknownBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch,
        SignalBranchSnapshotCaptureDenial::RetiredBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch,
        SignalBranchSnapshotCaptureDenial::BasisMismatch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::StaleBasis,
        other => panic!("unexpected capture denial in adjacency oracle: {other:?}"),
    }
}

pub(crate) fn restore_denial(
    denial: &worth_signal::facade::branch::SignalBranchRestoreDenial,
) -> ModelDenial {
    use {worth_signal::facade::branch::SignalBranchRestoreDenial};

    match denial {
        SignalBranchRestoreDenial::OwnerUnavailable(_) => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable,
        SignalBranchRestoreDenial::CancelledNoMovement => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::Cancelled,
        SignalBranchRestoreDenial::UnknownBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch,
        SignalBranchRestoreDenial::RetiredBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch,
        SignalBranchRestoreDenial::BasisMismatch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::StaleBasis,
        SignalBranchRestoreDenial::CrossBranchSnapshot { .. }
        | SignalBranchRestoreDenial::UnavailableSnapshot { .. }
        | SignalBranchRestoreDenial::ForeignSnapshotOwner { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::ForeignSnapshot,
        other => panic!("unexpected restore denial in adjacency oracle: {other:?}"),
    }
}