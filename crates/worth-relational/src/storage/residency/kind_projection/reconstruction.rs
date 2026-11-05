use super::super::RelationalResidencyDemandFailure as Failure;
use super::*;
use crate::identity::data::KindId;
use worth_execution::ExecutionRequest;

enum RebuildStop {
    Content(Cause),
    Request(crate::execution::RelationalExecutionDenialCause),
}
impl From<Cause> for RebuildStop {
    fn from(cause: Cause) -> Self {
        Self::Content(cause)
    }
}
fn check(request: Option<&ExecutionRequest<'_, '_>>) -> Result<(), RebuildStop> {
    if let Some(request) = request {
        request
            .consult()
            .map_err(|cause| RebuildStop::Request(cause.into()))?;
    }
    Ok(())
}
fn ordinary(cause: RebuildStop) -> Cause {
    match cause {
        RebuildStop::Content(cause) => cause,
        RebuildStop::Request(_) => unreachable!("ordinary projection has no request"),
    }
}
fn funded(cause: RebuildStop, partition: crate::identity::data::PartitionId) -> Failure {
    match cause {
        RebuildStop::Content(cause) => crate::storage::data::RelationalStorageUnavailable {
            subject: crate::storage::data::RelationalStorageSubject::Region(partition),
            cause,
        }
        .into(),
        RebuildStop::Request(cause) => Failure::Request(cause),
    }
}

/// Owned relations are reconstructed from record metadata. Peer relations use
/// only the projection authenticated by this region's canonical root leaf.
pub(in crate::storage::residency) fn rebuild(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    basis: AdjacencyKindBasis,
) -> Result<(), Cause> {
    rebuild_using(partition, projection, basis, None).map_err(ordinary)
}
pub(in crate::storage::residency) fn rebuild_for_request(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    basis: AdjacencyKindBasis,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Failure> {
    let id = partition.partition_id;
    rebuild_using(partition, projection, basis, Some(request)).map_err(|cause| funded(cause, id))
}
fn rebuild_using(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    basis: AdjacencyKindBasis,
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<(), RebuildStop> {
    let mut expected = BTreeMap::<RelationId, RegionRelationProjection>::new();
    for entry in projection {
        check(request)?;
        if expected.insert(entry.relation, entry.clone()).is_some() {
            return Err(Cause::Damaged.into());
        }
    }
    let historical = matches!(basis, AdjacencyKindBasis::Historical);
    if historical {
        // Discard every supplied claim about locally owned history, and derive
        // it again from the authenticated arena, including retired generations.
        expected.retain(|id, _| id.partition_id != partition.partition_id);
        project_owned_history(partition, &mut expected, request)?;
        if expected.values().cloned().collect::<Vec<_>>() != projection {
            return Err(Cause::IntegrityMismatch.into());
        }
    }
    install_projection(partition, &expected, basis, request)?;
    if !historical {
        for entry in expected
            .values()
            .filter(|entry| entry.relation.partition_id == partition.partition_id)
        {
            let view = partition
                .relation_arena
                .get_slot(entry.relation.slot_index())
                .ok_or(Cause::Damaged)?;
            if view.generation() != entry.relation.generation.0
                || view.kind_id() != Some(entry.kind)
            {
                return Err(Cause::IntegrityMismatch.into());
            }
            let mut local = BTreeMap::new();
            add_owned(
                partition,
                &mut local,
                entry.relation,
                entry.kind,
                view.extra().endpoints.as_ref().ok_or(Cause::Damaged)?,
            )?;
            if local.get(&entry.relation) != Some(entry) {
                return Err(Cause::IntegrityMismatch.into());
            }
        }
    }
    Ok(())
}

/// The digest is authenticated by the caller. Head can corroborate current
/// owned claims; Full must later reconstruct all owned historical claims.
pub(in crate::storage::residency) fn rebuild_head_history_for_request(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Failure> {
    let id = partition.partition_id;
    rebuild_head_using(partition, projection, Some(request)).map_err(|cause| funded(cause, id))
}
fn rebuild_head_using(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<(), RebuildStop> {
    let mut expected = BTreeMap::new();
    for entry in projection {
        check(request)?;
        if expected.insert(entry.relation, entry.clone()).is_some() {
            return Err(Cause::Damaged.into());
        }
    }
    let mut known = BTreeMap::new();
    project_owned_history(partition, &mut known, request)?;
    for (relation, claim) in known {
        let supplied = expected.get(&relation).ok_or(Cause::IntegrityMismatch)?;
        if claim.kind != supplied.kind
            || !claim.outgoing.is_subset(&supplied.outgoing)
            || !claim.incoming.is_subset(&supplied.incoming)
        {
            return Err(Cause::IntegrityMismatch.into());
        }
    }
    install_projection(
        partition,
        &expected,
        AdjacencyKindBasis::Historical,
        request,
    )
}

fn install_projection(
    partition: &mut PartitionState,
    expected: &BTreeMap<RelationId, RegionRelationProjection>,
    basis: AdjacencyKindBasis,
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<(), RebuildStop> {
    let historical = matches!(basis, AdjacencyKindBasis::Historical);
    for (incoming, table) in [
        (false, &mut partition.adjacency),
        (true, &mut partition.reverse_adjacency),
    ] {
        if let Some(request) = request {
            let original = table.clone();
            for (&slot, _) in original.iter() {
                check(Some(request))?;
                let adjacency = table
                    .get_mut_for_request(slot, request)
                    .map_err(RebuildStop::Request)?
                    .ok_or(Cause::IntegrityMismatch)?;
                if historical {
                    adjacency.reset_historical_kind_buckets();
                } else {
                    adjacency.reset_kind_buckets();
                }
            }
        } else {
            for (_, adjacency) in table.iter_mut() {
                if historical {
                    adjacency.reset_historical_kind_buckets();
                } else {
                    adjacency.reset_kind_buckets();
                }
            }
        }
        for entry in expected.values() {
            check(request)?;
            let slots = if incoming {
                &entry.incoming
            } else {
                &entry.outgoing
            };
            for &slot in slots {
                check(request)?;
                let adjacency = match request {
                    Some(request) => table
                        .get_mut_for_request(slot, request)
                        .map_err(RebuildStop::Request)?,
                    None => table.get_mut(slot),
                }
                .ok_or(Cause::IntegrityMismatch)?;
                match (historical, request) {
                    (true, Some(request)) => adjacency
                        .index_historical_kind_for_request(entry.kind, entry.relation, request)
                        .map_err(RebuildStop::Request)?,
                    (false, Some(request)) => adjacency
                        .index_current_kind_for_request(entry.kind, entry.relation, request)
                        .map_err(RebuildStop::Request)?,
                    (true, None) => adjacency.index_historical_kind(entry.kind, entry.relation),
                    (false, None) => adjacency.index_current_kind(entry.kind, entry.relation),
                }
            }
        }
        if !historical {
            for (_, adjacency) in table.iter() {
                check(request)?;
                let ids: BTreeSet<_> = adjacency.current_ids().iter().copied().collect();
                let indexed: BTreeSet<_> = adjacency
                    .checkpoint_kind_memberships(basis)
                    .into_iter()
                    .flat_map(|(_, ids)| ids)
                    .collect();
                if ids != indexed {
                    return Err(Cause::IntegrityMismatch.into());
                }
            }
        }
    }
    Ok(())
}

fn add_owned(
    partition: &PartitionState,
    target: &mut BTreeMap<RelationId, RegionRelationProjection>,
    relation: RelationId,
    kind: KindId,
    endpoints: &crate::storage::substrate::RelationEndpoints,
) -> Result<(), Cause> {
    let entry = target
        .entry(relation)
        .or_insert_with(|| RegionRelationProjection {
            relation,
            kind,
            outgoing: BTreeSet::new(),
            incoming: BTreeSet::new(),
        });
    if entry.kind != kind {
        return Err(Cause::IntegrityMismatch);
    }
    if endpoints.source.partition_id == partition.partition_id
        && partition
            .adjacency
            .get(endpoints.source.slot_index())
            .is_some()
    {
        entry.outgoing.insert(endpoints.source.slot_index());
    }
    if endpoints.target.partition_id == partition.partition_id
        && partition
            .reverse_adjacency
            .get(endpoints.target.slot_index())
            .is_some()
    {
        entry.incoming.insert(endpoints.target.slot_index());
    }
    if entry.outgoing.is_empty() && entry.incoming.is_empty() {
        target.remove(&relation);
    }
    Ok(())
}

pub(in crate::storage::residency) fn canonical_history(
    partition: &PartitionState,
) -> Result<Vec<RegionRelationProjection>, Cause> {
    let mut expected = capture(partition, AdjacencyKindBasis::Historical)?
        .into_iter()
        .filter(|entry| entry.relation.partition_id != partition.partition_id)
        .map(|entry| (entry.relation, entry))
        .collect();
    project_owned_history(partition, &mut expected, None).map_err(ordinary)?;
    Ok(expected.into_values().collect())
}

fn project_owned_history(
    partition: &PartitionState,
    expected: &mut BTreeMap<RelationId, RegionRelationProjection>,
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<(), RebuildStop> {
    for slot in partition.relation_arena.occupied_slots_iter() {
        check(request)?;
        project_prefix(partition, slot, expected, request)?;
        for metadata in partition
            .relation_arena
            .metadata_history_at(slot)
            .into_iter()
            .flatten()
        {
            check(request)?;
            add_owned(
                partition,
                expected,
                RelationId::new(partition.partition_id, slot as u64, metadata.generation),
                metadata.kind_id,
                &metadata.endpoints,
            )?;
        }
        if let Some(view) = partition.relation_arena.get_slot(slot)
            && let (Some(kind), Some(endpoints)) = (view.kind_id(), view.extra().endpoints.as_ref())
        {
            check(request)?;
            add_owned(
                partition,
                expected,
                RelationId::new(partition.partition_id, slot as u64, view.generation()),
                kind,
                endpoints,
            )?;
        }
    }
    Ok(())
}

fn project_prefix(
    partition: &PartitionState,
    slot: usize,
    expected: &mut BTreeMap<RelationId, RegionRelationProjection>,
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<(), RebuildStop> {
    let physical = partition
        .relation_arena
        .physical_index(slot)
        .ok_or(Cause::IntegrityMismatch)?;
    let Some(prefix) = &partition.relation_arena.history_prefixes[physical] else {
        return Ok(());
    };
    for claim in &prefix.history_opening.historical_relations {
        check(request)?;
        if claim.relation.partition_id != partition.partition_id
            || claim.relation.slot_index() != slot
        {
            return Err(Cause::IntegrityMismatch.into());
        }
        let entry = expected
            .entry(claim.relation)
            .or_insert_with(|| RegionRelationProjection {
                relation: claim.relation,
                kind: claim.kind,
                outgoing: BTreeSet::new(),
                incoming: BTreeSet::new(),
            });
        if entry.kind != claim.kind {
            return Err(Cause::IntegrityMismatch.into());
        }
        for (&slot, incoming) in claim
            .outgoing
            .iter()
            .map(|slot| (slot, false))
            .chain(claim.incoming.iter().map(|slot| (slot, true)))
        {
            check(request)?;
            if incoming {
                if partition.reverse_adjacency.get(slot).is_some() {
                    entry.incoming.insert(slot);
                }
            } else if partition.adjacency.get(slot).is_some() {
                entry.outgoing.insert(slot);
            }
        }
        if entry.outgoing.is_empty() && entry.incoming.is_empty() {
            expected.remove(&claim.relation);
        }
    }
    Ok(())
}
