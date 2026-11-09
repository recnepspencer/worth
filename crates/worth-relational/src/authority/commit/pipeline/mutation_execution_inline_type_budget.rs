//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<MutatedCommitExecution>(
            "crates/worth-relational/src/authority/commit/pipeline/mutation_execution.rs",
            "MutatedCommitExecution",
            128 * 1024,
        );
        owner.field::<PreparedCommitState>("prepared");
        owner.field::<WorkingState>("working_state");
        owner.field::<Vec<crate::validation::engine::InvariantExecutionResult>>(
            "invariant_executions",
        );
        owner.field::<MutationApplyOutcome>("outcome");
        owner.field::<PendingMutationIdentities>("mutation_identities");
        owner.field::<Option<crate::aggregate_identity::AggregateProposal>>("_aggregate");
        owner.field::<std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>>("source_interpretation");
        owner.field::<Option<crate::aggregate_identity::AggregateAdmission>>("aggregate_admission");
    }
}
