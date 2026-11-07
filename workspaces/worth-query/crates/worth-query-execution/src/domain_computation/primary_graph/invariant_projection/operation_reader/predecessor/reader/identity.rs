//! Historical read handles cannot pass as a current mutation projection identity.
use super::*;

pub struct PredecessorEntity<S, E> {
    pub(super) identity: WorthQueryInvariantEntityIdentity<S, E>,
}
impl<S, E> PredecessorEntity<S, E> {
    /// Descriptive full-generation coordinate. Reusing it at C requires fresh
    /// current liveness and declared-read checks through the current reader.
    pub fn entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.identity.entity_id()
    }
}
pub struct PredecessorRelation<S, R, F, T> {
    from: PredecessorEntity<S, F>,
    to: PredecessorEntity<S, T>,
    relation: worth_relational::facade::identity::RelationId,
    marker: PhantomData<fn() -> R>,
}
impl<S, R, F, T> PredecessorRelation<S, R, F, T> {
    pub(super) fn from_read(relation: WorthQueryInvariantRelation<S, R, F, T>) -> Self {
        Self {
            from: PredecessorEntity {
                identity: relation.from,
            },
            to: PredecessorEntity {
                identity: relation.to,
            },
            relation: relation.relation_id,
            marker: PhantomData,
        }
    }
    pub fn from(&self) -> &PredecessorEntity<S, F> {
        &self.from
    }
    pub fn to(&self) -> &PredecessorEntity<S, T> {
        &self.to
    }
    pub fn into_from(self) -> PredecessorEntity<S, F> {
        self.from
    }
    pub fn into_to(self) -> PredecessorEntity<S, T> {
        self.to
    }
    pub fn relation_id(&self) -> worth_relational::facade::identity::RelationId {
        self.relation
    }
}
