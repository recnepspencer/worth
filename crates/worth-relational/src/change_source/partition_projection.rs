use crate::identity::data::PartitionId;
use crate::publication::patch::data::PublishedAuthoritativePatchEnvelope;
use crate::transactions::data::RecordRef;

pub(super) struct PartitionPatchProjection {
    pub(super) patch: PublishedAuthoritativePatchEnvelope,
    pub(super) records_examined: u64,
    pub(super) records_filtered_out: u64,
}

/// Keep only the records in `partition_id`, or every record when it is
/// `None`, and count what was kept and dropped.
pub(super) fn project_patch_partition(
    patch: PublishedAuthoritativePatchEnvelope,
    partition_id: Option<PartitionId>,
) -> PartitionPatchProjection {
    let records_examined = patch.authoritative_record_patches.len() as u64;
    let mut patch = patch;
    if let Some(partition_id) = partition_id {
        patch
            .authoritative_record_patches
            .retain(|record| record_partition(&record.target) == partition_id);
    }
    let retained = patch.authoritative_record_patches.len() as u64;
    PartitionPatchProjection {
        patch,
        records_examined,
        records_filtered_out: records_examined - retained,
    }
}

fn record_partition(record: &RecordRef) -> PartitionId {
    match record {
        RecordRef::Entity(entity) => entity.partition_id,
        RecordRef::Relation(relation) => relation.partition_id,
    }
}
