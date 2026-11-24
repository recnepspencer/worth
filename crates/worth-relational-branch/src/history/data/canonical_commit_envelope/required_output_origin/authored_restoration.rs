//! Canonical wire event for the reissue of recorded producer routes.
use super::{CanonicalRequiredOutputOrigin, CommitId, Deserialize, EntityId, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct CanonicalAuthoredRestoration {
    pub retirement: CommitId,
    pub original_anchor: EntityId,
    pub anchor: EntityId,
    pub owned_members: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub co_anchors: Vec<(EntityId, EntityId)>,
    pub entities: Vec<(EntityId, EntityId)>,
    pub relations: Vec<(
        crate::identity::data::RelationId,
        crate::identity::data::RelationId,
    )>,
    pub publications: Vec<Arc<CanonicalRequiredOutputOrigin>>,
    #[serde(skip)]
    pub _decoded_memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
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
pub mod inline_type_budget;
