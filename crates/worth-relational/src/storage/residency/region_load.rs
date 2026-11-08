mod head_validation;
pub(super) mod materialization_funding;
use super::region_image::*;
use super::resident_history::projection_funding;
use super::{RelationalColdRegion, RelationalResidencyDemandFailure as Failure, ResidentRegion};
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::{
    partition_from_image_for_request, partition_to_image_with_contracts,
};
use crate::identity::data::PartitionId;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::partition::AdjacencyKindBasis;
use materialization_funding as funding;
use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) fn load_head(
    cold: &RelationalColdRegion,
    source: &Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    limit: u64,
) -> Result<Arc<ResidentRegion>, Failure> {
    let (_contract_memory, contract_bound) = funding::contracts(&cold.schema, execution)?;
    check_member_limit(cold.head.length, limit).map_err(|cause| unavailable(cold, cause))?;
    let _read = funding::Funding::reserve(
        funding::vector::<u8>(usize::try_from(cold.head.length).map_err(|_| funding::overflow())?)?,
        execution,
    )?;
    funding::work(execution, cold.head.length)?;
    let bytes = source
        .read_member(cold.head, limit)
        .map_err(|cause| unavailable(cold, cause))?;
    let frame = RelationalRegionImage::decode(
        &bytes,
        cold.head,
        cold.partition_id,
        cold.schema_carrier_digest,
        RelationalRegionImageKind::Head,
    )
    .map_err(|cause| unavailable(cold, cause))?;
    let quote = funding::region_image::head(funding::Raw(frame.payload), contract_bound, execution)
        .map_err(|cause| funding::at_region(cause, cold.partition_id))?;
    // Capacity quotes reserve backing; decoding pays for the bytes it visits.
    funding::work(execution, frame.payload.len() as u64)?;
    let memory = funding::MaterializationFunding::admit(quote, execution)?;
    let (mut partition, frontiers, mut historical) = (|| -> Result<_, Failure> {
        let head: RegionHeadImage =
            rmp_serde::from_slice(frame.payload).map_err(|_| unavailable(cold, Cause::Damaged))?;
        head_validation::verify_kind(&head.kind_projection, cold.kind_authority.head, execution)
            .map_err(|cause| funding::at_region(cause, cold.partition_id))?;
        head_validation::verify_kind(
            &head.history_kind_projection,
            cold.kind_authority.history,
            execution,
        )
        .map_err(|cause| funding::at_region(cause, cold.partition_id))?;
        if head.partition.partition_id != cold.partition_id {
            return Err(unavailable(cold, Cause::Damaged).into());
        }
        if head.partition.entity_arena.metadata_history.len()
            != head.partition.entity_arena.generations.len()
            || head.partition.relation_arena.metadata_history.len()
                != head.partition.relation_arena.generations.len()
        {
            return Err(unavailable(cold, Cause::Damaged).into());
        }
        let contracts = contracts(cold).map_err(|cause| unavailable(cold, cause))?;
        let mut partition = partition_from_image_for_request(
            head.partition,
            cold.schema.aspect_plans(),
            &contracts,
            execution,
        )
        .map_err(|cause| funding::image(cause, cold.partition_id))?;
        super::kind_projection::rebuild_for_request(
            &mut partition,
            &head.kind_projection,
            AdjacencyKindBasis::Current,
            execution,
        )?;
        head_validation::verify_buckets(
            &partition,
            &head.adjacency_kinds,
            &head.reverse_adjacency_kinds,
            execution,
        )
        .map_err(|cause| funding::at_region(cause, cold.partition_id))?;
        super::kind_projection::rebuild_head_history_for_request(
            &mut partition,
            &head.history_kind_projection,
            execution,
        )?;
        let mut historical = BTreeMap::<usize, Vec<_>>::new();
        for claim in head.history_kind_projection {
            funding::work(execution, 1 + 11 * (historical.len() + 1).ilog2() as u64)?;
            if claim.relation.partition_id == cold.partition_id {
                historical
                    .entry(claim.relation.slot_index())
                    .or_default()
                    .push(claim);
            }
        }
        Ok((partition, head.history_frontiers, historical))
    })()?;
    cold.symbols.demand_symbol_ids(
        execution,
        super::required_symbols::collect_for_request(&partition, execution)?,
    )?;
    let symbols = cold.symbols.interner_snapshot();
    let commitment = PartitionContentCommitment::capture_head_for_request(
        &partition, &symbols, &frontiers, execution,
    )
    .map_err(|cause| funding::content(cause, cold.partition_id))?;
    verify_leaf(cold, &partition, &commitment).map_err(|cause| unavailable(cold, cause))?;
    let history_custody = funding::history_custody::capture(cold, source, execution)?;
    let (entity_openings, relation_openings) = frontiers.into_arenas();
    install_history_prefixes(
        &mut partition.entity_arena,
        cold,
        &history_custody,
        entity_openings,
        &mut BTreeMap::new(),
        execution,
    )?;
    install_history_prefixes(
        &mut partition.relation_arena,
        cold,
        &history_custody,
        relation_openings,
        &mut historical,
        execution,
    )?;
    crate::execution::check_request(execution).map_err(Failure::Request)?;
    Ok(Arc::new(ResidentRegion {
        partition,
        commitment: Some(commitment),
        _history_custody: Some(history_custody),
        _memory: memory.retained.into_memory(),
    }))
}

pub(super) fn load_full(
    cold: &RelationalColdRegion,
    source: &Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    head: &ResidentRegion,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    limit: u64,
) -> Result<Arc<ResidentRegion>, Failure> {
    let (_contract_memory, contract_bound) = funding::contracts(&cold.schema, execution)?;
    let (_projection_memory, _) = projection_funding(&head.partition, execution)?;
    let retained = cold.schema.retained_aspect_contracts();
    let current_quote = funding::current(&head.partition, contract_bound, retained, execution)?;
    let _current_wire = funding::Funding::reserve(current_quote.wire, execution)?;
    let reference = RegionHistoryBase {
        member: cold.history,
        partition_id: cold.partition_id,
        schema_carrier_digest: cold.schema_carrier_digest,
        row_index: 0,
        kind_commitment: *cold.kind_authority.history.bytes(),
    };
    let mut folder = super::history_fold::HistoryFolder::new(execution, cold.partition_id, limit);
    let history = folder.read(source.as_ref(), &cold.history_members, &reference)?;
    super::kind_commitment::verify(&history.image.kind_projection, cold.kind_authority.history)
        .map_err(|cause| unavailable(cold, cause))?;
    if history.image.entities.len() != head.partition.entity_arena.slot_count()
        || history.image.relations.len() != head.partition.relation_arena.slot_count()
    {
        return Err(unavailable(cold, Cause::Damaged).into());
    }
    let mut entities = folder.fold_rows::<crate::storage::substrate::EntityRecordKind>(
        source.as_ref(),
        &cold.history_members,
        &reference,
        history.image.entities.len(),
    )?;
    let mut relations = folder.fold_rows::<crate::storage::substrate::RelationRecordKind>(
        source.as_ref(),
        &cold.history_members,
        &reference,
        history.image.relations.len(),
    )?;
    drop(folder);
    let (partition, retained) = (|| -> Result<_, Failure> {
        let contracts = contracts(cold).map_err(|cause| unavailable(cold, cause))?;
        let mut image = partition_to_image_with_contracts(
            head.partition.clone(),
            cold.schema.aspect_plans(),
            &contracts,
        )
        .map_err(|_| unavailable(cold, Cause::Damaged))?;
        for (current, earlier) in image
            .entity_arena
            .metadata_history
            .iter_mut()
            .zip(&mut entities.rows)
        {
            earlier.append_current(current, execution)?;
            *current = std::mem::take(&mut earlier.values);
        }
        for (current, earlier) in image
            .relation_arena
            .metadata_history
            .iter_mut()
            .zip(&mut relations.rows)
        {
            earlier.append_current(current, execution)?;
            *current = std::mem::take(&mut earlier.values);
        }
        let retained = funding::Funding::reserve(funding::shared::<ResidentRegion>()?, execution)?;
        let mut partition = partition_from_image_for_request(
            image,
            cold.schema.aspect_plans(),
            &contracts,
            execution,
        )
        .map_err(|cause| funding::image(cause, cold.partition_id))?;
        partition.adjacency = head.partition.adjacency.clone();
        partition.reverse_adjacency = head.partition.reverse_adjacency.clone();
        super::kind_projection::rebuild_for_request(
            &mut partition,
            &history.image.kind_projection,
            AdjacencyKindBasis::Historical,
            execution,
        )?;
        verify_buckets(
            &partition,
            &history.image.adjacency_kinds,
            &history.image.reverse_adjacency_kinds,
            AdjacencyKindBasis::Historical,
        )
        .map_err(|cause| unavailable(cold, cause))?;
        Ok((partition, retained))
    })()?;
    cold.symbols.demand_symbol_ids(
        execution,
        super::required_symbols::collect_for_request(&partition, execution)?,
    )?;
    let symbols = cold.symbols.interner_snapshot();
    let (commitment, _) =
        PartitionContentCommitment::capture_full_for_request(&partition, &symbols, execution)
            .map_err(|cause| funding::content(cause, cold.partition_id))?;
    verify_leaf(cold, &partition, &commitment).map_err(|cause| unavailable(cold, cause))?;
    crate::execution::check_request(execution).map_err(Failure::Request)?;
    Ok(Arc::new(ResidentRegion {
        partition,
        commitment: Some(commitment),
        _history_custody: None,
        _memory: retained.into_memory(),
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

pub(in crate::storage::residency) fn at_region(cause: Failure, partition: PartitionId) -> Failure {
    match cause {
        Failure::Request(cause) => Failure::Request(cause),
        Failure::Content(cause) => Failure::Content(cause),
        Failure::StorageUnavailable(mut cause) => {
            cause.subject = RelationalStorageSubject::Region(partition);
            cause.into()
        }
    }
}

pub(in crate::storage::residency) fn scalars<T: crate::storage::substrate::StoredValue>(
    raw: funding::Raw<'_>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<funding::Quote, Failure> {
    let count = funding::sequence(raw, request, |_| Ok(()))?;
    Ok(funding::Quote::bytes(
        funding::vector::<T::Value>(count)?,
        funding::column::<T>(count)?,
    ))
}

pub(in crate::storage::residency) fn content(
    cause: crate::storage::overlay::PartitionContentDigestError,
    partition: crate::identity::data::PartitionId,
) -> Failure {
    use crate::storage::overlay::PartitionContentDigestError as Error;
    match cause {
        Error::Request(cause) => Failure::Request(cause),
        Error::AllocationAccountingOverflow => funding::overflow(),
        Error::UnresolvedContentSymbol(symbol) => RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Symbols {
                first: symbol,
                last: symbol,
            },
            cause: Cause::SourceUnavailable,
        }
        .into(),
        Error::InvalidHistoryCoverage | Error::Encoding => {
            funding::at_region(funding::damaged(), partition)
        }
    }
}
