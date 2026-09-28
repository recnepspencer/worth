use std::collections::BTreeSet;

use crate::identity::data::{EntityId, RelationId};
use crate::runtime::RelationalRuntime;
use crate::storage::data::RecordLifecycleState;
use crate::visibility::materialization::read_records::{
    ProjectionAspectScope, VisibilityProjectionView,
};

use super::dependency_collection::RelationalAuthorizationDependencySets;
use super::field_observation::{entity_is_live_kind, observed_field};
use super::{
    RelationalAuthorizationAdjacencyDependency, RelationalAuthorizationObservationCounters,
    RelationalAuthorizationObservationPlan, RelationalAuthorizationPathObservation,
    RelationalAuthorizationPathPlan, RelationalAuthorizationPathWitness,
    RelationalAuthorizationTraversal, RelationalAuthorizationTraversalDirection,
};

mod anchored_traversal;
mod exact_adjacency;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct RelationalAuthorizationWitness {
    entities: Vec<EntityId>,
}

impl RelationalAuthorizationWitness {
    fn root(principal: EntityId) -> Self {
        Self {
            entities: vec![principal],
        }
    }

    fn current(&self) -> EntityId {
        *self
            .entities
            .last()
            .expect("an authorization witness always retains its principal")
    }

    fn entity_at(&self, ordinal: usize) -> Option<EntityId> {
        self.entities.get(ordinal).copied()
    }

    fn extend(&self, entity: EntityId) -> Self {
        let mut entities = self.entities.clone();
        entities.push(entity);
        Self { entities }
    }
}

struct PathReadContext<'runtime, 'projection, 'snapshot> {
    runtime: &'runtime RelationalRuntime,
    view: &'projection VisibilityProjectionView<'snapshot>,
}

struct PathEvaluationState<'counters> {
    dependencies: RelationalAuthorizationDependencySets,
    counters: &'counters mut RelationalAuthorizationObservationCounters,
}

pub(super) fn evaluate_path(
    runtime: &RelationalRuntime,
    view: &VisibilityProjectionView<'_>,
    plan: &RelationalAuthorizationObservationPlan,
    path: &RelationalAuthorizationPathPlan,
    counters: &mut RelationalAuthorizationObservationCounters,
) -> RelationalAuthorizationPathObservation {
    let context = PathReadContext { runtime, view };
    let mut state = PathEvaluationState {
        dependencies: RelationalAuthorizationDependencySets::new(plan.principal()),
        counters,
    };
    state.counters.paths_evaluated += 1;
    let root = RelationalAuthorizationWitness::root(plan.principal());
    let mut frontier = BTreeSet::from([root]);
    state.counters.maximum_frontier_width =
        state.counters.maximum_frontier_width.max(frontier.len());
    apply_constraints(&context, path, 0, &mut frontier, &mut state);
    for (index, traversal) in path.traversals().iter().enumerate() {
        frontier = traverse_frontier(
            &context,
            traversal,
            anchored_traversal::unique_anchor_at(path, index + 1),
            &frontier,
            &mut state,
        );
        apply_constraints(&context, path, index + 1, &mut frontier, &mut state);
        state.counters.maximum_frontier_width =
            state.counters.maximum_frontier_width.max(frontier.len());
        if frontier.is_empty() {
            break;
        }
    }
    let witness = frontier
        .iter()
        .find(|witness| witness.current() == plan.scope())
        .map(|witness| RelationalAuthorizationPathWitness::new(witness.entities.clone()));
    RelationalAuthorizationPathObservation::new(
        witness.is_some(),
        witness,
        state.dependencies.finish(),
        true,
    )
}

fn traverse_frontier(
    context: &PathReadContext<'_, '_, '_>,
    traversal: &RelationalAuthorizationTraversal,
    next_anchor: Option<EntityId>,
    frontier: &BTreeSet<RelationalAuthorizationWitness>,
    state: &mut PathEvaluationState<'_>,
) -> BTreeSet<RelationalAuthorizationWitness> {
    let mut next = BTreeSet::new();
    for witness in frontier {
        let entity = witness.current();
        let relation_ids = match next_anchor {
            Some(anchor) => anchored_traversal::relation_ids_for_anchored_step(
                context, anchor, traversal, state,
            ),
            None => relation_ids_for_step(context, entity, traversal, state),
        };
        for relation_id in relation_ids {
            state.counters.relation_records_inspected += 1;
            let Some(candidate) = traverse_relation(
                context.view,
                relation_id,
                entity,
                traversal,
                &mut state.dependencies.relations,
            ) else {
                continue;
            };
            if next_anchor.is_some_and(|anchor| candidate.0 != anchor) {
                continue;
            }
            state.dependencies.entities.insert(candidate.0);
            if entity_is_live_kind(context.view, candidate.0, candidate.1, state.counters) {
                next.insert(witness.extend(candidate.0));
            }
        }
    }
    next
}

fn apply_constraints(
    context: &PathReadContext<'_, '_, '_>,
    path: &RelationalAuthorizationPathPlan,
    ordinal: usize,
    frontier: &mut BTreeSet<RelationalAuthorizationWitness>,
    state: &mut PathEvaluationState<'_>,
) {
    apply_predicates(context, path, ordinal, frontier, state);
    apply_field_constraints(context, path, ordinal, frontier, state);
    apply_entity_anchors(path, ordinal, frontier);
    apply_related_entities(context, path, ordinal, frontier, state);
    exact_adjacency::apply(context, path, ordinal, frontier, state);
}

fn apply_field_constraints(
    context: &PathReadContext<'_, '_, '_>,
    path: &RelationalAuthorizationPathPlan,
    ordinal: usize,
    frontier: &mut BTreeSet<RelationalAuthorizationWitness>,
    state: &mut PathEvaluationState<'_>,
) {
    for constraint in path.field_constraints().iter().filter(|constraint| {
        constraint
            .left()
            .traversal_ordinal()
            .max(constraint.right().traversal_ordinal())
            == ordinal
    }) {
        frontier.retain(|witness| {
            let Some(left_entity) = witness.entity_at(constraint.left().traversal_ordinal()) else {
                return false;
            };
            let Some(right_entity) = witness.entity_at(constraint.right().traversal_ordinal())
            else {
                return false;
            };
            state.counters.predicate_fields_inspected += 2;
            state.counters.entity_records_inspected += 2;
            state
                .dependencies
                .fields
                .insert((left_entity, constraint.left().field().clone()));
            state
                .dependencies
                .fields
                .insert((right_entity, constraint.right().field().clone()));
            let left = observed_field(
                context.runtime,
                context.view,
                left_entity,
                constraint.left().entity_kind(),
                constraint.left().field(),
            );
            let right = observed_field(
                context.runtime,
                context.view,
                right_entity,
                constraint.right().entity_kind(),
                constraint.right().field(),
            );
            left.zip(right)
                .is_some_and(|(left, right)| constraint.matches(&left, &right))
        });
    }
}

fn apply_predicates(
    context: &PathReadContext<'_, '_, '_>,
    path: &RelationalAuthorizationPathPlan,
    ordinal: usize,
    frontier: &mut BTreeSet<RelationalAuthorizationWitness>,
    state: &mut PathEvaluationState<'_>,
) {
    for predicate in path
        .predicates()
        .iter()
        .filter(|predicate| predicate.traversal_ordinal() == ordinal)
    {
        frontier.retain(|witness| {
            let entity = witness.current();
            state.counters.predicate_fields_inspected += 1;
            state.counters.entity_records_inspected += 1;
            state
                .dependencies
                .fields
                .insert((entity, predicate.field().clone()));
            observed_field(
                context.runtime,
                context.view,
                entity,
                predicate.entity_kind(),
                predicate.field(),
            )
            .is_some_and(|value| predicate.matches(&value))
        });
    }
}

fn apply_entity_anchors(
    path: &RelationalAuthorizationPathPlan,
    ordinal: usize,
    frontier: &mut BTreeSet<RelationalAuthorizationWitness>,
) {
    for anchor in path
        .entity_anchors()
        .iter()
        .filter(|anchor| anchor.traversal_ordinal() == ordinal)
    {
        frontier.retain(|witness| witness.current() == anchor.entity());
    }
}

fn apply_related_entities(
    context: &PathReadContext<'_, '_, '_>,
    path: &RelationalAuthorizationPathPlan,
    ordinal: usize,
    frontier: &mut BTreeSet<RelationalAuthorizationWitness>,
    state: &mut PathEvaluationState<'_>,
) {
    for constraint in path
        .related_entities()
        .iter()
        .filter(|constraint| constraint.traversal_ordinal() == ordinal)
    {
        frontier.retain(|witness| {
            let source = witness.current();
            relation_ids_for_step(context, source, constraint.traversal(), state)
                .into_iter()
                .any(|relation_id| {
                    state.counters.relation_records_inspected += 1;
                    let Some((candidate, kind)) = traverse_relation(
                        context.view,
                        relation_id,
                        source,
                        constraint.traversal(),
                        &mut state.dependencies.relations,
                    ) else {
                        return false;
                    };
                    if candidate != constraint.entity() {
                        return false;
                    }
                    state.dependencies.entities.insert(candidate);
                    if !entity_is_live_kind(context.view, candidate, kind, state.counters) {
                        return false;
                    }
                    true
                })
        });
    }
}

fn relation_ids_for_step(
    context: &PathReadContext<'_, '_, '_>,
    entity: EntityId,
    traversal: &RelationalAuthorizationTraversal,
    state: &mut PathEvaluationState<'_>,
) -> Vec<RelationId> {
    state
        .dependencies
        .adjacencies
        .insert(RelationalAuthorizationAdjacencyDependency::new(
            entity,
            traversal.relation_kind(),
            traversal.direction(),
        ));
    // Evaluation always reads an exact basis, which carries its own branch
    // root and adjacency index. The runtime's current edition belongs to
    // whichever branch published last, so it cannot answer for this root.
    let entities = BTreeSet::from([entity]);
    let read = match traversal.direction() {
        RelationalAuthorizationTraversalDirection::Forward => {
            context.view.bounded_outgoing_relations_for_frontier(
                &entities,
                traversal.relation_kind(),
                usize::MAX,
            )
        }
        RelationalAuthorizationTraversalDirection::Reverse => {
            context.view.bounded_incoming_relations_for_frontier(
                &entities,
                traversal.relation_kind(),
                usize::MAX,
            )
        }
    }
    .expect("an unbounded adjacency read never exceeds its limit");
    state.counters.adjacency_lists_read += read.adjacency_lists_read;
    state.counters.adjacency_edges_inspected += read.relation_records_examined;
    read.records
        .into_iter()
        .map(|record| record.relation_id)
        .collect()
}

fn traverse_relation(
    view: &VisibilityProjectionView<'_>,
    relation_id: RelationId,
    current: EntityId,
    traversal: &RelationalAuthorizationTraversal,
    touched: &mut BTreeSet<RelationId>,
) -> Option<(EntityId, crate::identity::data::KindId)> {
    view.relation_record_with_projection_scope(
        relation_id,
        ProjectionAspectScope::empty(),
        |record| {
            touched.insert(record.relation_id());
            if record.lifecycle() != RecordLifecycleState::Live
                || record.kind_id() != traversal.relation_kind()
            {
                return None;
            }
            match traversal.direction() {
                RelationalAuthorizationTraversalDirection::Forward
                    if record.source() == current =>
                {
                    Some((record.target(), traversal.to_kind()))
                }
                RelationalAuthorizationTraversalDirection::Reverse
                    if record.target() == current =>
                {
                    Some((record.source(), traversal.from_kind()))
                }
                _ => None,
            }
        },
    )
}
