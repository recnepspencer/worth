//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryApplicationAttemptBasis>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/application_attempt_affinity.rs", "WorthQueryApplicationAttemptBasis", 128 * 1024);
        owner.field::<WorthQueryApplicationAttemptCore>("core");
        owner.field::<Option<super::super::super::super::provider::OriginalCompletionReceiver>>(
            "original_completion",
        );
        owner.field::<crate::basis::WorthQueryProductBranchLease>("product");
    }
}
