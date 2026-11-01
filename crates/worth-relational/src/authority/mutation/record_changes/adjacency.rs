//! The same endpoint membership changes under ordinary or original-request control.
use super::*;
pub(crate) fn apply_adjacency_deltas(
    state: &mut WorkingState,
    deltas: &[AdjacencyDelta],
    version_id: crate::identity::data::VersionId,
) {
    for delta in deltas {
        let (source, target) = match delta.kind {
            AdjacencyDeltaKind::Created { source, target }
            | AdjacencyDeltaKind::Deleted { source, target } => (source, target),
        };
        state.mark_adjacency_slot_touched(source.partition_id, source.slot_index());
        state.mark_reverse_adjacency_slot_touched(target.partition_id, target.slot_index());

        let source_partition = state.get_partition_mut(source.partition_id);
        ensure_entity_adjacency_capacity(source_partition, source.slot_index());
        match delta.kind {
            AdjacencyDeltaKind::Created { .. } => {
                source_partition
                    .adjacency
                    .ensure(source.slot_index(), &source_partition.adjacency_policy)
                    .insert_at(delta.kind_id, delta.relation_id, version_id);
            }
            AdjacencyDeltaKind::Deleted { .. } => {
                if let Some(relations) = source_partition.adjacency.get_mut(source.slot_index()) {
                    relations.remove_at(delta.kind_id, &delta.relation_id, version_id);
                }
            }
        }

        let target_partition = state.get_partition_mut(target.partition_id);
        ensure_entity_adjacency_capacity(target_partition, target.slot_index());
        match delta.kind {
            AdjacencyDeltaKind::Created { .. } => {
                target_partition
                    .reverse_adjacency
                    .ensure(target.slot_index(), &target_partition.adjacency_policy)
                    .insert_at(delta.kind_id, delta.relation_id, version_id);
            }
            AdjacencyDeltaKind::Deleted { .. } => {
                if let Some(relations) = target_partition
                    .reverse_adjacency
                    .get_mut(target.slot_index())
                {
                    relations.remove_at(delta.kind_id, &delta.relation_id, version_id);
                }
            }
        }
    }
}

fn ensure_entity_adjacency_capacity(partition: &mut PartitionState, slot: usize) {
    let policy = partition.adjacency_policy.clone();
    partition.adjacency.ensure(slot, &policy);
    partition.reverse_adjacency.ensure(slot, &policy);
}

pub(crate) fn apply_adjacency_deltas_for_request(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    deltas: &[AdjacencyDelta],
    version_id: crate::identity::data::VersionId,
) -> Result<(), crate::transactions::data::TransactionCommitError> {
    for delta in deltas {
        let (source, target) = match delta.kind {
            AdjacencyDeltaKind::Created { source, target }
            | AdjacencyDeltaKind::Deleted { source, target } => (source, target),
        };
        state
            .mark_adjacency_slot_touched_for_request(
                source.partition_id,
                source.slot_index(),
                execution,
            )
            .map_err(super::execution_denial)?;
        state
            .mark_reverse_adjacency_slot_touched_for_request(
                target.partition_id,
                target.slot_index(),
                execution,
            )
            .map_err(super::execution_denial)?;

        let source_partition =
            super::ensure_partition_state(execution, state, source.partition_id)?;

        match delta.kind {
            AdjacencyDeltaKind::Created { .. } => {
                source_partition
                    .adjacency
                    .ensure_for_request(
                        source.slot_index(),
                        &source_partition.adjacency_policy,
                        execution,
                    )
                    .map_err(super::execution_denial)?
                    .insert_at_for_request(delta.kind_id, delta.relation_id, version_id, execution)
                    .map_err(super::execution_denial)?;
            }
            AdjacencyDeltaKind::Deleted { .. } => {
                if let Some(relations) = source_partition
                    .adjacency
                    .get_mut_for_request(source.slot_index(), execution)
                    .map_err(super::execution_denial)?
                {
                    relations
                        .remove_at_for_request(
                            delta.kind_id,
                            &delta.relation_id,
                            version_id,
                            execution,
                        )
                        .map_err(super::execution_denial)?;
                }
            }
        }

        let target_partition =
            super::ensure_partition_state(execution, state, target.partition_id)?;

        match delta.kind {
            AdjacencyDeltaKind::Created { .. } => {
                target_partition
                    .reverse_adjacency
                    .ensure_for_request(
                        target.slot_index(),
                        &target_partition.adjacency_policy,
                        execution,
                    )
                    .map_err(super::execution_denial)?
                    .insert_at_for_request(delta.kind_id, delta.relation_id, version_id, execution)
                    .map_err(super::execution_denial)?;
            }
            AdjacencyDeltaKind::Deleted { .. } => {
                if let Some(relations) = target_partition
                    .reverse_adjacency
                    .get_mut_for_request(target.slot_index(), execution)
                    .map_err(super::execution_denial)?
                {
                    relations
                        .remove_at_for_request(
                            delta.kind_id,
                            &delta.relation_id,
                            version_id,
                            execution,
                        )
                        .map_err(super::execution_denial)?;
                }
            }
        }
    }
    Ok(())
}
