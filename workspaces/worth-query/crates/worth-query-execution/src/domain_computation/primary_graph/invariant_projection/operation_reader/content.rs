//! Typed content reads retain the ordinary field dependency and sticky refusal.
use super::WorthQueryApplicationOperationInvariantProjectionReader as Reader;
use crate::domain_computation::primary_graph::{
    AdmittedContent, ContentReadDenial as Denial, HandlerExecutionDenial,
    WorthQueryInvariantEntityIdentity,
};

use worth_query_declaration::facade::application_schema::{
    ApplicationContentBinding, ApplicationContentFieldRef,
};
use worth_query_installation::facade::ApplicationSchema;

impl<S: ApplicationSchema, O> Reader<'_, '_, S, O> {
    pub(in crate::domain_computation::primary_graph) fn decision_content<E, F, C>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<S, E>,
        content: ApplicationContentFieldRef<F, C>,
    ) -> Result<Option<AdmittedContent<C>>, Denial>
    where
        F: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = E,
        >,

        F::Binding:
            worth_query_declaration::facade::application_schema::ApplicationContentReferenceBinding,
        C: ApplicationContentBinding,
    {
        let field = content.field();
        self.require_decision_field(identity, field)
            .map_err(|cause| Denial::Field(HandlerExecutionDenial::new(cause)))?;
        self.reader.content(identity, content)
    }

    /// Reads admitted bytes under this operation's declared field permission.
    pub fn content<E, F, C>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<S, E>,
        content: ApplicationContentFieldRef<F, C>,
    ) -> Result<Option<AdmittedContent<C>>, Denial>
    where
        F: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = E,
        >,
        F::Binding:
            worth_query_declaration::facade::application_schema::ApplicationContentReferenceBinding,
        C: ApplicationContentBinding,
    {
        self.decision_content(identity, content)
    }
}
