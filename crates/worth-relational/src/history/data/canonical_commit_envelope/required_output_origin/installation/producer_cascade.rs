//! Canonical producer retirement membership.
use super::CanonicalRequiredOutputOrigin;
use crate::identity::data::EntityId;
use std::sync::Arc;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct CanonicalProducerRetirement {
    pub(crate) anchors: Vec<EntityId>,
    pub(crate) publications: Vec<Arc<CanonicalRequiredOutputOrigin>>,
    #[serde(skip)]
    pub(crate) _decoded_memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl PartialEq for CanonicalProducerRetirement {
    fn eq(&self, other: &Self) -> bool {
        self.anchors == other.anchors && self.publications == other.publications
    }
}
impl Eq for CanonicalProducerRetirement {}
