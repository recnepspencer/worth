use super::{admission, Context, ObservationResult, State};
use crate::authorization::{
    RelationalAuthorizationAdjacencyDependency, RelationalAuthorizationBudgetedObservationStop,
    RelationalAuthorizationObservationAdmission, RelationalAuthorizationTraversal,
    RelationalAuthorizationTraversalDirection,
};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::storage::data::RecordLifecycleState;
use crate::visibility::materialization::read_records::{
    RelationalAdjacencyDirection, RelationalAdjacencyVisit,
};

type OrderedRelation = ((u32, u64, u32, u64, u32, u64), RelationId);

pub(super) fn collect<A: RelationalAuthorizationObservationAdmission>(
    context: &Context<'_, '_, '_>,
    entity: EntityId,
    traversal: &RelationalAuthorizationTraversal,
    state: &mut State<'_, A>,
) -> ObservationResult<Vec<RelationId>, A> {
    state.adjacency(RelationalAuthorizationAdjacencyDependency::new(
        entity,
        traversal.relation_kind(),
        traversal.direction(),
    ))?;
    let mut ordered = Vec::<OrderedRelation>::new();
    let direction = match traversal.direction() {
        RelationalAuthorizationTraversalDirection::Forward => {
            RelationalAdjacencyDirection::Outgoing
        }
        RelationalAuthorizationTraversalDirection::Reverse => {
            RelationalAdjacencyDirection::Incoming
        }
    };
    context.view.try_visit_adjacency_ids(
        entity,
        traversal.relation_kind(),
        direction,
        |visit| {
            if let RelationalAdjacencyVisit::Prepare { work, bytes } = visit {
                return admission::prepare(state.admission, work, bytes);
            }
            admission::prepare(state.admission, 1, 0)?;
            let id = match visit {
                RelationalAdjacencyVisit::Prepare { .. } => {
                    unreachable!("preparation is handled above")
                }
                RelationalAdjacencyVisit::List => {
                    state.counters.adjacency_lists_read += 1;
                    return Ok(());
                }
                RelationalAdjacencyVisit::Relation(id) => id,
            };
            state.counters.adjacency_edges_inspected += 1;
            // Exact metadata borrows the selected native slot; no read record is built.
            let read_work = context
                .view
                .exact_relation_metadata_read_work_bound()
                .ok_or(RelationalAuthorizationBudgetedObservationStop::ExactBasisRequired)?;
            admission::prepare(state.admission, read_work, 0)?;
            let record = context
                .view
                .exact_relation_metadata(id)
                .map_err(|_| RelationalAuthorizationBudgetedObservationStop::ExactBasisRequired)?;
            let descriptor = record.and_then(|record| {
                if record.lifecycle != RecordLifecycleState::Live
                    || record.kind_id != traversal.relation_kind()
                    || match direction {
                        RelationalAdjacencyDirection::Outgoing => record.source != entity,
                        RelationalAdjacencyDirection::Incoming => record.target != entity,
                    }
                {
                    return None;
                }
                Some((
                    (
                        record.source.partition_value(),
                        record.source.local_slot_value(),
                        record.target.partition_value(),
                        record.target.local_slot_value(),
                        record.relation_id.partition_value(),
                        record.relation_id.local_slot_value(),
                    ),
                    record.relation_id,
                ))
            });
            if let Some(descriptor) = descriptor {
                if ordered.len() == ordered.capacity() {
                    let capacity = ordered
                        .capacity()
                        .checked_mul(2)
                        .map(|capacity| capacity.max(4))
                        .ok_or(
                            RelationalAuthorizationBudgetedObservationStop::AccountingOverflow,
                        )?;
                    admission::array::<A, OrderedRelation>(state.admission, capacity)?;
                    ordered.reserve_exact(capacity - ordered.len());
                }
                admission::prepare(state.admission, 1, 0)?;
                ordered.push(descriptor);
            }
            Ok(())
        },
    )?;
    if !ordered.is_empty() {
        let comparisons = ordered
            .len()
            .checked_mul(ordered.len().ilog2() as usize + 1)
            .and_then(|work| work.checked_mul(64))
            .ok_or(RelationalAuthorizationBudgetedObservationStop::AccountingOverflow)?;
        admission::prepare(state.admission, comparisons as u64, 0)?;
        ordered.sort_unstable_by_key(|entry| entry.0);
    }
    admission::array::<A, RelationId>(state.admission, ordered.len())?;
    let mut ids = Vec::with_capacity(ordered.len());
    ids.extend(ordered.into_iter().map(|(_, id)| id));
    Ok(ids)
}

pub(super) fn traverse<A: RelationalAuthorizationObservationAdmission>(
    context: &Context<'_, '_, '_>,
    id: RelationId,
    current: EntityId,
    traversal: &RelationalAuthorizationTraversal,
    state: &mut State<'_, A>,
) -> ObservationResult<Option<(EntityId, KindId)>, A> {
    let read_work = context
        .view
        .exact_relation_metadata_read_work_bound()
        .ok_or(RelationalAuthorizationBudgetedObservationStop::ExactBasisRequired)?;
    admission::prepare(state.admission, read_work, 0)?;
    let record = context
        .view
        .exact_relation_metadata(id)
        .map_err(|_| RelationalAuthorizationBudgetedObservationStop::ExactBasisRequired)?;
    if let Some(record) = record {
        let candidate = if record.lifecycle != RecordLifecycleState::Live
            || record.kind_id != traversal.relation_kind()
        {
            None
        } else {
            match traversal.direction() {
                RelationalAuthorizationTraversalDirection::Forward if record.source == current => {
                    Some((record.target, traversal.to_kind()))
                }
                RelationalAuthorizationTraversalDirection::Reverse if record.target == current => {
                    Some((record.source, traversal.from_kind()))
                }
                _ => None,
            }
        };
        state.relation(record.relation_id)?;
        Ok(candidate)
    } else {
        Ok(None)
    }
}
