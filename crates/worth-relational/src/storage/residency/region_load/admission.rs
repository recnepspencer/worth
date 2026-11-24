//! Shared admission rules for authenticated region images.
use super::*;
pub(super) fn unavailable(
    cold: &RelationalColdRegion,
    cause: Cause,
) -> RelationalStorageUnavailable {
    RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Region(cold.partition_id),
        cause,
    }
}

pub(super) fn install_history_prefixes<K: crate::storage::substrate::RecordKind>(
    arena: &mut crate::storage::substrate::RecordArena<K>,
    cold: &RelationalColdRegion,
    custody: &Arc<crate::storage::substrate::RecordHistoryCustody>,
    openings: Vec<(u64, crate::storage::substrate::HistoryAppendFrontier)>,
    historical: &mut BTreeMap<usize, Vec<crate::storage::partition::RegionRelationProjection>>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Failure> {
    use crate::storage::substrate::{RecordHistoryOpening, RecordHistoryPrefix, StorageAllocation};
    for (slot, opening) in openings {
        funding::work(request, 1)?;
        let row_index = usize::try_from(slot)
            .ok()
            .and_then(|slot| arena.physical_index(slot))
            .ok_or_else(|| unavailable(cold, Cause::Damaged))?;
        if arena.metadata_history[row_index].is_empty() {
            continue;
        }
        let claims = historical.remove(&(slot as usize)).unwrap_or_default();
        let memory = funding::opening(&opening, &claims, claims.capacity(), request)?;
        let prefix_memory = funding::Funding::reserve(
            StorageAllocation::<Option<RecordHistoryPrefix>>::layout_bytes(),
            request,
        )?;
        let prefix = RecordHistoryPrefix {
            custody: custody.clone(),
            member: cold.history,
            history_members: cold.history_members.clone(),
            partition_id: cold.partition_id,
            schema_carrier_digest: cold.schema_carrier_digest,
            row_index,
            history_opening: StorageAllocation::from_reserved(
                RecordHistoryOpening {
                    frontier: opening,
                    kind_commitment: *cold.kind_authority.history.bytes(),
                    historical_relations: claims,
                },
                memory.into_memory(),
            ),
        };
        arena
            .history_prefixes
            .set_shared_for_request(
                row_index,
                StorageAllocation::from_reserved(Some(prefix), prefix_memory.into_memory()),
                request,
            )
            .map_err(Failure::Request)?;
    }
    Ok(())
}

pub(super) fn contracts(
    cold: &RelationalColdRegion,
) -> Result<CheckpointAspectContractCatalog, Cause> {
    CheckpointAspectContractCatalog::from_contracts(cold.schema.retained_aspect_contracts())
        .map_err(|_| Cause::Damaged)
}

pub(super) fn check_member_limit(length: u64, limit: u64) -> Result<(), Cause> {
    if length > limit {
        return Err(Cause::LimitExceeded { length, limit });
    }
    Ok(())
}

pub(super) fn verify_leaf(
    cold: &RelationalColdRegion,
    partition: &PartitionState,
    commitment: &PartitionContentCommitment,
) -> Result<(), Cause> {
    if commitment.digest(cold.partition_id) != cold.content_digest
        || super::super::LiveRecordCounts::from_partition(partition) != cold.live_counts
        || partition.entity_arena.slot_count() != cold.entity_slots
        || partition.relation_arena.slot_count() != cold.relation_slots
        || partition.entity_arena.lifecycle_counts().unavailable
            + partition.relation_arena.lifecycle_counts().unavailable
            != cold.unavailable_records
    {
        return Err(Cause::IntegrityMismatch);
    }
    Ok(())
}

pub(super) fn verify_buckets(
    lane: &ReconstructionLane<'_>,
    partition: &PartitionState,
    outgoing: &[RegionAdjacencyKinds],
    incoming: &[RegionAdjacencyKinds],
    basis: AdjacencyKindBasis,
    checkpoint: &mut super::super::kind_projection::ProjectionCheckpoint<'_>,
) -> Result<(), Cause> {
    if super::super::region_capture::capture_kinds(lane, &partition.adjacency, basis, checkpoint)?
        != outgoing
        || super::super::region_capture::capture_kinds(
            lane,
            &partition.reverse_adjacency,
            basis,
            checkpoint,
        )? != incoming
    {
        return Err(Cause::IntegrityMismatch);
    }
    Ok(())
}
