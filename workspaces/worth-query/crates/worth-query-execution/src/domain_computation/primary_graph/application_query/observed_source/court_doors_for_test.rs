//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryObservedSourceFootprint {
    #[doc(hidden)]
    pub fn new_for_test(root: EntityId, complete: bool, entities: Vec<EntityId>, aspects: Vec<WorthQueryObservedFieldRevision>, root_selection: Option<std::sync::Arc<WorthQueryObservedRootSelection>>, adjacencies: Vec<WorthQueryObservedAdjacencyRevision>) -> Self {
        Self { root, complete, entities, aspects, root_selection, adjacencies }
    }
}

impl WorthQueryObservedSourceFootprint {
    #[doc(hidden)]
    pub fn aspects_for_test(&self) -> &Vec<WorthQueryObservedFieldRevision> {
        &self.aspects
    }
}
