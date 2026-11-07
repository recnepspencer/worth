//! Initial owned region image; there is no maintained predecessor to update.
use super::{
    PreparedResidentRegion, RegionContentFailure, RelationalRegionResidency,
    RelationalRegionRetention,
};
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::symbols::data::StringInterner;
use std::sync::Arc;
impl PreparedResidentRegion {
    pub(crate) fn initial(
        mut partition: PartitionState,
        symbols: &StringInterner,
    ) -> Result<Self, RegionContentFailure> {
        ReconstructionLane::enter_reconstruction(|lane| {
            normalize(lane, &mut partition)?;
            let (commitment, hashed) =
                PartitionContentCommitment::capture(&partition, symbols, None)
                    .map_err(RegionContentFailure::Content)?;
            let inventory = partition.allocation_inventory();
            let unavailable_records = partition.entity_arena.lifecycle_counts().unavailable
                + partition.relation_arena.lifecycle_counts().unavailable;
            let kind_indexes = Arc::new(
                super::kind_incremental::RegionKindIndexes::capture(lane, &partition)
                    .map_err(|_| RegionContentFailure::Adjacency)?,
            );
            let kind_commitment = kind_indexes.commitment();
            let live_counts = super::LiveRecordCounts::from_partition(&partition);
            let entity_slots = partition.entity_arena.slot_count();
            let relation_slots = partition.relation_arena.slot_count();
            let content_digest = commitment.digest(partition.partition_id);
            Ok(Self {
                partition_id: partition.partition_id,
                retention: RelationalRegionRetention::from_lease(
                    RelationalRegionResidency::resident(partition, commitment, kind_indexes),
                ),
                inventory,
                content_digest,
                kind_commitment,
                live_counts,
                entity_slots,
                relation_slots,
                content_values_hashed: hashed,
                unavailable_records,
                copied_truth_bytes: 0,
                new_payload_bytes: inventory.authoritative_bytes,
            })
        })
    }
}

fn normalize(
    lane: &ReconstructionLane<'_>,
    partition: &mut PartitionState,
) -> Result<(), RegionContentFailure> {
    super::rebuild_region_history_kind_buckets(lane, partition)
        .map_err(|_| RegionContentFailure::Adjacency)?;
    partition.clear_runtime_pin_counters();
    Ok(())
}
