//! Sealed test doors over this module's private members.
use super::*;

impl SourceFacts {
    #[doc(hidden)]
    pub fn new_for_test(entities: Vec<EntityId>, aspects: Vec<WorthQueryObservedFieldRevision>, adjacencies: Vec<WorthQueryObservedAdjacencyRevision>, walks: Option<Arc<StructuralWalkDependencies>>, derived: Option<Arc<DerivedSourceDependencies>>, allocation: Option<Arc<SourceFactsAllocation>>) -> Self {
        Self { entities, aspects, adjacencies, walks, derived, allocation }
    }
}
