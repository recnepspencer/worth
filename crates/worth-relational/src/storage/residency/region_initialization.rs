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
        lane: &ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut partition: PartitionState,
        symbols: &StringInterner,
    ) -> Result<Self, RegionContentFailure> {
        let kind_indexes = super::kind_admission::recover(
            lane,
            execution,
            partition.partition_id,
            |checkpoint, retain| {
                super::kind_admission::normalize_value(lane, &mut partition, checkpoint, retain)?;
                super::kind_incremental::RegionKindIndexes::capture_controlled(
                    lane, &partition, checkpoint,
                )
            },
        )
        .map(Arc::new)
        .map_err(RegionContentFailure::from)?;
        partition.clear_runtime_pin_counters();
        let (commitment, hashed) =
            PartitionContentCommitment::capture(lane, execution, &partition, symbols)
                .map_err(RegionContentFailure::Content)?;
        let inventory = partition.allocation_inventory();
        let unavailable_records = partition.entity_arena.lifecycle_counts().unavailable
            + partition.relation_arena.lifecycle_counts().unavailable;
        let kind_commitment = kind_indexes.commitment();
        let directory = lane
            .recover(execution, |lane, work| {
                super::RegionRecordDirectory::capture(lane, &partition, &mut |units, bytes| {
                    Ok(work.prepare(units, bytes)?)
                })
            })
            .map_err(|error| error.into_content_failure(partition.partition_id))?;
        let live_counts = super::LiveRecordCounts::from_partition(&partition);
        let entity_slots = partition.entity_arena.slot_count();
        let relation_slots = partition.relation_arena.slot_count();
        let content_digest = commitment.digest(partition.partition_id);
        let requirements =
            super::required_symbols::CapturedSymbolRequirements::capture(lane, &partition, true);
        Ok(Self {
            partition_id: partition.partition_id,
            retention: RelationalRegionRetention::from_lease(RelationalRegionResidency::resident(
                partition,
                commitment,
                kind_indexes,
                Some(requirements),
            )),
            inventory,
            content_digest,
            kind_commitment,
            directory,
            live_counts,
            entity_slots,
            relation_slots,
            content_values_hashed: hashed,
            unavailable_records,
            copied_truth_bytes: 0,
            new_payload_bytes: inventory.authoritative_bytes,
        })
    }
}
