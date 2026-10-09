//! Root admission is a pre-effect refusal at resource admission.

use super::{
    WorthQueryApplicationCommitDenial as Denial, WorthQueryApplicationCommitDenialStage as Stage,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial as Stop, WorthQueryApplicationCommitOutcome as Outcome,
    WorthQueryManagedComputationInterruption as Interruption,
};

impl Stop {
    pub fn into_commit_outcome(self) -> Outcome {
        let mut denial = match self {
            Self::Resource(cause) => Denial::execution_resource(cause, None, None, "request admission refused"),
            Self::NestedOpening => Denial::execution_resource(
                crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::NestedAdvancementOpening,
                None, None, "second advancement opening refused",
            ),
            Self::ForeignPhase => Denial::execution_resource(
                crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ForeignAdvancementPhase,
                None, None, "advancement belongs to another installed runtime",
            ),
            Self::NestedStopped => Denial::execution_nested_stopped(None, "request admission stopped"),
            // The body may already have published. Admission cannot promise
            // a pre-effect denial after an unwind outside a checked pattern.
            Self::Panicked => return Outcome::Indeterminate(
                crate::domain_computation::primary_graph::application_attempt::provider_execution::unknown_commit_recovery_evidence(
                    "advancement panicked outside a checked pattern",
                ),
            ),
            Self::Interrupted(Interruption::Cancelled) => return Outcome::Cancelled,
            Self::Interrupted(Interruption::DeadlineExceeded) => return Outcome::TimedOut,
        };
        denial.stage = Stage::ResourceAdmission;
        Outcome::Denied(denial)
    }
}
