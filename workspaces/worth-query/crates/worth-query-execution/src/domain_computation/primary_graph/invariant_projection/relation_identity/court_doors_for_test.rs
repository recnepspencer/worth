//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Relation, From, To> WorthQueryInvariantRelation<Schema, Relation, From, To> {
    #[doc(hidden)]
    pub fn from_for_test(&self) -> &WorthQueryInvariantEntityIdentity<Schema, From> {
        &self.from
    }
}

impl<Schema, Relation, From, To> WorthQueryInvariantRelation<Schema, Relation, From, To> {
    #[doc(hidden)]
    pub fn relation_id_for_test(&self) -> &RelationId {
        &self.relation_id
    }
}
