use std::sync::Arc;

use super::region_image::*;
use super::{RelationalColdRegion, RelationalResidencyDemandFailure as Failure, ResidentRegion};
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::{
    partition_from_image, partition_to_image_with_contracts,
};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::partition::AdjacencyKindBasis;

pub(super) fn load_head(
    cold: &RelationalColdRegion,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    limit: u64,
) -> Result<Arc<ResidentRegion>, Failure> {
    let (partition, frontiers) = (|| -> Result<_, Cause> {
        check_member_limit(cold.head.length, limit)?;
        let bytes = cold.source.read_member(cold.head, limit)?;
        let frame = RelationalRegionImage::decode(
            &bytes,
            cold.head,
            cold.partition_id,
            cold.schema_carrier_digest,
            RelationalRegionImageKind::Head,
        )?;
        let head: RegionHeadImage =
            rmp_serde::from_slice(frame.payload).map_err(|_| Cause::Damaged)?;
        super::kind_commitment::verify(&head.kind_projection, cold.kind_authority.head)?;
        if head.partition.partition_id != cold.partition_id {
            return Err(Cause::Damaged);
        }
        let contracts = contracts(cold)?;
        let mut partition =
            partition_from_image(head.partition, cold.schema.aspect_plans(), &contracts)
                .map_err(|_| Cause::Damaged)?;
        super::kind_projection::rebuild(
            &mut partition,
            &head.kind_projection,
            AdjacencyKindBasis::Current,
        )?;
        verify_buckets(
            &partition,
            &head.adjacency_kinds,
            &head.reverse_adjacency_kinds,
            AdjacencyKindBasis::Current,
        )?;
        Ok((partition, head.history_frontiers))
    })()
    .map_err(|cause| unavailable(cold, cause))?;
    cold.symbols
        .demand_symbol_ids(execution, super::required_symbols::collect(&partition))?;
    let symbols = cold.symbols.interner_snapshot();
    let commitment = PartitionContentCommitment::capture_head(&partition, &symbols, &frontiers)
        .ok_or_else(|| unavailable(cold, Cause::Damaged))?;
    verify_leaf(cold, &partition, &commitment).map_err(|cause| unavailable(cold, cause))?;
    crate::execution::check_request(execution).map_err(Failure::Request)?;
    Ok(Arc::new(ResidentRegion {
        partition,
        commitment: Some(commitment),
    }))
}

pub(super) fn load_full(
    cold: &RelationalColdRegion,
    head: &ResidentRegion,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    limit: u64,
) -> Result<Arc<ResidentRegion>, Failure> {
    let partition = (|| -> Result<_, Cause> {
        check_member_limit(cold.history.length, limit)?;
        let bytes = cold.source.read_member(cold.history, limit)?;
        let frame = RelationalRegionImage::decode(
            &bytes,
            cold.history,
            cold.partition_id,
            cold.schema_carrier_digest,
            RelationalRegionImageKind::History,
        )?;
        let history: RegionHistoryImage =
            rmp_serde::from_slice(frame.payload).map_err(|_| Cause::Damaged)?;
        super::kind_commitment::verify(&history.kind_projection, cold.kind_authority.history)?;
        let contracts = contracts(cold)?;
        let mut image = partition_to_image_with_contracts(
            head.partition.clone(),
            cold.schema.aspect_plans(),
            &contracts,
        )
        .map_err(|_| Cause::Damaged)?;
        if history.entities.len() != image.entity_arena.metadata_history.len()
            || history.relations.len() != image.relation_arena.metadata_history.len()
        {
            return Err(Cause::Damaged);
        }
        for (current, mut earlier) in image
            .entity_arena
            .metadata_history
            .iter_mut()
            .zip(history.entities)
        {
            earlier.append(current);
            *current = earlier;
        }
        for (current, mut earlier) in image
            .relation_arena
            .metadata_history
            .iter_mut()
            .zip(history.relations)
        {
            earlier.append(current);
            *current = earlier;
        }
        let mut partition = partition_from_image(image, cold.schema.aspect_plans(), &contracts)
            .map_err(|_| Cause::Damaged)?;
        partition.adjacency = head.partition.adjacency.clone();
        partition.reverse_adjacency = head.partition.reverse_adjacency.clone();
        super::kind_projection::rebuild(
            &mut partition,
            &history.kind_projection,
            AdjacencyKindBasis::Historical,
        )?;
        verify_buckets(
            &partition,
            &history.adjacency_kinds,
            &history.reverse_adjacency_kinds,
            AdjacencyKindBasis::Historical,
        )?;
        Ok(partition)
    })()
    .map_err(|cause| unavailable(cold, cause))?;
    cold.symbols
        .demand_symbol_ids(execution, super::required_symbols::collect(&partition))?;
    let symbols = cold.symbols.interner_snapshot();
    let (commitment, _) = PartitionContentCommitment::capture(&partition, &symbols, None)
        .map_err(|_| unavailable(cold, Cause::Damaged))?;
    verify_leaf(cold, &partition, &commitment).map_err(|cause| unavailable(cold, cause))?;
    crate::execution::check_request(execution).map_err(Failure::Request)?;
    Ok(Arc::new(ResidentRegion {
        partition,
        commitment: Some(commitment),
    }))
}

fn unavailable(cold: &RelationalColdRegion, cause: Cause) -> RelationalStorageUnavailable {
    RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Region(cold.partition_id),
        cause,
    }
}

fn contracts(cold: &RelationalColdRegion) -> Result<CheckpointAspectContractCatalog, Cause> {
    CheckpointAspectContractCatalog::from_contracts(cold.schema.retained_aspect_contracts())
        .map_err(|_| Cause::Damaged)
}

fn check_member_limit(length: u64, limit: u64) -> Result<(), Cause> {
    if length > limit {
        return Err(Cause::LimitExceeded { length, limit });
    }
    Ok(())
}

fn verify_leaf(
    cold: &RelationalColdRegion,
    partition: &PartitionState,
    commitment: &PartitionContentCommitment,
) -> Result<(), Cause> {
    if commitment.digest(cold.partition_id) != cold.content_digest
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

fn verify_buckets(
    partition: &PartitionState,
    outgoing: &[RegionAdjacencyKinds],
    incoming: &[RegionAdjacencyKinds],
    basis: AdjacencyKindBasis,
) -> Result<(), Cause> {
    if super::region_capture::capture_kinds(&partition.adjacency, basis) != outgoing
        || super::region_capture::capture_kinds(&partition.reverse_adjacency, basis) != incoming
    {
        return Err(Cause::IntegrityMismatch);
    }
    Ok(())
}
