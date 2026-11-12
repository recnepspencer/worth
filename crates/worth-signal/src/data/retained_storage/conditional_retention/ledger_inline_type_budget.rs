//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalConditionalRetentionLedger>(
            "crates/worth-signal/src/data/retained_storage/conditional_retention/ledger.rs",
            "SignalConditionalRetentionLedger",
            128 * 1024,
        );
        owner.field::<Option<usize>>("maximum_slots");
        owner.field::<Option<u64>>("maximum_bytes");
        owner.field::<Option<usize>>("maximum_partitions");
        owner.field::<Option<usize>>("maximum_reserved_wakes");
        owner.field::<Mutex<RetainedConditionalUsage>>("state");
    }
}
