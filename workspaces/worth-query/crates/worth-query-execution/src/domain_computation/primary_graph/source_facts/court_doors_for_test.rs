//! Sealed test doors over this module's private members.
use super::*;

impl SourceFacts {
    #[doc(hidden)]
    pub fn new_for_test(
        entities: Vec<EntityId>,
        aspects: Vec<WorthQueryObservedFieldRevision>,
        adjacencies: Vec<WorthQueryObservedAdjacencyRevision>,
        walks: Option<Arc<StructuralWalkDependencies>>,
        derived: Option<Arc<DerivedSourceDependencies>>,
        allocation: Option<Arc<SourceFactsAllocation>>,
    ) -> Self {
        Self {
            entities,
            aspects,
            adjacencies,
            walks,
            derived,
            allocation,
        }
    }
}

impl SourceFacts {
    #[doc(hidden)]
    pub fn aspects_for_test(&self) -> &Vec<WorthQueryObservedFieldRevision> {
        &self.aspects
    }
}

impl SourceFacts {
    #[doc(hidden)]
    pub fn entities_for_test(&self) -> &Vec<EntityId> {
        &self.entities
    }
}

impl SourceFacts {
    #[doc(hidden)]
    pub fn walks_for_test(&self) -> &Option<Arc<StructuralWalkDependencies>> {
        &self.walks
    }
}
