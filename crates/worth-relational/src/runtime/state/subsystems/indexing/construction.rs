//! Create and fork the index subsystem with independent publication gates.
use super::*;
impl RuntimeSubsystem for IndexingSubsystem {
    type Config = ();

    fn new(_: &Self::Config) -> Self {
        Self {
            state: RuntimeOwnedState::new(IndexingState::empty()),
            definition_publication: RuntimeOwnedState::default(),
            generation_publication: Arc::default(),
        }
    }

    fn fork(&self) -> Self {
        Self {
            state: RuntimeOwnedState::new(self.snapshot()),
            definition_publication: RuntimeOwnedState::default(),
            generation_publication: Arc::default(),
        }
    }
}
