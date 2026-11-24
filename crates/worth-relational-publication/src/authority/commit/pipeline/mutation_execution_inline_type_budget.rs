//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;
use worth_relational_mutation::internal_port as mutation;

pub(crate) fn collect(
    census: &mut worth_relational_contracts::internal_port::inline_type_sizes::census::Census,
) {
    {
        let mut owner = census.owner::<MutatedCommitExecution>(
            "crates/worth-relational/src/authority/commit/pipeline/mutation_execution.rs",
            "MutatedCommitExecution",
            128 * 1024,
        );
        owner.field::<PreparedCommitState>("prepared");
        owner.field::<WorkingState>("working_state");
        owner.field::<Vec<mutation::validation::engine::result::InvariantExecutionResult>>(
            "invariant_executions",
        );
        owner.field::<MutationApplyOutcome>("outcome");
        owner.field::<PendingMutationIdentities>("mutation_identities");
        owner.field::<Option<mutation::aggregate_identity::proposal::AggregateProposal>>(
            "_aggregate",
        );
        owner.field::<std::sync::Arc<mutation::validation::candidate_source::interpretation::NativeManagedSourceInterpretation>>("source_interpretation");
        owner.field::<Option<mutation::aggregate_identity::preparation::AggregateAdmission>>(
            "aggregate_admission",
        );
    }
}
