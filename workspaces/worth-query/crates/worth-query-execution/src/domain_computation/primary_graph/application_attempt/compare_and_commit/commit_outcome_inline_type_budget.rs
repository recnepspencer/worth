//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryApplicationCommitOutcome>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/compare_and_commit/commit_outcome.rs", "WorthQueryApplicationCommitOutcome", 128 * 1024);
    }
    {
        let _owner = census.owner::<WorthQueryApplicationUncommitted>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/compare_and_commit/commit_outcome.rs", "WorthQueryApplicationUncommitted", 128 * 1024);
    }
    super::denial::inline_type_budget::collect(census);
}
