//! Canonical wire event for the reissue of recorded producer routes.
use super::{CanonicalRequiredOutputOrigin, CommitId, Deserialize, EntityId, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CanonicalAuthoredRestoration {
    pub(crate) retirement: CommitId,
    pub(crate) original_anchor: EntityId,
    pub(crate) anchor: EntityId,
    pub(crate) owned_members: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) co_anchors: Vec<(EntityId, EntityId)>,
    pub(crate) entities: Vec<(EntityId, EntityId)>,
    pub(crate) relations: Vec<(
        crate::identity::data::RelationId,
        crate::identity::data::RelationId,
    )>,
    pub(crate) publications: Vec<Arc<CanonicalRequiredOutputOrigin>>,
    #[serde(skip)]
    pub(crate) _decoded_memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl PartialEq for CanonicalAuthoredRestoration {
    fn eq(&self, other: &Self) -> bool {
        self.retirement == other.retirement
            && self.original_anchor == other.original_anchor
            && self.anchor == other.anchor
            && self.entities == other.entities
            && self.relations == other.relations
            && self.co_anchors == other.co_anchors
            && self.publications == other.publications
    }
}
impl Eq for CanonicalAuthoredRestoration {}
#[cfg(test)]
#[path = "authored_restoration_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
