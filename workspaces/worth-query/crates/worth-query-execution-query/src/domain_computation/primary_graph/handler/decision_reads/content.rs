//! The ordinary typed decision door; content is field-bound, never a free lookup.
use super::super::invariant::DecisionReader;
use crate::domain_computation::primary_graph::{
    AdmittedContent, ContentReadDenial, WorthQueryInvariantEntityIdentity,
};

use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{ApplicationContentBinding, ApplicationContentFieldRef},
};
use worth_query_installation::facade::ApplicationSchema;
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    /// Uses the operation's installed decision field permission.
    pub fn content<E, F, C>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<S, E>,
        field: ApplicationContentFieldRef<F, C>,
    ) -> Result<Option<AdmittedContent<C>>, ContentReadDenial>
    where
        F: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = E,
        >,

        F::Binding:
            worth_query_declaration::facade::application_schema::ApplicationContentReferenceBinding,
        C: ApplicationContentBinding,
    {
        self.reader.decision_content(identity, field)
    }
}
