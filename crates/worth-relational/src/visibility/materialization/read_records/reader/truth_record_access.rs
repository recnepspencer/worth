use super::projection_record_reader::ProjectionRecordReader;
use crate::storage::{
    data::{EntityReadRecord, RelationReadRecord},
    overlay::PartitionAccess,
    partition::DenseSlotBitSet,
};

#[path = "truth_record_access/runtime_reads.rs"]
mod runtime_reads;

pub(super) fn sort_authoritative_relation_records(records: &mut [RelationReadRecord]) {
    records.sort_unstable_by_key(|record| {
        (
            record.source.partition_id.0,
            record.source.local_slot.0,
            record.target.partition_id.0,
            record.target.local_slot.0,
            record.relation_id.partition_id.0,
            record.relation_id.local_slot.0,
        )
    });
}
