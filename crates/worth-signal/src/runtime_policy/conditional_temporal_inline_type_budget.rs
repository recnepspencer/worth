//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalConditionalTemporalBudget>(
            "crates/worth-signal/src/runtime_policy/conditional_temporal.rs",
            "SignalConditionalTemporalBudget",
            128 * 1024,
        );
        owner.field::<Option<usize>>("maximum_live_partitions");
        owner.field::<Option<usize>>("maximum_reserved_active_wakes");
    }
}
