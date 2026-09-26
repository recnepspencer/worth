use std::collections::BTreeSet;

use super::{
    entity_is_live_kind, relation_ids_for_step, traverse_relation, PathEvaluationState,
    PathReadContext, RelationalAuthorizationPathPlan, RelationalAuthorizationWitness,
};

pub(super) fn apply(
    context: &PathReadContext<'_, '_, '_>,
    path: &RelationalAuthorizationPathPlan,
    ordinal: usize,
    frontier: &mut BTreeSet<RelationalAuthorizationWitness>,
    state: &mut PathEvaluationState<'_>,
) {
    for constraint in path
        .exact_adjacencies()
        .iter()
        .filter(|constraint| constraint.traversal_ordinal() == ordinal)
    {
        frontier.retain(|witness| {
            let source = witness.current();
            let relation_ids =
                relation_ids_for_step(context, source, constraint.traversal(), state);
            let mut observed = Vec::with_capacity(relation_ids.len());
            for relation_id in relation_ids {
                state.counters.relation_records_inspected += 1;
                let Some((candidate, kind)) = traverse_relation(
                    context.view,
                    relation_id,
                    source,
                    constraint.traversal(),
                    &mut state.dependencies.relations,
                ) else {
                    continue;
                };
                state.dependencies.entities.insert(candidate);
                if entity_is_live_kind(context.view, candidate, kind, state.counters) {
                    observed.push(candidate);
                }
            }
            observed.sort_unstable();
            observed == constraint.expected_entities()
        });
    }
}
