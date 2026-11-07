//! Historical read handles cannot pass as a current mutation projection identity.
use super::*;

/// A historical entity is descriptive read data, never a current mutation handle.
/// The current counterpart is accepted by the ordinary reader:
///
/// ```
/// use worth_query_declaration::facade::application_schema::ApplicationSchema;
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryApplicationOperationInvariantProjectionReader as Reader,
///     WorthQueryInvariantEntityIdentity as CurrentEntity, PredecessorEntity,
/// };
/// fn current<S: ApplicationSchema, E, O>(reader: &Reader<'_, '_, S, O>, entity: &CurrentEntity<S, E>) {
///     let _ = reader.mutation_target(entity);
///     let _historical_type: Option<&PredecessorEntity<S, E>> = None;
/// }
/// ```
///
/// Historical identities cannot take that path:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_schema::ApplicationSchema;
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryApplicationOperationInvariantProjectionReader as Reader,
///     PredecessorEntity,
/// };
/// fn historical<S: ApplicationSchema, E, O>(reader: &Reader<'_, '_, S, O>, entity: &PredecessorEntity<S, E>) {
///     let _ = reader.mutation_target(entity);
/// }
/// ```
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
