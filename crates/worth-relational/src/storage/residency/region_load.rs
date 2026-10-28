use std::collections::BTreeMap;
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
    let (mut partition, frontiers, mut historical) = (|| -> Result<_, Cause> {
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
        super::kind_commitment::verify(&head.history_kind_projection, cold.kind_authority.history)?;
        if head.partition.partition_id != cold.partition_id {
            return Err(Cause::Damaged);
        }
        if head.partition.entity_arena.metadata_history.len()
            != head.partition.entity_arena.generations.len()
            || head.partition.relation_arena.metadata_history.len()
                != head.partition.relation_arena.generations.len()
        {
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
        super::kind_projection::rebuild_head_history(
            &mut partition,
            &head.history_kind_projection,
        )?;
        let mut historical = BTreeMap::<usize, Vec<_>>::new();
        for claim in head.history_kind_projection {
            if claim.relation.partition_id == cold.partition_id {
                historical
                    .entry(claim.relation.slot_index())
                    .or_default()
                    .push(claim);
            }
        }
        Ok((partition, head.history_frontiers, historical))
    })()
    .map_err(|cause| unavailable(cold, cause))?;
    cold.symbols
        .demand_symbol_ids(execution, super::required_symbols::collect(&partition))?;
    let symbols = cold.symbols.interner_snapshot();
    let commitment = PartitionContentCommitment::capture_head(&partition, &symbols, &frontiers)
        .ok_or_else(|| unavailable(cold, Cause::Damaged))?;
    verify_leaf(cold, &partition, &commitment).map_err(|cause| unavailable(cold, cause))?;
    let history_custody = cold
        .source
        .retain_members(
            &cold
                .history_members
                .members(cold.history)
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .map_err(|cause| unavailable(cold, cause))?;
    let (entity_openings, relation_openings) = frontiers.into_arenas();
    install_history_prefixes(
        &mut partition.entity_arena,
        cold,
        &history_custody,
        entity_openings,
        &mut BTreeMap::new(),
    )?;
    install_history_prefixes(
        &mut partition.relation_arena,
        cold,
        &history_custody,
        relation_openings,
        &mut historical,
    )?;
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
    let reference = RegionHistoryBase {
        member: cold.history,
        partition_id: cold.partition_id,
        schema_carrier_digest: cold.schema_carrier_digest,
        row_index: 0,
        kind_commitment: cold.kind_authority.history,
    };
    let mut folder = super::history_fold::HistoryFolder::new(execution, cold.partition_id, limit);
    let history = folder.read(cold.source.as_ref(), &cold.history_members, &reference)?;
    super::kind_commitment::verify(&history.kind_projection, cold.kind_authority.history)
        .map_err(|cause| unavailable(cold, cause))?;
    if history.entities.len() != head.partition.entity_arena.slot_count()
        || history.relations.len() != head.partition.relation_arena.slot_count()
    {
        return Err(unavailable(cold, Cause::Damaged).into());
    }
    let entities = folder.fold_rows::<crate::storage::substrate::EntityRecordKind>(
        cold.source.as_ref(),
        &cold.history_members,
        &reference,
        history.entities.len(),
    )?;
    let relations = folder.fold_rows::<crate::storage::substrate::RelationRecordKind>(
        cold.source.as_ref(),
        &cold.history_members,
        &reference,
        history.relations.len(),
    )?;
    drop(folder);
    let partition = (|| -> Result<_, Cause> {
        let contracts = contracts(cold)?;
        let mut image = partition_to_image_with_contracts(
            head.partition.clone(),
            cold.schema.aspect_plans(),
            &contracts,
        )
        .map_err(|_| Cause::Damaged)?;
        for (current, mut earlier) in image.entity_arena.metadata_history.iter_mut().zip(entities) {
            earlier.append(current);
            *current = earlier;
        }
        for (current, mut earlier) in image
            .relation_arena
            .metadata_history
            .iter_mut()
            .zip(relations)
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

fn install_history_prefixes<K: crate::storage::substrate::RecordKind>(
    arena: &mut crate::storage::substrate::RecordArena<K>,
    cold: &RelationalColdRegion,
    custody: &crate::durability::medium::RelationalDurableMemberCustody,
    openings: Vec<(u64, crate::storage::substrate::HistoryAppendFrontier)>,
    historical: &mut BTreeMap<usize, Vec<crate::storage::partition::RegionRelationProjection>>,
) -> Result<(), Failure> {
    for (slot, opening) in openings {
        let row_index = usize::try_from(slot)
            .ok()
            .and_then(|slot| arena.physical_index(slot))
            .ok_or_else(|| unavailable(cold, Cause::Damaged))?;
        if !arena.metadata_history[row_index].is_empty() {
            arena.history_prefixes.set(
                row_index,
                Some(crate::storage::substrate::RecordHistoryPrefix {
                    source: cold.source.clone(),
                    _placement_custody: custody.clone(),
                    member: cold.history,
                    history_members: cold.history_members.clone(),
                    partition_id: cold.partition_id,
                    schema_carrier_digest: cold.schema_carrier_digest,
                    row_index,
                    history_opening: crate::storage::substrate::StorageAllocation::new(
                        crate::storage::substrate::RecordHistoryOpening {
                            frontier: opening,
                            kind_commitment: cold.kind_authority.history,
                            historical_relations: historical
                                .remove(&(slot as usize))
                                .unwrap_or_default(),
                        },
                    ),
                }),
            );
        }
    }
    Ok(())
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
