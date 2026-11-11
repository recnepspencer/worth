//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalEvaluationPartition>(
            "crates/worth-signal/src/data/graph/storage/evaluation_partition.rs",
            "SignalEvaluationPartition",
            128 * 1024,
        );
        owner.field::<SignalExecutionDefinitions>("definitions");
        owner.field::<SignalEvaluationStorage>("evaluation");
        owner.field::<EvaluationObservationStorage>("observation");
        owner.field::<TraversalResources>("traversal");
        owner.field::<Option<SignalPartitionConditionalUnwind>>("pending_unwind");
        owner
            .field::<Option<crate::data::retained_storage::SignalConditionalRetentionReservation>>(
                "_slot_custody",
            );
    }
    super::conditional_execution::inline_type_budget::collect(census);
    super::unwind::inline_type_budget::collect(census);
}
