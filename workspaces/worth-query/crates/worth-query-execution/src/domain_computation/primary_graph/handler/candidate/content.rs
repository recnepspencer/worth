//! Typed wrappers preserve each operation's existing field write capabilities.
use super::CandidateWriter;
use crate::domain_computation::primary_graph::{
    ContentWriteDenial, WorthQueryApplicationEffectEntity,
};
use worth_foundational::facade::ContentRefId;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_schema::{
        ApplicationContentBinding, ApplicationContentFieldRef, ContentRefApplicationValueBinding,
    },
};
use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationSchema, DeclaredApplicationFieldValue, OperationCreates,
    OperationWrites, WritableCapability,
};
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> CandidateWriter<'_, S, B> {
    pub fn initialize_content<E, A, F, W, Q, U, C>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<S, E>,
        field: ApplicationContentFieldRef<S, E, A, F, W, Q, U, C>,
        value: &C::Value,
    ) -> Result<(), ContentWriteDenial>
    where
        E: OperationCreates<B::Operation>,
        F: OperationWrites<B::Operation>
            + DeclaredApplicationFieldValue<
                Value = ContentRefId,
                Binding = ContentRefApplicationValueBinding,
            >,
        U: ApplicationFieldUnit,
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
    pub fn write_content<E, A, F, W, Q, U, C>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<S, E>,
        field: ApplicationContentFieldRef<S, E, A, F, W, Q, U, C>,
        value: &C::Value,
    ) -> Result<(), ContentWriteDenial>
    where
        F: OperationWrites<B::Operation>
            + DeclaredApplicationFieldValue<
                Value = ContentRefId,
                Binding = ContentRefApplicationValueBinding,
            >,
        W: WritableCapability,
        U: ApplicationFieldUnit,
        C: ApplicationContentBinding,
    {
        self.candidate
            .write_content(&self.execution, self.value_bindings, target, field, value)
    }
}
