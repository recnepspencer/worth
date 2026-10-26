//! One exhausted FIFO, with canonical output independent of seed order.
use std::collections::{BTreeMap, BTreeSet};

use worth_execution::{ExecutionRequest, MapKernelContext};
use worth_foundational::facade::RelationDirection;

use super::super::breadth_first_frontier::BreadthFirstFrontier;
use super::{
    memory, PinnedStructuralWalk, PinnedStructuralWalkDenialKind as Denial,
    PinnedStructuralWalkReads, PinnedStructuralWalkScope, PinnedStructuralWalkVisit,
};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::identity::data::{EntityId, RelationId};
use crate::storage::data::RecordLifecycleState;
use crate::visibility::materialization::read_records::projection::{
    RelationalAdjacencyDirection, RelationalAdjacencyVisit, RelationalEntityMetadata,
    RelationalRelationMetadata, VisibilityProjectionView,
};

pub(super) fn execute(
    projection: &VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    scope: &PinnedStructuralWalkScope<'_>,
    context: &mut MapKernelContext<'_, '_>,
    reads: &mut PinnedStructuralWalkReads,
    work: &mut usize,
    mut memory: memory::WalkMemory,
) -> Result<PinnedStructuralWalk, Denial> {
    let mut frontier = BreadthFirstFrontier::new();
    let mut visits = BTreeMap::new();
    let mut relations = BTreeMap::new();
    let mut roots = BTreeSet::new();
    for root in scope.roots {
        checkpoint(context, scope, work, 1)?;
        if !roots.contains(root) {
            memory.claim_scratch(memory::tree_entry::<EntityId, ()>()?)?;
            roots.insert(*root);
        }
    }
    for root in roots {
        checkpoint(context, scope, work, 1)?;
        discover(
            projection,
            scope,
            root,
            0,
            root,
            &mut frontier,
            context,
            reads,
            work,
            &mut memory,
        )?;
    }
    while let Some((entity, distance, root)) = frontier.pop() {
        checkpoint(context, scope, work, 1)?;
        memory.claim_scratch(memory::tree_entry::<
            (usize, EntityId),
            PinnedStructuralWalkVisit,
        >()?)?;
        visits.insert(
            (distance, entity.entity_id),
            PinnedStructuralWalkVisit {
                entity,
                distance,
                root,
            },
        );
        let direction = match scope.direction {
            RelationDirection::Forward => RelationalAdjacencyDirection::Outgoing,
            RelationDirection::Reverse => RelationalAdjacencyDirection::Incoming,
        };
        projection.try_visit_adjacency_ids(
            execution,
            entity.entity_id,
            scope.relation_kind,
            direction,
            |event| {
                match event {
                    RelationalAdjacencyVisit::Prepare { work: units, bytes } => {
                        checkpoint(context, scope, work, units)?;
                        memory.temporary(bytes)?;
                    }
                    RelationalAdjacencyVisit::List => {
                        checkpoint(context, scope, work, 1)?;
                    }
                    RelationalAdjacencyVisit::Acquired => {
                        increment(&mut reads.adjacency_lists)?;
                        #[cfg(test)]
                        observe(
                            scope,
                            super::PinnedWalkEventForTest::Adjacency(entity.entity_id),
                        );
                    }
                    RelationalAdjacencyVisit::Relation(id) => {
                        checkpoint(
                            context,
                            scope,
                            work,
                            projection
                                .exact_relation_metadata_read_work_bound()
                                .ok_or(Denial::EndpointUnavailable)?,
                        )?;
                        let record = projection
                            .exact_relation_metadata(id)
                            .map_err(|_| Denial::EndpointUnavailable)?;
                        if record.is_some() {
                            increment(&mut reads.relation_records)?;
                            #[cfg(test)]
                            observe(scope, super::PinnedWalkEventForTest::Relation(id));
                        }
                        checkpoint(context, scope, work, 0)?;
                        let Some(record) = record else {
                            return Ok(());
                        };
                        if record.lifecycle != RecordLifecycleState::Live
                            || record.kind_id != scope.relation_kind
                        {
                            return Ok(());
                        }
                        let (anchor, neighbor) = match scope.direction {
                            RelationDirection::Forward => (record.source, record.target),
                            RelationDirection::Reverse => (record.target, record.source),
                        };
                        if anchor != entity.entity_id {
                            return Err(Denial::EndpointUnavailable);
                        }
                        // Emit physical self/back/diamond edges even if discovery deduplicates.
                        if !relations.contains_key(&record.relation_id) {
                            memory.claim_scratch(memory::tree_entry::<
                                RelationId,
                                RelationalRelationMetadata,
                            >()?)?;
                            relations.insert(record.relation_id, record);
                        }
                        if !frontier.contains(&neighbor) {
                            let next = distance.checked_add(1).ok_or(Denial::DistanceOverflow)?;
                            discover(
                                projection,
                                scope,
                                neighbor,
                                next,
                                root,
                                &mut frontier,
                                context,
                                reads,
                                work,
                                &mut memory,
                            )?;
                        }
                    }
                }
                Ok::<_, Denial>(())
            },
        )??;
        memory.release_temporary()?;
    }
    let bytes = memory::vector_bytes::<PinnedStructuralWalkVisit>(visits.len())?
        .checked_add(memory::vector_bytes::<RelationalRelationMetadata>(
            relations.len(),
        )?)
        .ok_or(Cause::ChargedBytesOverflow)?;
    memory.result(bytes)?;
    let mut ordered_visits = Vec::new();
    let mut ordered_relations = Vec::new();
    ordered_visits
        .try_reserve_exact(visits.len())
        .map_err(|_| Cause::AllocationFailed)?;
    ordered_relations
        .try_reserve_exact(relations.len())
        .map_err(|_| Cause::AllocationFailed)?;
    for visit in visits.into_values() {
        checkpoint(context, scope, work, 1)?;
        ordered_visits.push(visit);
    }
    for relation in relations.into_values() {
        checkpoint(context, scope, work, 1)?;
        ordered_relations.push(relation);
    }
    Ok(PinnedStructuralWalk {
        visits: ordered_visits,
        relations: ordered_relations,
        reads: *reads,
        work: *work,
        _retention: memory.take_result(),
    })
}

fn discover(
    projection: &VisibilityProjectionView,
    scope: &PinnedStructuralWalkScope<'_>,
    id: EntityId,
    distance: usize,
    root: EntityId,
    frontier: &mut BreadthFirstFrontier<(RelationalEntityMetadata, usize, EntityId)>,
    context: &mut MapKernelContext<'_, '_>,
    reads: &mut PinnedStructuralWalkReads,
    work: &mut usize,
    memory: &mut memory::WalkMemory,
) -> Result<(), Denial> {
    if frontier.contains(&id) {
        return Ok(());
    }
    checkpoint(
        context,
        scope,
        work,
        projection
            .exact_entity_state_read_work_bound()
            .ok_or(Denial::EndpointUnavailable)?,
    )?;
    let entity = projection
        .with_exact_entity_state(id, |metadata, _| metadata)
        .map_err(|_| Denial::EndpointUnavailable)?;
    if entity.is_some() {
        increment(&mut reads.entity_records)?;
        #[cfg(test)]
        observe(scope, super::PinnedWalkEventForTest::Entity(id));
    }
    checkpoint(context, scope, work, 0)?;
    let entity = entity.ok_or(Denial::EndpointUnavailable)?;
    if entity.kind_id != scope.entity_kind {
        return Err(Denial::WrongEntityKind);
    }
    if entity.lifecycle != RecordLifecycleState::Live {
        return Err(Denial::EndpointUnavailable);
    }
    frontier.enqueue(entity.entity_id, (entity, distance, root), || {
        let queue = memory::vector_bytes::<(RelationalEntityMetadata, usize, EntityId)>(8)?;
        memory.claim_scratch(
            memory::tree_entry::<EntityId, ()>()?
                .checked_add(queue)
                .ok_or(Cause::ChargedBytesOverflow)?,
        )
    })?;
    Ok(())
}

pub(super) fn checkpoint(
    context: &mut MapKernelContext<'_, '_>,
    scope: &PinnedStructuralWalkScope<'_>,
    work: &mut usize,
    units: u64,
) -> Result<(), Denial> {
    let admitted = if context.has_accounting() {
        context.checkpoint(units)
    } else {
        context.poll()
    };
    admitted.map_err(crate::execution::denial::kernel_stop)?;
    if let Some(control) = scope.control {
        control()?;
    }
    let units = usize::try_from(units).map_err(|_| Cause::WorkCounterOverflow)?;
    let next = work.checked_add(units).ok_or(Cause::WorkCounterOverflow)?;
    if scope.maximum_work.is_some_and(|limit| next > limit) {
        return Err(Denial::WorkLimitExceeded);
    }
    *work = next;
    Ok(())
}

fn increment(counter: &mut usize) -> Result<(), Denial> {
    *counter = counter.checked_add(1).ok_or(Cause::WorkCounterOverflow)?;
    Ok(())
}

#[cfg(test)]
fn observe(scope: &PinnedStructuralWalkScope<'_>, event: super::PinnedWalkEventForTest) {
    if let Some(observer) = scope.observer {
        observer(event);
    }
}
