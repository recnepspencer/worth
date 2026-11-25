//! A superseded coupled round's creates leave the candidate's correspondence.
use super::CommitCreatedEntityBindings;
use crate::identity::data::EntityId;
use crate::transactions::data::primitives::CreatedEntityRef;
use std::collections::BTreeSet;

impl CommitCreatedEntityBindings {
    /// Every exact create reference and the identity it was given.
    pub fn iter(&self) -> impl Iterator<Item = (&CreatedEntityRef, &EntityId)> {
        self.by_exact_reference.iter()
    }

    /// Remove every binding to an identity in `withdrawn` (a superseded
    /// round's creates). O(bindings · log withdrawn).
    pub fn withdraw(&mut self, withdrawn: &BTreeSet<EntityId>) {
        self.by_exact_reference
            .retain(|_, id| !withdrawn.contains(id));
        self.by_stable_reference
            .retain(|_, id| !withdrawn.contains(id));
        self.by_entity_identity.retain(|id| !withdrawn.contains(id));
    }
}
