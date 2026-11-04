//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryObservedSourceMeaningRegistry {
    #[doc(hidden)]
    pub fn state_for_test(&self) -> &Arc<Mutex<WorthQueryObservedSourceMeaningRegistryState>> {
        &self.state
    }
}

impl WorthQueryObservedSourceEpoch {
    #[doc(hidden)]
    pub fn checkpoint_occurrence_for_test(
        &self,
    ) -> worth_runtime_world::facade::ProductBranchIncarnation {
        Self::checkpoint_occurrence(self)
    }
}

impl WorthQueryObservedSourceEpoch {
    #[doc(hidden)]
    pub fn from_observation_for_test(
        query: &[u8; 32],
        parameters: &[u8; 32],
        root: worth_relational::facade::identity::EntityId,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        meaning: Arc<WorthQueryObservedSourceMeaning>,
    ) -> Option<Self> {
        Self::from_observation(query, parameters, root, selection, meaning)
    }
}

impl WorthQueryObservedSourceMeaningRegistryState {
    #[doc(hidden)]
    pub fn next_ordinal_for_test(&self) -> &u64 {
        &self.next_ordinal
    }
}

impl WorthQueryObservedSourceMeaning {
    #[doc(hidden)]
    pub fn checkpoint_identity_for_test(&self) -> WorthQueryCheckpointSourceIdentity {
        Self::checkpoint_identity(self)
    }
}

impl WorthQueryObservedSourceMeaning {
    #[doc(hidden)]
    pub fn identity_for_test(&self) -> &[u8; 32] {
        Self::identity(self)
    }
}

impl WorthQueryObservedSourceEpoch {
    #[doc(hidden)]
    pub fn observation_generation_for_test(&self) -> u64 {
        Self::observation_generation(self)
    }
}

impl WorthQueryObservedSourceMeaning {
    #[doc(hidden)]
    pub fn runtime_idempotency_identity_for_test(&self) -> WorthQueryRuntimeSourceIdentity {
        Self::runtime_idempotency_identity(self)
    }
}
