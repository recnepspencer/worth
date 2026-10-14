//! Execution-only body read; no raw-field API enters the application facade.
use super::super::super::HandlerExecutionDenial;
use super::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding, ApplicationSchema, DecisionReader,
};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    pub(in crate::domain_computation::primary_graph) fn body_scope_field(
        &mut self,
        field: &ApplicationFieldBindingLocus,
        expected: Option<&AspectValue>,
    ) -> Result<bool, HandlerExecutionDenial> {
        self.checkpoint().map_err(HandlerExecutionDenial::new)?;
        self.context_use.set(self.context_use.get().scope());
        self.reader
            .body_field(self.scope, field, expected)
            .map_err(HandlerExecutionDenial::new)
    }

    pub(in crate::domain_computation::primary_graph) fn body_scope_target(
        &mut self,
    ) -> Result<
        super::super::super::WorthQueryInvariantMutationTarget<
            Schema,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        HandlerExecutionDenial,
    > {
        self.context_use.set(self.context_use.get().scope());
        self.reader
            .mutation_target(self.scope)
            .map_err(HandlerExecutionDenial::new)
    }
}
