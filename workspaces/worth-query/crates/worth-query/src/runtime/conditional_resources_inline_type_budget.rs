//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryConditionalExecutionResources>(
            "workspaces/worth-query/crates/worth-query/src/runtime/conditional_resources.rs",
            "WorthQueryConditionalExecutionResources",
            128 * 1024,
        );
        owner.field::<WorthQueryConditionalEvaluationCacheBudget>("query_cache");
        owner.field::<SignalConditionalEvaluationBudget>("signal_evaluations");
    }
}
