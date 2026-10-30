//! Stable semantic ordering and hashing for an observed source epoch.

use std::hash::{Hash, Hasher};

use super::WorthQueryObservedSourceEpoch;

impl WorthQueryObservedSourceEpoch {
    fn ordering_coordinates(
        &self,
    ) -> (
        [u8; 32],
        [u8; 32],
        worth_relational::facade::identity::EntityId,
        worth_runtime_world::facade::ProductBranchIncarnation,
        u64,
        [u8; 32],
    ) {
        (
            self.query,
            self.parameters,
            self.root,
            self.occurrence,
            self.observation_generation,
            *self.meaning.identity(),
        )
    }
}

impl PartialEq for WorthQueryObservedSourceEpoch {
    fn eq(&self, other: &Self) -> bool {
        self.ordering_coordinates() == other.ordering_coordinates()
    }
}

impl Eq for WorthQueryObservedSourceEpoch {}

impl PartialOrd for WorthQueryObservedSourceEpoch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WorthQueryObservedSourceEpoch {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordering_coordinates()
            .cmp(&other.ordering_coordinates())
    }
}

impl Hash for WorthQueryObservedSourceEpoch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.query.hash(state);
        self.parameters.hash(state);
        self.root.hash(state);
        self.occurrence.hash(state);
        self.observation_generation.hash(state);
        self.meaning.identity().hash(state);
    }
}
