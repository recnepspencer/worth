pub(super) mod admitted;

use super::RelationalAuthorizationPathPlan;
use crate::identity::data::EntityId;

fn unique_anchor_at(path: &RelationalAuthorizationPathPlan, ordinal: usize) -> Option<EntityId> {
    let mut anchors = path
        .entity_anchors()
        .iter()
        .filter(|anchor| anchor.traversal_ordinal() == ordinal);
    let first = anchors.next()?.entity();
    anchors
        .all(|anchor| anchor.entity() == first)
        .then_some(first)
}
