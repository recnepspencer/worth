//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryObservedSourceFootprint {
    #[doc(hidden)]
    pub fn new_for_test(root: EntityId, complete: bool, entities: Vec<EntityId>, aspects: Vec<WorthQueryObservedFieldRevision>, adjacencies: Vec<WorthQueryObservedAdjacencyRevision>, root_selection: Option<std::sync::Arc<WorthQueryObservedRootSelection>>) -> Self {
        Self { root, complete, entities, aspects, adjacencies, root_selection }
    }
}
