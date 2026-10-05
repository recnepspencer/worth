use crate::authorization::admission::{self, ObservationResult};
use crate::authorization::{
    RelationalAuthorizationBudgetedObservationStop, RelationalAuthorizationObservationAdmission,
    RelationalAuthorizationObservationCounters, RelationalAuthorizationObservationPlan,
    RelationalAuthorizationPathObservation, RelationalAuthorizationPathPlan,
    RelationalAuthorizationPathWitness, RelationalAuthorizationTraversal,
    RelationalAuthorizationTraversalDirection,
};
use crate::identity::data::EntityId;
use crate::runtime::RelationalRuntime;
use crate::visibility::materialization::read_records::VisibilityProjectionView;
use std::collections::BTreeSet;

mod constraints;
mod field;
mod relations;
mod state;
use state::{Context, State, Witness};

pub(in crate::authorization) fn evaluate_path<A: RelationalAuthorizationObservationAdmission>(
    runtime: &RelationalRuntime,
    view: &VisibilityProjectionView<'_>,
    plan: &RelationalAuthorizationObservationPlan,
    path: &RelationalAuthorizationPathPlan,
    counters: &mut RelationalAuthorizationObservationCounters,
    admission: &mut A,
) -> ObservationResult<RelationalAuthorizationPathObservation, A> {
    let context = Context { runtime, view };
    let mut state = State::new(plan.principal(), counters, admission)?;
    admission::ordered_insert::<A, Witness>(
        state.admission,
        0,
        1,
        std::mem::size_of::<EntityId>(),
    )?;
    let mut frontier = BTreeSet::from([Witness {
        entities: vec![plan.principal()],
    }]);
    state.counters.paths_evaluated += 1;
    state.counters.maximum_frontier_width =
        state.counters.maximum_frontier_width.max(frontier.len());
    constraints::apply(&context, path, 0, &mut frontier, &mut state)?;
    for (index, traversal) in path.traversals().iter().enumerate() {
        admission::prepare(state.admission, 1 + path.entity_anchors().len() as u64, 0)?;
        let anchor = super::unique_anchor_at(path, index + 1);
        frontier = traverse(&context, traversal, anchor, &frontier, &mut state)?;
        constraints::apply(&context, path, index + 1, &mut frontier, &mut state)?;
        state.counters.maximum_frontier_width =
            state.counters.maximum_frontier_width.max(frontier.len());
        if frontier.is_empty() {
            break;
        }
    }
    let mut witness = None;
    for candidate in &frontier {
        admission::prepare(state.admission, 1, 0)?;
        if candidate.current() == plan.scope() {
            admission::array::<A, EntityId>(state.admission, candidate.entities.len())?;
            witness = Some(RelationalAuthorizationPathWitness::new(
                candidate.entities.clone(),
            ));
            break;
        }
    }
    let dependencies = state.finish()?;
    Ok(RelationalAuthorizationPathObservation::new(
        witness.is_some(),
        witness,
        dependencies,
        true,
    ))
}

fn traverse<A: RelationalAuthorizationObservationAdmission>(
    context: &Context<'_, '_, '_>,
    traversal: &RelationalAuthorizationTraversal,
    anchor: Option<EntityId>,
    frontier: &BTreeSet<Witness>,
    state: &mut State<'_, A>,
) -> ObservationResult<BTreeSet<Witness>, A> {
    let mut next = BTreeSet::new();
    for witness in frontier {
        admission::prepare(state.admission, 1, 0)?;
        let current = witness.current();
        let ids = if let Some(anchor) = anchor {
            let inverse = RelationalAuthorizationTraversal::new(
                traversal.relation_kind(),
                traversal.from_kind(),
                traversal.to_kind(),
                match traversal.direction() {
                    RelationalAuthorizationTraversalDirection::Forward => {
                        RelationalAuthorizationTraversalDirection::Reverse
                    }
                    RelationalAuthorizationTraversalDirection::Reverse => {
                        RelationalAuthorizationTraversalDirection::Forward
                    }
                },
            );
            relations::collect(context, anchor, &inverse, state)?
        } else {
            relations::collect(context, current, traversal, state)?
        };
        for id in ids {
            admission::prepare(state.admission, 1, 0)?;
            state.counters.relation_records_inspected += 1;
            let Some(candidate) = relations::traverse(context, id, current, traversal, state)?
            else {
                continue;
            };
            if anchor.is_some_and(|anchor| candidate.0 != anchor) {
                continue;
            }
            state.entity(candidate.0)?;
            if field::live(context, candidate.0, candidate.1, state)? {
                let count =
                    witness.entities.len().checked_add(1).ok_or(
                        RelationalAuthorizationBudgetedObservationStop::AccountingOverflow,
                    )?;
                admission::ordered_insert::<A, Witness>(state.admission, next.len(), count, 0)?;
                admission::array::<A, EntityId>(state.admission, count)?;
                let mut entities = Vec::with_capacity(count);
                entities.extend_from_slice(&witness.entities);
                entities.push(candidate.0);
                next.insert(Witness { entities });
            }
        }
    }
    Ok(next)
}

fn try_retain<A: RelationalAuthorizationObservationAdmission>(
    frontier: &mut BTreeSet<Witness>,
    mut predicate: impl FnMut(&Witness) -> ObservationResult<bool, A>,
) -> ObservationResult<(), A> {
    let mut stop = None;
    frontier.retain(|witness| {
        if stop.is_some() {
            return false;
        }
        match predicate(witness) {
            Ok(keep) => keep,
            Err(reason) => {
                stop = Some(reason);
                false
            }
        }
    });
    match stop {
        Some(stop) => Err(stop),
        None => Ok(()),
    }
}
