use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, SignalBranchRetentionLease,
    SignalBranchRetirementDenial,
};

use super::super::super::comparison::{neutral_basis, neutral_snapshot};
use super::super::super::state::{ModelObservation, ModelSnapshot};
use super::super::super::transition::{ModelDenial, ModelResult, ModelSuccess};

/// Normalized result from a real public owner-port call.
///
/// The wrapper deliberately keeps only neutral observations and the owner-issued
/// values needed to continue the second half of an adjacent pair. It never
/// calls production semantic helpers to decide what the result means.
#[derive(Debug)]
pub(super) enum RealResult {
    Success(RealSuccess),
    Denied(ModelDenial),
}

#[derive(Debug)]
pub(super) enum RealSuccess {
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

pub(super) fn assert_equivalent(expected: &ModelResult, actual: &RealResult, context: &str) {
    match (expected, actual) {
        (
            ModelResult::Success(ModelSuccess::Fork(expected)),
            RealResult::Success(RealSuccess::Fork {
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
            ModelResult::Success(ModelSuccess::Advance(expected)),
            RealResult::Success(RealSuccess::Advance { observation, basis }),
        )
        | (
            ModelResult::Success(ModelSuccess::Restore(expected)),
            RealResult::Success(RealSuccess::Restore { observation, basis }),
        ) => {
            assert_eq!(expected, observation, "{context}: movement observation diverged");
            assert_eq!(
                observation,
                &neutral_basis(basis),
                "{context}: movement basis diverged"
            );
        }
        (
            ModelResult::Success(ModelSuccess::Capture {
                observation: expected_observation,
                snapshot: expected_snapshot,
            }),
            RealResult::Success(RealSuccess::Capture {
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
            ModelResult::Success(ModelSuccess::Lease),
            RealResult::Success(RealSuccess::Lease(_)),
        )
        | (
            ModelResult::Success(ModelSuccess::Release),
            RealResult::Success(RealSuccess::Release { .. }),
        )
        | (
            ModelResult::Success(ModelSuccess::Retirement),
            RealResult::Success(RealSuccess::Retirement),
        )
        | (
            ModelResult::Success(ModelSuccess::Closed),
            RealResult::Success(RealSuccess::Closed),
        ) => {}
        (ModelResult::Denied(expected), RealResult::Denied(actual)) => {
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

pub(super) fn retirement_denial(denial: &SignalBranchRetirementDenial) -> ModelDenial {
    match denial {
        SignalBranchRetirementDenial::OwnerUnavailable(_) => ModelDenial::OwnerUnavailable,
        SignalBranchRetirementDenial::CancelledNoMovement => ModelDenial::Cancelled,
        SignalBranchRetirementDenial::UnknownBranch { .. } => ModelDenial::UnknownBranch,
        SignalBranchRetirementDenial::RetiredBranch { .. } => ModelDenial::RetiredBranch,
        SignalBranchRetirementDenial::CurrentBranch { .. }
        | SignalBranchRetirementDenial::CanonicalBranch { .. } => ModelDenial::CurrentBranch,
        SignalBranchRetirementDenial::CanonicalBasisMismatch
        | SignalBranchRetirementDenial::StaleBranchHead { .. } => ModelDenial::StaleBasis,
        SignalBranchRetirementDenial::RetainedComponentBasis { .. }
        | SignalBranchRetirementDenial::RetainedAdmittedBasis { .. }
        | SignalBranchRetirementDenial::SharedAdmittedBasis { .. } => ModelDenial::RetainedBasis,
        SignalBranchRetirementDenial::ForeignRetirementSnapshot { .. }
        | SignalBranchRetirementDenial::RetirementSnapshotBranchMismatch { .. } => {
            ModelDenial::ForeignSnapshot
        }
        other => panic!("unexpected retirement denial in adjacency oracle: {other:?}"),
    }
}
