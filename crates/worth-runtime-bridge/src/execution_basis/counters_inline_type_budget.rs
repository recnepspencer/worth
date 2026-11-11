//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeExecutionBasisCounters>(
            "crates/worth-runtime-bridge/src/execution_basis/counters.rs",
            "BridgeExecutionBasisCounters",
            128 * 1024,
        );
        owner.field::<usize>("managed_intent_check_count");
        owner.field::<usize>("signal_attempt_check_count");
        owner.field::<usize>("signal_attempt_admission_count");
        owner.field::<usize>("signal_queue_binding_count");
        owner.field::<usize>("truth_basis_check_count");
        owner.field::<usize>("reservation_check_count");
        owner.field::<usize>("truth_materialization_count");
    }
}
