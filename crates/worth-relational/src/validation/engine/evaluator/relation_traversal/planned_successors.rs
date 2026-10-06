use std::collections::BTreeMap;

use super::super::super::context::InvariantExecutionContext;
use crate::transactions::data::EntityReference;

pub(super) type PlannedSuccessorMap = BTreeMap<EntityReference, Vec<EntityReference>>;

pub(super) fn planned_successor_map(
    planned_edges: &[super::super::super::request::PlannedRelationEdge],
    context: &InvariantExecutionContext<'_, '_>,
) -> PlannedSuccessorMap {
    let mut successors = BTreeMap::new();
    for edge in planned_edges {
        if !context.checkpoint(1) {
            return successors;
        }
        let owned = [&edge.source, &edge.target]
            .into_iter()
            .map(|reference| match reference {
                EntityReference::Existing(_) => 0,
                EntityReference::Created(created) => {
                    created.client_key.owned_allocation_capacity_bytes()
                }
            })
            .sum::<u64>();
        if !context.claim_scratch(
            ((2 * std::mem::size_of::<EntityReference>()) + 3 * std::mem::size_of::<usize>())
                as u64
                + owned,
        ) {
            return successors;
        }
        successors
            .entry(edge.source.clone())
            .or_insert_with(Vec::new)
            .push(edge.target.clone());
    }
    successors
}

pub(super) fn planned_successor_count(planned_successors: &PlannedSuccessorMap) -> usize {
    planned_successors.values().map(Vec::len).sum()
}
