use worth_relational::facade::identity::RelationId;

use super::WorthQueryInvariantEntityIdentity;
use std::marker::PhantomData;

/// A relation observed by an invariant projection, with the identities of its
/// source and target entities.
pub struct WorthQueryInvariantRelation<Schema, Relation, From, To> {
    pub(super) relation_id: RelationId,
    pub(super) from: WorthQueryInvariantEntityIdentity<Schema, From>,
    pub(super) to: WorthQueryInvariantEntityIdentity<Schema, To>,
    pub(super) _relation: PhantomData<fn() -> Relation>,
}

impl<Schema, Relation, From, To> WorthQueryInvariantRelation<Schema, Relation, From, To> {
    pub const fn from(&self) -> &WorthQueryInvariantEntityIdentity<Schema, From> {
        &self.from
    }

    pub const fn to(&self) -> &WorthQueryInvariantEntityIdentity<Schema, To> {
        &self.to
    }

    pub fn into_to(self) -> WorthQueryInvariantEntityIdentity<Schema, To> {
        self.to
    }

    pub fn into_from(self) -> WorthQueryInvariantEntityIdentity<Schema, From> {
        self.from
    }

    pub const fn relation_id(&self) -> RelationId {
        self.relation_id
    }
}

impl<S, R, F, T> Clone for WorthQueryInvariantRelation<S, R, F, T> {
    fn clone(&self) -> Self {
        Self {
            relation_id: self.relation_id,
            from: self.from.clone(),
            to: self.to.clone(),
            _relation: PhantomData,
        }
    }
}
impl<S, R, F, T> std::fmt::Debug for WorthQueryInvariantRelation<S, R, F, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryInvariantRelation")
            .field("relation_id", &self.relation_id)
            .field("from", &self.from)
            .field("to", &self.to)
            .finish()
    }
}
impl<S, R, F, T> PartialEq for WorthQueryInvariantRelation<S, R, F, T> {
    fn eq(&self, other: &Self) -> bool {
        self.relation_id == other.relation_id && self.from == other.from && self.to == other.to
    }
}
impl<S, R, F, T> Eq for WorthQueryInvariantRelation<S, R, F, T> {}
mod court_doors_for_test;
