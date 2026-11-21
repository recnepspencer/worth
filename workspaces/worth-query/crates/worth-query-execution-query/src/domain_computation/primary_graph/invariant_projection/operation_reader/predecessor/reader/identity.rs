//! Historical read handles cannot pass as a current mutation projection identity.
use super::*;

/// A historical entity is descriptive read data, never a current mutation handle.
/// The current counterpart is accepted by the ordinary reader:
///
/// Historical identities cannot take that path:
pub struct BeforeEntity<S, E> {
    pub(crate) identity: WorthQueryInvariantEntityIdentity<S, E>,
}
impl<S, E> BeforeEntity<S, E> {
    /// Descriptive full-generation coordinate. Reusing it at C requires fresh
    /// current liveness and declared-read checks through the current reader.
    pub fn entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.identity.entity_id()
    }
}
pub struct BeforeRelation<S, R, F, T> {
    from: BeforeEntity<S, F>,
    to: BeforeEntity<S, T>,
    relation: worth_relational::facade::identity::RelationId,
    marker: PhantomData<fn() -> R>,
}
impl<S, R, F, T> BeforeRelation<S, R, F, T> {
    pub fn from_read(relation: WorthQueryInvariantRelation<S, R, F, T>) -> Self {
        use worth_query_execution_lineage::facade::ports::InvariantRelationInternalPort as _;
        let (relation_id, from, to) = relation.into_parts();
        Self {
            from: BeforeEntity { identity: from },
            to: BeforeEntity { identity: to },
            relation: relation_id,
            marker: PhantomData,
        }
    }
    pub fn from(&self) -> &BeforeEntity<S, F> {
        &self.from
    }
    pub fn to(&self) -> &BeforeEntity<S, T> {
        &self.to
    }
    pub fn into_from(self) -> BeforeEntity<S, F> {
        self.from
    }
    pub fn into_to(self) -> BeforeEntity<S, T> {
        self.to
    }
    pub fn relation_id(&self) -> worth_relational::facade::identity::RelationId {
        self.relation
    }
}
