use crate::identity::data::{EntityId, RelationId};

use super::{relation_ids_for_step, PathEvaluationState, PathReadContext};
use crate::authorization::{
    RelationalAuthorizationPathPlan, RelationalAuthorizationTraversal,
    RelationalAuthorizationTraversalDirection,
};

pub(super) fn unique_anchor_at(
    path: &RelationalAuthorizationPathPlan,
    ordinal: usize,
) -> Option<EntityId> {
    let mut anchors = path
        .entity_anchors()
        .iter()
        .filter(|anchor| anchor.traversal_ordinal() == ordinal);
    let first = anchors.next()?.entity();
    anchors
        .all(|anchor| anchor.entity() == first)
        .then_some(first)
}

pub(super) fn relation_ids_for_anchored_step(
    context: &PathReadContext<'_, '_, '_>,
    anchor: EntityId,
    traversal: &RelationalAuthorizationTraversal,
    state: &mut PathEvaluationState<'_>,
) -> Vec<RelationId> {
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
    relation_ids_for_step(context, anchor, &inverse, state)
}
