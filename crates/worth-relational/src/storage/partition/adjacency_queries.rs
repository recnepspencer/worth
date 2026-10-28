use std::collections::BTreeSet;

use crate::identity::data::{EntityId, RelationId, VersionId};
use crate::runtime::RelationalRuntime;
use crate::storage::overlay::PartitionAccess;

use super::AdjacencyDirection;

/// Ordered borrowed candidates for a leased traversal. The caller checkpoints
/// each edge before materialization, so fanout never becomes a temporary Vec.
pub(crate) fn relation_candidates_from_state(
    state: &dyn PartitionAccess,
    entity_id: EntityId,
    direction: AdjacencyDirection,
) -> super::adjacency::AdjacencyIds<'_> {
    state
        .get_partition(entity_id.partition_id)
        .and_then(|partition| direction.table(partition).get(entity_id.slot_index()))
        .map(|relations| relations.current_ids())
        .unwrap_or_default()
}

pub(crate) fn outgoing_relations_for_entity(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity_id: EntityId,
    version_id: VersionId,
) -> Result<Vec<RelationId>, crate::execution::RelationalExecutionDenialCause> {
    whole_neighborhood(
        runtime,
        execution,
        entity_id,
        version_id,
        AdjacencyDirection::Outgoing,
    )
}

pub(crate) fn incoming_relations_for_entity(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity_id: EntityId,
    version_id: VersionId,
) -> Result<Vec<RelationId>, crate::execution::RelationalExecutionDenialCause> {
    whole_neighborhood(
        runtime,
        execution,
        entity_id,
        version_id,
        AdjacencyDirection::Incoming,
    )
}

pub(crate) fn all_relations_for_entity(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity_id: EntityId,
    version_id: VersionId,
) -> Result<Vec<RelationId>, crate::execution::RelationalExecutionDenialCause> {
    let edition = runtime.acquire_partition_edition();
    crate::visibility::materialization::read_records::read_work::with_full_residency(
        execution,
        &edition,
        &runtime.services.symbols,
        |state| {
            Ok(all_relations_for_entity_from_state(
                runtime, state, entity_id, version_id,
            ))
        },
    )
}

pub(crate) fn all_relations_for_entity_from_state(
    runtime: &RelationalRuntime,
    state: &dyn PartitionAccess,
    entity_id: EntityId,
    version_id: VersionId,
) -> Vec<RelationId> {
    let slot = entity_id.slot_index();
    let mut candidates = BTreeSet::new();
    if let Some(partition) = state.get_partition(entity_id.partition_id) {
        for direction in [AdjacencyDirection::Outgoing, AdjacencyDirection::Incoming] {
            if let Some(relations) = direction.table(partition).get(slot) {
                relations.extend_into(&mut candidates);
            }
        }
    }
    charge_copied_ids(runtime, candidates.len());
    retain_visible(runtime, state, candidates, version_id)
}

/// The whole unfiltered neighborhood, which is an owned answer by contract.
///
/// This lane genuinely copies Theta(degree) relation ids, because the caller
/// asked for every one of them. That copy is charged, which is what makes the
/// bounded readers' zero interpretable: they lease the same slice instead.
fn whole_neighborhood(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity_id: EntityId,
    version_id: VersionId,
    direction: AdjacencyDirection,
) -> Result<Vec<RelationId>, crate::execution::RelationalExecutionDenialCause> {
    let edition = runtime.acquire_partition_edition();
    crate::visibility::materialization::read_records::read_work::with_full_residency(
        execution,
        &edition,
        &runtime.services.symbols,
        |state| {
            let candidates = state
                .get_partition(entity_id.partition_id)
                .and_then(|partition| direction.table(partition).get(entity_id.slot_index()))
                .map(|relations| relations.current_ids().to_vec())
                .unwrap_or_default();
            charge_copied_ids(runtime, candidates.len());
            Ok(retain_visible(runtime, state, candidates, version_id))
        },
    )
}

/// Resolve every candidate against the one edition already pinned.
///
/// Re-acquiring per candidate would make an answer of size R cost R substrate
/// lookups against a moving substrate, and would also let the neighborhood be
/// filtered against editions that never coexisted.
fn retain_visible(
    runtime: &RelationalRuntime,
    edition: &dyn PartitionAccess,
    candidates: impl IntoIterator<Item = RelationId>,
    version_id: VersionId,
) -> Vec<RelationId> {
    let reader = runtime.read_truth();
    candidates
        .into_iter()
        .filter(|relation_id| reader.relation_visible_in_edition(edition, *relation_id, version_id))
        .collect()
}

fn charge_copied_ids(runtime: &RelationalRuntime, copied: usize) {
    if copied == 0 {
        return;
    }
    runtime
        .services
        .instrumentation
        .count(|counters| counters.adjacency_relation_ids_copied += copied);
}
