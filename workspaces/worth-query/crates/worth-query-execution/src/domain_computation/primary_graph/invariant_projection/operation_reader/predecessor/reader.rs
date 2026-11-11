//! Read-only tracked access; no raw reader, mutation target, query or computation.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationEntityRef;

mod identity;
pub use identity::{PredecessorEntity, PredecessorRelation};

pub struct PredecessorDecisionReader<'borrow, 'reader, 'runtime, S, O> {
    pub(super) reader: &'borrow mut WorthQueryApplicationOperationInvariantProjectionReader<
        'reader,
        'runtime,
        S,
        O,
    >,
}
impl<S: ApplicationSchema, O> PredecessorDecisionReader<'_, '_, '_, S, O> {
    /// Reacquire the same full-generation identity at the predecessor's basis.
    pub fn entity<E: ApplicationEntityMarkerIdentity<S> + OperationReads<O>>(
        &mut self,
        entity: ApplicationEntityRef<S, E>,
        current: &WorthQueryInvariantEntityIdentity<S, E>,
    ) -> Result<PredecessorEntity<S, E>, WorthQueryCurrentOutputDenial> {
        let _ = entity;
        let execution = *self.reader.reader.execution;
        self.reader
            .live_current_identity(&execution, E::IDENTIFIER, current.entity_id)
            .map(|identity| PredecessorEntity { identity })
    }
    pub fn field<E, A, F, V>(
        &mut self,
        entity: &PredecessorEntity<S, E>,
        field: F,
    ) -> Result<Option<V>, WorthQueryInvariantDecisionPlanDenial>
    where
        F: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = E,
            Aspect = A,
            Value = V,
        >,
        F::Binding: ApplicationReadableScalarValueBinding,
    {
        self.reader.decision_field(&entity.identity, field)
    }
    pub fn relations_from<R: OperationReads<O>, F, T>(
        &mut self,
        relation: ApplicationRelationRef<S, R, F, T>,
        from: &PredecessorEntity<S, F>,
    ) -> Result<Vec<PredecessorRelation<S, R, F, T>>, WorthQueryInvariantProjectionTraversalDenial>
    {
        self.reader
            .decision_relations_from(relation, &from.identity)
            .map(|relations| {
                relations
                    .into_iter()
                    .map(PredecessorRelation::from_read)
                    .collect()
            })
    }
    pub fn relations_to<R: OperationReads<O>, F, T>(
        &mut self,
        relation: ApplicationRelationRef<S, R, F, T>,
        to: &PredecessorEntity<S, T>,
    ) -> Result<Vec<PredecessorRelation<S, R, F, T>>, WorthQueryInvariantProjectionTraversalDenial>
    {
        self.reader
            .decision_relations_to(relation, &to.identity)
            .map(|relations| {
                relations
                    .into_iter()
                    .map(PredecessorRelation::from_read)
                    .collect()
            })
    }
}
