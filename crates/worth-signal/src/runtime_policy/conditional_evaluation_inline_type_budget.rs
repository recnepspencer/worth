//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalConditionalEvaluationBudget>(
            "crates/worth-signal/src/runtime_policy/conditional_evaluation.rs",
            "SignalConditionalEvaluationBudget",
            128 * 1024,
        );
        owner.field::<Option<usize>>("maximum_retained_slots");
        owner.field::<Option<u64>>("maximum_retained_bytes");
        owner.field::<Option<usize>>("maximum_attempt_visits");
    }
}
