//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalPartitionConditionalUnwind>(
            "crates/worth-signal/src/data/graph/storage/evaluation_partition/unwind.rs",
            "SignalPartitionConditionalUnwind",
            128 * 1024,
        );
        owner.field::<SignalPartitionConditionalUnwindReason>("reason");
        owner.field::<SignalRejectedConditionalEvaluation>("rejected");
    }
}
