use worth_relational::facade::identity::RelationId;

use super::{WorthQueryInvariantEntityIdentity, WorthQueryInvariantRelation};

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
