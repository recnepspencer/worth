//! A superseded coupled round's relation creates leave the correspondence.
use super::CommitCreatedRelationBindings;
use crate::identity::data::RelationId;
use std::collections::BTreeSet;

impl CommitCreatedRelationBindings {
    /// Remove every binding to a relation in `withdrawn`. O(bindings · log n).
    pub fn withdraw(&mut self, withdrawn: &BTreeSet<RelationId>) {
        self.by_exact_reference
            .retain(|_, id| !withdrawn.contains(id));
        self.by_stable_reference
            .retain(|_, id| !withdrawn.contains(id));
    }
}
