use std::collections::BTreeMap;

use crate::identity::data::{EntityId, KindId, RelationId};
use crate::indexes::data::{
    RelationJoinDefinition, RelationJoinEntry, RelationJoinKey, RelationJoinLeg,
    RelationJoinSharedEndpoint,
};
use crate::storage::data::RelationReadRecord;

use super::checked_entry_map::{checked_push, IndexKernelStop};
use super::IndexProjectionSource;

pub(in crate::indexes) fn build_relation_join_index_checked(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &IndexProjectionSource<'_>,
    definition: RelationJoinDefinition,
    context: &mut crate::execution::PacketKernelContext<'_, '_, '_>,
) -> Result<BTreeMap<RelationJoinKey, Vec<RelationJoinEntry>>, IndexKernelStop> {
    let mut left = BTreeMap::new();
    let mut right = BTreeMap::new();
    collect_join_relations_checked(
        request_execution,
        projection,
        definition.left(),
        definition.shared_entity_kind(),
        &mut left,
        context,
    )?;
    collect_join_relations_checked(
        request_execution,
        projection,
        definition.right(),
        definition.shared_entity_kind(),
        &mut right,
        context,
    )?;
    let mut entries = BTreeMap::new();
    for (shared_entity, left_relations) in left {
        context.checkpoint(1)?;
        let Some(right_relations) = right.get(&shared_entity) else {
            continue;
        };
        for (left_entity, left_relation) in left_relations {
            context.checkpoint(1)?;
            for (right_entity, right_relation) in right_relations {
                context.checkpoint(1)?;
                checked_push(
                    &mut entries,
                    RelationJoinKey::new(left_entity, *right_entity),
                    RelationJoinEntry::new(shared_entity, left_relation, *right_relation),
                    0,
                    0,
                    context,
                )?;
            }
        }
    }
    for rows in entries.values_mut() {
        context.checkpoint(rows.len() as u64)?;
        rows.sort();
        rows.dedup_by_key(|entry| entry.shared_entity_id());
    }
    Ok(entries)
}

fn collect_join_relations_checked(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &IndexProjectionSource<'_>,
    leg: RelationJoinLeg,
    shared_entity_kind: KindId,
    collected: &mut BTreeMap<EntityId, BTreeMap<EntityId, RelationId>>,
    context: &mut crate::execution::PacketKernelContext<'_, '_, '_>,
) -> Result<(), IndexKernelStop> {
    let budget = std::cell::RefCell::new(context);
    projection
        .try_for_each_relation(
            request_execution,
            leg.relation_kind(),
            |bytes| {
                budget.borrow_mut().checkpoint(1)?;
                budget.borrow().check_scratch_peak(bytes)
            },
            |relation| {
                let (shared, external) = join_endpoints(relation, leg.shared_endpoint());
                budget.borrow_mut().checkpoint(2)?;
                budget
                    .borrow()
                    .check_scratch_peak(projection.candidate_entity_bytes(shared))?;
                budget
                    .borrow()
                    .check_scratch_peak(projection.candidate_entity_bytes(external))?;
                if projection
                    .with_entity(request_execution, shared, |record| record.kind.kind_id)
                    .map_err(super::checked_entry_map::read_request)?
                    != Some(shared_entity_kind)
                    || projection
                        .with_entity(request_execution, external, |record| record.kind.kind_id)
                        .map_err(super::checked_entry_map::read_request)?
                        != Some(leg.external_entity_kind())
                {
                    return Ok(());
                }
                let new_shared = !collected.contains_key(&shared);
                let new_external = !collected
                    .get(&shared)
                    .is_some_and(|rows| rows.contains_key(&external));
                if new_shared || new_external {
                    let bytes = u64::from(new_shared)
                        * (std::mem::size_of::<(EntityId, BTreeMap<EntityId, RelationId>)>()
                            + 3 * std::mem::size_of::<usize>()) as u64
                        + u64::from(new_external)
                            * (std::mem::size_of::<(EntityId, RelationId)>()
                                + 3 * std::mem::size_of::<usize>())
                                as u64;
                    budget.borrow_mut().claim_scratch(bytes)?;
                }
                collected
                    .entry(shared)
                    .or_default()
                    .entry(external)
                    .and_modify(|selected| *selected = (*selected).min(relation.relation_id))
                    .or_insert(relation.relation_id);
                Ok(())
            },
        )
        .map_err(super::checked_entry_map::read_stop)
}

pub(in crate::indexes) fn build_relation_join_index(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &IndexProjectionSource<'_>,
    definition: RelationJoinDefinition,
) -> Result<
    BTreeMap<RelationJoinKey, Vec<RelationJoinEntry>>,
    crate::execution::RelationalExecutionDenialCause,
> {
    let left = collect_join_relations(
        execution,
        projection,
        definition.left(),
        definition.shared_entity_kind(),
    )?;
    let right = collect_join_relations(
        execution,
        projection,
        definition.right(),
        definition.shared_entity_kind(),
    )?;
    let mut entries = BTreeMap::<RelationJoinKey, Vec<RelationJoinEntry>>::new();
    for (shared_entity, left_relations) in left {
        let Some(right_relations) = right.get(&shared_entity) else {
            continue;
        };
        for (left_entity, left_relation) in left_relations {
            for (right_entity, right_relation) in right_relations {
                entries
                    .entry(RelationJoinKey::new(left_entity, *right_entity))
                    .or_default()
                    .push(RelationJoinEntry::new(
                        shared_entity,
                        left_relation,
                        *right_relation,
                    ));
            }
        }
    }
    for joined in entries.values_mut() {
        joined.sort();
        joined.dedup_by_key(|entry| entry.shared_entity_id());
    }
    Ok(entries)
}

fn collect_join_relations(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &IndexProjectionSource<'_>,
    leg: RelationJoinLeg,
    shared_entity_kind: KindId,
) -> Result<
    BTreeMap<EntityId, BTreeMap<EntityId, RelationId>>,
    crate::execution::RelationalExecutionDenialCause,
> {
    let mut collected = BTreeMap::<EntityId, BTreeMap<EntityId, RelationId>>::new();
    projection.for_each_relation(execution, leg.relation_kind(), |relation| {
        let (shared, external) = join_endpoints(relation, leg.shared_endpoint());
        if projection.with_entity(execution, shared, |record| record.kind.kind_id)?
            != Some(shared_entity_kind)
            || projection.with_entity(execution, external, |record| record.kind.kind_id)?
                != Some(leg.external_entity_kind())
        {
            return Ok(());
        }
        collected
            .entry(shared)
            .or_default()
            .entry(external)
            .and_modify(|selected| *selected = (*selected).min(relation.relation_id))
            .or_insert(relation.relation_id);
        Ok(())
    })?;
    Ok(collected)
}

pub(in crate::indexes) const fn join_endpoints(
    relation: &RelationReadRecord,
    shared_endpoint: RelationJoinSharedEndpoint,
) -> (EntityId, EntityId) {
    match shared_endpoint {
        RelationJoinSharedEndpoint::Source => (relation.source, relation.target),
        RelationJoinSharedEndpoint::Target => (relation.target, relation.source),
    }
}
