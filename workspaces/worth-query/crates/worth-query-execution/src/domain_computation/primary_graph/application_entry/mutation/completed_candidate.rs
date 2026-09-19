use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram;

/// Candidate program and typed consumer result completed by the same handler.
pub struct WorthQueryCompletedMutationCandidate<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    program: WorthQueryApplicationEffectProgram<
        Schema,
        Binding::Operation,
        Binding::Input,
        <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    >,
    result: Binding::Result,
}

impl<Schema, Binding> WorthQueryCompletedMutationCandidate<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(in crate::domain_computation::primary_graph) fn new(
        program: WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        result: Binding::Result,
    ) -> Self {
        Self { program, result }
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        Binding::Result,
    ) {
        (self.program, self.result)
    }
}
