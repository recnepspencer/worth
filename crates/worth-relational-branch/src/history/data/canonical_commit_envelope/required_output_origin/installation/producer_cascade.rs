//! Canonical producer retirement membership.
use super::CanonicalRequiredOutputOrigin;
use crate::identity::data::EntityId;
use std::sync::Arc;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CanonicalProducerRetirement {
    pub anchors: Vec<EntityId>,
    pub publications: Vec<Arc<CanonicalRequiredOutputOrigin>>,
    #[serde(skip)]
    pub _decoded_memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl PartialEq for CanonicalProducerRetirement {
    fn eq(&self, other: &Self) -> bool {
        self.anchors == other.anchors && self.publications == other.publications
    }
}
impl Eq for CanonicalProducerRetirement {}
