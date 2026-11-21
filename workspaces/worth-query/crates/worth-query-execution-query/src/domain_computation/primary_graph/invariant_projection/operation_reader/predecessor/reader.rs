//! Read-only tracked access; no raw reader, mutation target, query or computation.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationEntityRef;

mod identity;
pub use identity::{BeforeEntity, BeforeRelation};

pub struct BeforeDecisionReader<'borrow, 'reader, 'runtime, S, O> {
    pub(crate) reader: &'borrow mut WorthQueryApplicationOperationInvariantProjectionReader<
        'reader,
        'runtime,
        S,
        O,
    >,
}
impl<S: ApplicationSchema, O> BeforeDecisionReader<'_, '_, '_, S, O> {
    /// Descriptive version of this actual data basis, never a Current certificate.
    pub fn version(&self) -> worth_relational::facade::identity::VersionId {
        self.reader
            .reader
            .native
            .version(self.reader.reader.snapshot)
    }

    /// Reacquire the same full-generation identity at the predecessor's basis.
    pub fn entity<E: ApplicationEntityMarkerIdentity<S> + OperationReads<O>>(
        &mut self,
        entity: ApplicationEntityRef<S, E>,
        current: &WorthQueryInvariantEntityIdentity<S, E>,
    ) -> Result<BeforeEntity<S, E>, WorthQueryCurrentOutputDenial> {
        use worth_query_execution_lineage::facade::ports::InvariantEntityIdentityInternalPort as _;
        let _ = entity;
        let execution = *self.reader.reader.execution;
        self.reader
            .live_current_identity(&execution, E::IDENTIFIER, current.entity_id())
            .map(|identity| BeforeEntity { identity })
    }
    pub fn field<E, A, F, V>(
        &mut self,
        entity: &BeforeEntity<S, E>,
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
    /// Read admitted content at this predecessor basis under the independently
    /// authorized operation's field contract. The handle never becomes current.
    pub fn content<E, F, C>(
        &mut self,
        entity: &BeforeEntity<S, E>,
        content: worth_query_declaration::facade::application_schema::ApplicationContentFieldRef<
            F,
            C,
        >,
    ) -> Result<
        Option<crate::domain_computation::primary_graph::AdmittedContent<C>>,
        crate::domain_computation::primary_graph::ContentReadDenial,
    >
    where
        F: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = E,
        >,
        F::Binding:
            worth_query_declaration::facade::application_schema::ApplicationContentReferenceBinding,
        C: worth_query_declaration::facade::application_schema::ApplicationContentBinding,
    {
        self.reader.decision_content(&entity.identity, content)
    }
    pub fn relations_from<R: OperationReads<O>, F, T>(
        &mut self,
        relation: ApplicationRelationRef<S, R, F, T>,
        from: &BeforeEntity<S, F>,
    ) -> Result<Vec<BeforeRelation<S, R, F, T>>, WorthQueryInvariantProjectionTraversalDenial> {
        self.reader
            .decision_relations_from(relation, &from.identity)
            .map(|relations| {
                relations
                    .into_iter()
                    .map(BeforeRelation::from_read)
                    .collect()
            })
    }
    pub fn relations_to<R: OperationReads<O>, F, T>(
        &mut self,
        relation: ApplicationRelationRef<S, R, F, T>,
        to: &BeforeEntity<S, T>,
    ) -> Result<Vec<BeforeRelation<S, R, F, T>>, WorthQueryInvariantProjectionTraversalDenial> {
        self.reader
            .decision_relations_to(relation, &to.identity)
            .map(|relations| {
                relations
                    .into_iter()
                    .map(BeforeRelation::from_read)
                    .collect()
            })
    }
}
