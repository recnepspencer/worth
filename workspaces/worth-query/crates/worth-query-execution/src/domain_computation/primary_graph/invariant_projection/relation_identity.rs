use worth_relational::facade::identity::RelationId;

use super::WorthQueryInvariantEntityIdentity;
use std::marker::PhantomData;

/// A relation observed by an invariant projection, with the identities of its
/// source and target entities.
#[derive(Clone, Debug, Eq, PartialEq)]
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
