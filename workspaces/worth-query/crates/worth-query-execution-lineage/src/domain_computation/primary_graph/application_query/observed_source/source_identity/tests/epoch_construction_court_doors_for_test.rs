//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryObservedSourceEpoch {
    #[doc(hidden)]
    pub fn new_for_test(
        query: [u8; 32],
        parameters: [u8; 32],
        root: worth_relational::facade::identity::EntityId,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        observation_generation: u64,
        identity: [u8; 32],
    ) -> Self {
        Self::new(
            query,
            parameters,
            root,
            occurrence,
            observation_generation,
            identity,
        )
    }
}
