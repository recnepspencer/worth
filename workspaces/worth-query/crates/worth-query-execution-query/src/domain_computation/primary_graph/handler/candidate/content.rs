//! Typed wrappers preserve each operation's existing field write capabilities.
use super::CandidateWriter;
use crate::domain_computation::primary_graph::{
    ContentWriteDenial, WorthQueryApplicationEffectEntity,
};

use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{ApplicationContentBinding, ApplicationContentFieldRef},
};
use worth_query_installation::facade::{ApplicationSchema, OperationCreates};
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> CandidateWriter<'_, S, B> {
    pub fn initialize_content<E, F, C>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<S, E>,
        field: ApplicationContentFieldRef<F, C>,
        value: &C::Value,
    ) -> Result<(), ContentWriteDenial>
    where
        E: OperationCreates<B::Operation>,
        F: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = E,
        >,

        F::Binding:
            worth_query_declaration::facade::application_schema::ApplicationContentReferenceBinding,
        C: ApplicationContentBinding,
    {
        self.candidate.initialize_content(
            &self.execution,
            self.value_bindings,
            target,
            field,
            value,
        )
    }
    pub fn write_content<E, F, C>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<S, E>,
        field: ApplicationContentFieldRef<F, C>,
        value: &C::Value,
    ) -> Result<(), ContentWriteDenial>
    where
        F: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = E,
        >,

        F::Binding:
            worth_query_declaration::facade::application_schema::ApplicationContentReferenceBinding,
        C: ApplicationContentBinding,
    {
        self.candidate
            .write_content(&self.execution, self.value_bindings, target, field, value)
    }
}
