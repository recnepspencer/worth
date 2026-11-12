//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryPreparedApplicationCommit>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/provider/application_attempt_state/commit_preparation.rs", "WorthQueryPreparedApplicationCommit", 128 * 1024);
        owner.field::<WorthQueryPrimaryGraphApplicationAttempt>("attempt");
        owner.field::<worth_relational::facade::mvcc::ValidatedRelationalProposal>("candidate");
        owner.field::<Option<crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner>>("predicate");
        owner.field::<WorthQueryPrimaryMutationWorkCounters>("work");
        owner.field::<Option<WorthQueryRetainedPreImage>>("retained_preimage");
        owner.field::<WorthQueryPreImageRetentionWork>("preimage_retention_work");
        owner.field::<relational_commit::PreparedSourceFactRebase>("source_fact_rebase");
        owner.field::<super::super::commit_completion::WorthQueryApplicationAttemptCompletion>(
            "_completion",
        );
    }
    super::relational_commit::inline_type_budget::collect(census);
}
