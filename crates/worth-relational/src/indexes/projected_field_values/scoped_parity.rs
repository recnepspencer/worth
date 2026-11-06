//! Independent certification from the requested endpoints' authoritative adjacency.
use super::{compare_related_entries, entity_aspect_field_ordering_value, IndexProjectionSource};
use crate::execution::RelationalExecutionDenialCause;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::indexes::data::{
    RelatedEntityEndpoint, RelatedEntityOrderingEntry, RelatedEntityOrderingField,
    RelationJoinDefinition, RelationJoinEntry, RelationJoinLeg, RelationJoinSharedEndpoint,
};
use std::collections::BTreeMap;

/// Cost: the selected parent's degree, field projections and local ordering.
pub(in crate::indexes) fn ordered_children_for_parent(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &IndexProjectionSource<'_>,
    relation_kind: KindId,
    endpoint: RelatedEntityEndpoint,
    child_kind: KindId,
    ordering: &[RelatedEntityOrderingField],
    parent: EntityId,
) -> Result<Vec<RelatedEntityOrderingEntry>, RelationalExecutionDenialCause> {
    let outgoing = endpoint == RelatedEntityEndpoint::SourceParent;
    let mut rows = Vec::new();
    for relation in source.relations_for_endpoint(execution, parent, relation_kind, outgoing)? {
        let (observed_parent, child) = if outgoing {
            (relation.source, relation.target)
        } else {
            (relation.target, relation.source)
        };
        if observed_parent != parent
            || source.with_entity(execution, child, |r| r.kind.kind_id)? != Some(child_kind)
        {
            continue;
        }
        let values = ordering
            .iter()
            .map(|field| {
                entity_aspect_field_ordering_value(execution, source, child, field.locator())
            })
            .collect::<Result<Option<Vec<_>>, _>>()?;
        if let Some(values) = values {
            rows.push(RelatedEntityOrderingEntry::new(
                values,
                child,
                relation.relation_id,
            ));
        }
    }
    rows.sort_by(|a, b| compare_related_entries(a, b, ordering));
    Ok(rows)
}

/// Cost: the two requested external endpoints' degrees plus a keyed intersection.
pub(in crate::indexes) fn joined_rows_for_pair(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &IndexProjectionSource<'_>,
    join: RelationJoinDefinition,
    left: EntityId,
    right: EntityId,
) -> Result<Vec<RelationJoinEntry>, RelationalExecutionDenialCause> {
    let left = leg_by_shared(
        execution,
        source,
        join.left(),
        join.shared_entity_kind(),
        left,
    )?;
    let right = leg_by_shared(
        execution,
        source,
        join.right(),
        join.shared_entity_kind(),
        right,
    )?;
    Ok(left
        .into_iter()
        .filter_map(|(shared, left)| {
            right
                .get(&shared)
                .map(|right| RelationJoinEntry::new(shared, left, *right))
        })
        .collect())
}

fn leg_by_shared(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &IndexProjectionSource<'_>,
    leg: RelationJoinLeg,
    shared_kind: KindId,
    external: EntityId,
) -> Result<BTreeMap<EntityId, RelationId>, RelationalExecutionDenialCause> {
    let mut shared = BTreeMap::new();
    if source.with_entity(execution, external, |r| r.kind.kind_id)?
        != Some(leg.external_entity_kind())
    {
        return Ok(shared);
    }
    let outgoing = leg.shared_endpoint() == RelationJoinSharedEndpoint::Target;
    for relation in
        source.relations_for_endpoint(execution, external, leg.relation_kind(), outgoing)?
    {
        let (hub, observed_external) = super::join_endpoints(&relation, leg.shared_endpoint());
        if observed_external == external
            && source.with_entity(execution, hub, |r| r.kind.kind_id)? == Some(shared_kind)
        {
            shared
                .entry(hub)
                .and_modify(|id: &mut RelationId| *id = (*id).min(relation.relation_id))
                .or_insert(relation.relation_id);
        }
    }
    Ok(shared)
}
