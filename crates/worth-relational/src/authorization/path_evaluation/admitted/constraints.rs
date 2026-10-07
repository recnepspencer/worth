use super::{admission, field, relations, try_retain, Context, ObservationResult, State, Witness};
use crate::authorization::{
    RelationalAuthorizationBudgetedObservationStop, RelationalAuthorizationObservationAdmission,
    RelationalAuthorizationPathPlan,
};
use crate::identity::data::EntityId;
use std::collections::BTreeSet;

pub(super) fn apply<A: RelationalAuthorizationObservationAdmission>(
    context: &Context<'_, '_, '_>,
    path: &RelationalAuthorizationPathPlan,
    ordinal: usize,
    frontier: &mut BTreeSet<Witness>,
    state: &mut State<'_, A>,
) -> ObservationResult<(), A> {
    let declarations = path
        .predicates()
        .len()
        .checked_add(path.field_constraints().len())
        .and_then(|n| n.checked_add(path.entity_anchors().len()))
        .and_then(|n| n.checked_add(path.related_entities().len()))
        .and_then(|n| n.checked_add(path.exact_adjacencies().len()))
        .ok_or(RelationalAuthorizationBudgetedObservationStop::AccountingOverflow)?;
    admission::prepare(state.admission, declarations as u64, 0)?;
    for predicate in path
        .predicates()
        .iter()
        .filter(|predicate| predicate.traversal_ordinal() == ordinal)
    {
        // Retain still visits/drops the rest of this temporary tree after a stop.
        // Admit that traversal before entering the fallible predicate callback.
        admission::prepare(state.admission, frontier.len() as u64, 0)?;
        try_retain::<A>(frontier, |witness| {
            let entity = witness.current();
            state.counters.predicate_fields_inspected += 1;
            state.counters.entity_records_inspected += 1;
            state.field(entity, predicate.field())?;
            let Some(value) = field::observe(
                context,
                entity,
                predicate.entity_kind(),
                predicate.field(),
                state,
            )?
            else {
                return Ok(false);
            };
            admission::prepare(
                state.admission,
                value.semantic_byte_width() as u64
                    + predicate.expected().semantic_byte_width() as u64,
                0,
            )?;
            Ok(predicate.matches(&value))
        })?;
    }
    for constraint in path.field_constraints().iter().filter(|constraint| {
        constraint
            .left()
            .traversal_ordinal()
            .max(constraint.right().traversal_ordinal())
            == ordinal
    }) {
        admission::prepare(state.admission, frontier.len() as u64, 0)?;
        try_retain::<A>(frontier, |witness| {
            let Some(left) = witness.entity_at(constraint.left().traversal_ordinal()) else {
                return Ok(false);
            };
            let Some(right) = witness.entity_at(constraint.right().traversal_ordinal()) else {
                return Ok(false);
            };
            state.counters.predicate_fields_inspected += 2;
            state.counters.entity_records_inspected += 2;
            state.field(left, constraint.left().field())?;
            state.field(right, constraint.right().field())?;
            let left = field::observe(
                context,
                left,
                constraint.left().entity_kind(),
                constraint.left().field(),
                state,
            )?;
            let right = field::observe(
                context,
                right,
                constraint.right().entity_kind(),
                constraint.right().field(),
                state,
            )?;
            match left.zip(right) {
                Some((left, right)) => {
                    admission::prepare(
                        state.admission,
                        left.semantic_byte_width() as u64 + right.semantic_byte_width() as u64,
                        0,
                    )?;
                    Ok(constraint.matches(&left, &right))
                }
                None => Ok(false),
            }
        })?;
    }
    for anchor in path
        .entity_anchors()
        .iter()
        .filter(|anchor| anchor.traversal_ordinal() == ordinal)
    {
        admission::prepare(state.admission, frontier.len() as u64, 0)?;
        frontier.retain(|witness| witness.current() == anchor.entity());
    }
    for constraint in path
        .related_entities()
        .iter()
        .filter(|constraint| constraint.traversal_ordinal() == ordinal)
    {
        admission::prepare(state.admission, frontier.len() as u64, 0)?;
        try_retain::<A>(frontier, |witness| {
            let source = witness.current();
            for id in relations::collect(context, source, constraint.traversal(), state)? {
                admission::prepare(state.admission, 1, 0)?;
                state.counters.relation_records_inspected += 1;
                let Some((candidate, kind)) =
                    relations::traverse(context, id, source, constraint.traversal(), state)?
                else {
                    continue;
                };
                if candidate != constraint.entity() {
                    continue;
                }
                state.entity(candidate)?;
                if field::live(context, candidate, kind, state)? {
                    return Ok(true);
                }
            }
            Ok(false)
        })?;
    }
    for constraint in path
        .exact_adjacencies()
        .iter()
        .filter(|constraint| constraint.traversal_ordinal() == ordinal)
    {
        admission::prepare(state.admission, frontier.len() as u64, 0)?;
        try_retain::<A>(frontier, |witness| {
            let source = witness.current();
            let ids = relations::collect(context, source, constraint.traversal(), state)?;
            admission::array::<A, EntityId>(state.admission, ids.len())?;
            let mut observed = Vec::with_capacity(ids.len());
            for id in ids {
                admission::prepare(state.admission, 1, 0)?;
                state.counters.relation_records_inspected += 1;
                let Some((candidate, kind)) =
                    relations::traverse(context, id, source, constraint.traversal(), state)?
                else {
                    continue;
                };
                state.entity(candidate)?;
                if field::live(context, candidate, kind, state)? {
                    observed.push(candidate);
                }
            }
            if !observed.is_empty() {
                let work = observed
                    .len()
                    .checked_mul(observed.len().ilog2() as usize + 1)
                    .and_then(|work| work.checked_mul(64))
                    .ok_or(RelationalAuthorizationBudgetedObservationStop::AccountingOverflow)?;
                admission::prepare(state.admission, work as u64, 0)?;
                observed.sort_unstable();
            }
            admission::prepare(
                state.admission,
                observed.len().min(constraint.expected_entities().len()) as u64 + 1,
                0,
            )?;
            Ok(observed == constraint.expected_entities())
        })?;
    }
    Ok(())
}
