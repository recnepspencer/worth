pub(crate) use {
    worth_signal::facade::branch::AdmittedSignalBranchBasis,
    worth_signal::facade::branch::AdmittedSignalBranchSnapshot,
    worth_signal::facade::branch::SignalBranchRetentionLease,
    worth_signal::facade::branch::SignalBranchRetirementDenial,
};

pub(crate) use {
    crate::signal_owner_services::reference_context::independent_oracle::comparison::neutral_basis,
    crate::signal_owner_services::reference_context::independent_oracle::comparison::neutral_snapshot,
};
pub(crate) use {
    crate::signal_owner_services::reference_context::independent_oracle::state::ModelObservation,
    crate::signal_owner_services::reference_context::independent_oracle::state::ModelSnapshot,
};
pub(crate) use {
    crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial,
    crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult,
    crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess,
};

/// Normalized result from a real public owner-port call.
///
/// The wrapper deliberately keeps only neutral observations and the owner-issued
/// values needed to continue the second half of an adjacent pair. It never
/// calls production semantic helpers to decide what the result means.
#[derive(Debug)]
pub(crate) enum RealResult {
    Success(RealSuccess),
    Denied(ModelDenial),
}

#[derive(Debug)]
pub(crate) enum RealSuccess {
    Fork {
        observation: ModelObservation,
        child_id: u64,
        basis: AdmittedSignalBranchBasis,
    },
    Advance {
        observation: ModelObservation,
        basis: AdmittedSignalBranchBasis,
    },
    Capture {
        observation: ModelObservation,
        snapshot: ModelSnapshot,
        admitted_snapshot: Box<AdmittedSignalBranchSnapshot>,
        basis: AdmittedSignalBranchBasis,
    },
    Restore {
        observation: ModelObservation,
        basis: AdmittedSignalBranchBasis,
    },
    Lease(SignalBranchRetentionLease),
    Release {
        model_key: u64,
    },
    Retirement,
    Closed,
}

pub(crate) fn assert_equivalent(expected: &ModelResult, actual: &RealResult, context: &str) {
    match (expected, actual) {
        (
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Fork(expected)),
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Fork {
                observation,
                child_id,
                basis,
            }),
        ) => {
            assert_eq!(expected, observation, "{context}: fork observation diverged");
            assert_eq!(
                *child_id, observation.branch,
                "{context}: real fork identity was not carried into the oracle"
            );
            assert_eq!(
                observation,
                &neutral_basis(basis),
                "{context}: real fork basis diverged"
            );
        }
        (
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Advance(expected)),
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Advance { observation, basis }),
        )
        | (
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Restore(expected)),
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Restore { observation, basis }),
        ) => {
            assert_eq!(expected, observation, "{context}: movement observation diverged");
            assert_eq!(
                observation,
                &neutral_basis(basis),
                "{context}: movement basis diverged"
            );
        }
        (
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Capture {
                observation: expected_observation,
                snapshot: expected_snapshot,
            }),
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Capture {
                observation,
                snapshot,
                admitted_snapshot,
                basis,
            }),
        ) => {
            assert_eq!(
                expected_observation, observation,
                "{context}: capture observation diverged"
            );
            assert_eq!(
                expected_snapshot, snapshot,
                "{context}: capture snapshot identity diverged"
            );
            assert_eq!(
                observation,
                &neutral_basis(basis),
                "{context}: capture basis diverged"
            );
            assert_eq!(
                snapshot,
                &neutral_snapshot(admitted_snapshot),
                "{context}: admitted snapshot diverged"
            );
        }
        (
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Lease),
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Lease(_)),
        )
        | (
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Release),
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Release { .. }),
        )
        | (
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Retirement),
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Retirement),
        )
        | (
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Closed),
            crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Success(crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealSuccess::Closed),
        ) => {}
        (crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(expected), crate::signal_owner_services::reference_context::independent_oracle::model_sequences::adjacency_roster::outcome::RealResult::Denied(actual)) => {
            assert_eq!(
                expected, actual,
                "{context}: normalized denial reason diverged"
            );
        }
        (expected, actual) => panic!(
            "{context}: owner/oracle result shape diverged: expected {expected:?}, actual {actual:?}"
        ),
    }
}

pub(crate) fn retirement_denial(denial: &SignalBranchRetirementDenial) -> ModelDenial {
    match denial {
        worth_signal::facade::branch::SignalBranchRetirementDenial::OwnerUnavailable(_) => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::OwnerUnavailable,
        worth_signal::facade::branch::SignalBranchRetirementDenial::CancelledNoMovement => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::Cancelled,
        worth_signal::facade::branch::SignalBranchRetirementDenial::UnknownBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::UnknownBranch,
        worth_signal::facade::branch::SignalBranchRetirementDenial::RetiredBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetiredBranch,
        worth_signal::facade::branch::SignalBranchRetirementDenial::CurrentBranch { .. }
        | worth_signal::facade::branch::SignalBranchRetirementDenial::CanonicalBranch { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::CurrentBranch,
        worth_signal::facade::branch::SignalBranchRetirementDenial::CanonicalBasisMismatch
        | worth_signal::facade::branch::SignalBranchRetirementDenial::StaleBranchHead { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::StaleBasis,
        worth_signal::facade::branch::SignalBranchRetirementDenial::RetainedComponentBasis { .. }
        | worth_signal::facade::branch::SignalBranchRetirementDenial::RetainedAdmittedBasis { .. }
        | worth_signal::facade::branch::SignalBranchRetirementDenial::SharedAdmittedBasis { .. } => crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::RetainedBasis,
        worth_signal::facade::branch::SignalBranchRetirementDenial::ForeignRetirementSnapshot { .. }
        | worth_signal::facade::branch::SignalBranchRetirementDenial::RetirementSnapshotBranchMismatch { .. } => {
            crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial::ForeignSnapshot
        }
        other => panic!("unexpected retirement denial in adjacency oracle: {other:?}"),
    }
}
