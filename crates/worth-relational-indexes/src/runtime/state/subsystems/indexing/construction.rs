//! Create and fork the index subsystem with independent publication gates.
use super::*;
impl IndexingSubsystem {
    pub fn fork_catalog(&self) -> Self {
        Self {
            state: SharedIndexingState::new(self.snapshot()),
            definition_publication: RuntimeOwnedState::default(),
            generation_publication: Arc::default(),
        }
    }
}
