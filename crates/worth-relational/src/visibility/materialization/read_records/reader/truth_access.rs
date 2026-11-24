use super::kind_scan_visibility::KindScanVisibility;
use crate::storage::data::{EntityReadRecord, RecordLifecycleState, RelationReadRecord};

#[path = "truth_access/runtime_reads.rs"]
mod runtime_reads;

pub(super) fn authoritative_entity_records_are_canonical(records: &[EntityReadRecord]) -> bool {
    records.windows(2).all(|window| {
        let left = &window[0];
        let right = &window[1];
        (
            left.entity_id.partition_id.0,
            left.entity_id.local_slot.0,
            left.entity_id.generation,
        ) <= (
            right.entity_id.partition_id.0,
            right.entity_id.local_slot.0,
            right.entity_id.generation,
        )
    })
}
