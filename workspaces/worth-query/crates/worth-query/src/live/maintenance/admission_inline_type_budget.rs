//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryMaintenanceScope>(
            "workspaces/worth-query/crates/worth-query/src/live/maintenance/admission.rs",
            "WorthQueryMaintenanceScope",
            128 * 1024,
        );
        owner.field::<u32>("ExactSourceRecord.partition_id");
        owner.field::<u64>("ExactSourceRecord.local_slot");
        owner.field::<u32>("ExactSourceRecord.generation");
    }
}
