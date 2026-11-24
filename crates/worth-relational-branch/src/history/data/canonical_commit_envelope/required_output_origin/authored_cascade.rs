//! One atomic authored retirement retains every genuine route's own provenance.
use super::*;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct CanonicalAuthoredRetirement {
    pub anchor: EntityId,
    pub owned_members: bool,
    pub birth: CommitId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub co_anchors: Vec<(EntityId, CommitId)>,
    pub publications: Vec<Arc<CanonicalRequiredOutputOrigin>>,
    #[serde(skip)]
    pub _decoded_memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl CanonicalAuthoredRetirement {
    pub fn anchors(&self) -> impl Iterator<Item = EntityId> + '_ {
        std::iter::once(self.anchor).chain(self.co_anchors.iter().map(|(id, _)| *id))
    }
    pub fn contains_anchor(&self, id: EntityId) -> bool {
        self.anchors().any(|anchor| anchor == id)
    }
}

impl PartialEq for CanonicalAuthoredRetirement {
    fn eq(&self, other: &Self) -> bool {
        self.anchor == other.anchor
            && self.birth == other.birth
            && self.co_anchors == other.co_anchors
            && self.publications == other.publications
    }
}
impl Eq for CanonicalAuthoredRetirement {}
