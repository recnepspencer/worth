//! Closed correspondence to each existing typed program's actual admission.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::effect_program::WorthQueryCommitEntryRefusal;
mod sealed {
    pub trait Sealed {}
}

/// A foreign committed receipt cannot become this carrier's refusal.
/// ```compile_fail,E0308
/// use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::entry_custody::CommitCarrier;
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitOutcome;
/// fn cannot_substitute<S, P: CommitCarrier<S>>(program: P, unrelated: WorthQueryApplicationCommitOutcome) {
///     program.refuse(unrelated);
/// }
/// ```
pub trait CommitCarrier<Schema>: sealed::Sealed {
    type Operation;
    type Input;
    type Scope;
    type Refusal;
    fn refuse(self, refusal: WorthQueryCommitEntryRefusal) -> Self::Refusal;
    fn admission(
        &self,
    ) -> &WorthQueryAdmittedApplicationOperation<Schema, Self::Operation, Self::Input, Self::Scope>;
}

impl<S, O, I, E> sealed::Sealed for WorthQueryApplicationEffectProgram<S, O, I, E> {}
impl<S, O, I, E> CommitCarrier<S> for WorthQueryApplicationEffectProgram<S, O, I, E> {
    type Operation = O;
    type Input = I;
    type Scope = E;
    type Refusal = WorthQueryApplicationCommitOutcome;
    fn refuse(self, refusal: WorthQueryCommitEntryRefusal) -> Self::Refusal {
        refusal.into_outcome()
    }
    fn admission(&self) -> &WorthQueryAdmittedApplicationOperation<S, O, I, E> {
        &self.read_set.admission
    }
}

macro_rules! lifecycle_carrier {
    ($program:ident, $outcome:ty, $take:ident, $convert:ident, $missing:expr) => {
        impl<S, O, I, E> sealed::Sealed for $program<S, O, I, E> {}
        impl<S, O, I, E> CommitCarrier<S> for $program<S, O, I, E> {
            type Operation = O;
            type Input = I;
            type Scope = E;
            type Refusal = $outcome;
            fn admission(&self) -> &WorthQueryAdmittedApplicationOperation<S, O, I, E> {
                &self.application_program().read_set.admission
            }
            fn refuse(self, refusal: WorthQueryCommitEntryRefusal) -> Self::Refusal {
                let outcome = refusal.into_outcome();
                let mut program = self.into_inner();
                let Some(binding) = program.read_set.admission.$take() else {
                    return $missing;
                };
                $convert(outcome, binding)
            }
        }
    };
}
use crate::domain_computation::primary_graph::application_attempt::{
    approved_outcome, closed_outcome, requested_outcome, reviewed_outcome,
};
use crate::domain_computation::primary_graph::{
    WorthQueryElevationApprovalOutcome, WorthQueryElevationCloseOutcome,
    WorthQueryElevationRequestOutcome, WorthQueryMandatoryReviewOutcome,
};
lifecycle_carrier!(
    WorthQueryElevationRequestProgram,
    WorthQueryElevationRequestOutcome,
    take_elevation_request_binding,
    requested_outcome,
    WorthQueryElevationRequestOutcome::Denied(
        WorthQueryApplicationCommitDenial::elevation_request_program_mismatch()
    )
);
lifecycle_carrier!(
    WorthQueryElevationApprovalProgram,
    WorthQueryElevationApprovalOutcome,
    take_elevation_approval_binding,
    approved_outcome,
    WorthQueryElevationApprovalOutcome::Indeterminate
);
lifecycle_carrier!(
    WorthQueryElevationCloseProgram,
    WorthQueryElevationCloseOutcome,
    take_elevation_close_binding,
    closed_outcome,
    WorthQueryElevationCloseOutcome::Indeterminate
);
lifecycle_carrier!(
    WorthQueryMandatoryReviewProgram,
    WorthQueryMandatoryReviewOutcome,
    take_mandatory_review_binding,
    reviewed_outcome,
    WorthQueryMandatoryReviewOutcome::Indeterminate
);
macro_rules! commit_carrier {
    ($program:ident) => {
        impl<S, O, I, E> sealed::Sealed for $program<S, O, I, E> {}
        impl<S, O, I, E> CommitCarrier<S> for $program<S, O, I, E> {
            type Operation = O;
            type Input = I;
            type Scope = E;
            type Refusal = WorthQueryApplicationCommitOutcome;
            fn admission(&self) -> &WorthQueryAdmittedApplicationOperation<S, O, I, E> {
                &self.application_program().read_set.admission
            }
            fn refuse(self, refusal: WorthQueryCommitEntryRefusal) -> Self::Refusal {
                refusal.into_outcome()
            }
        }
    };
}
commit_carrier!(WorthQueryDelegationActivationProgram);
commit_carrier!(WorthQueryCapabilityRevocationProgram);
