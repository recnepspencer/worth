//! Root admission is a pre-effect refusal at resource admission.

use super::{
    WorthQueryApplicationCommitDenial as Denial, WorthQueryApplicationCommitDenialStage as Stage,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial as Stop, WorthQueryApplicationCommitOutcome as Outcome,
    WorthQueryApplicationUncommitted as Uncommitted,
    WorthQueryManagedComputationInterruption as Interruption,
};

impl Stop {
    pub fn into_commit_outcome(self) -> Outcome {
        self.into_uncommitted().into()
    }

    /// An admission stop cannot carry a committed receipt. An unwind still
    /// retains indeterminate recovery custody because the body may have run.
    pub fn into_uncommitted(self) -> Uncommitted {
        let mut denial = match self {
            Self::Resource(cause) => Denial::execution_resource(cause, None, None, "request admission refused"),
            Self::NestedOpening => Denial::execution_resource(
                crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::NestedAdvancementOpening,
                None, None, "second advancement opening refused",
            ),
            Self::NestedStopped => Denial::execution_nested_stopped(None, "request admission stopped"),
            // The body may already have published. Admission cannot promise
            // a pre-effect denial after an unwind outside a checked pattern.
            Self::Panicked => return Uncommitted::Indeterminate(
                crate::domain_computation::primary_graph::application_attempt::provider_execution::unknown_commit_recovery_evidence(
                    "advancement panicked outside a checked pattern",
                ),
            ),
            Self::Interrupted(Interruption::Cancelled) => return Uncommitted::Cancelled,
            Self::Interrupted(Interruption::DeadlineExceeded) => return Uncommitted::TimedOut,
        };
        denial.stage = Stage::ResourceAdmission;
        Uncommitted::Denied(denial)
    }
}
