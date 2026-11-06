//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationObservedRelation {
    #[doc(hidden)]
    pub fn from_for_test(&self) -> &EntityId {
        &self.from
    }
}

impl WorthQueryApplicationObservedRelation {
    #[doc(hidden)]
    pub fn relation_id_for_test(&self) -> &RelationId {
        &self.relation_id
    }
}

impl WorthQueryApplicationObservedRelation {
    #[doc(hidden)]
    pub fn to_for_test(&self) -> &EntityId {
        &self.to
    }
}

impl WorthQueryApplicationObservedFact {
    #[doc(hidden)]
    pub fn locator_identity_for_test(&self) -> String {
        Self::locator_identity(self)
    }
}

impl WorthQueryApplicationObservedRelation {
    #[doc(hidden)]
    pub fn new_for_test(relation_id: RelationId, from: EntityId, to: EntityId) -> Self {
        Self {
            relation_id,
            from,
            to,
        }
    }
}
