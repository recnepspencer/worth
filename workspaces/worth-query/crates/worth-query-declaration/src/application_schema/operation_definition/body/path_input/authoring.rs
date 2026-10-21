//! A path guard never changes the body's scope to the terminal field's entity.
use super::ApplicationOperationBodyPathInputExpectation as Guard;
use crate::{
    application_query::ApplicationQueryRootPath,
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationFieldRef, ApplicationFieldUnit,
        ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
        ApplicationOperationBodyPathFieldSource, ApplicationScalarValueBinding,
        DeclaredApplicationFieldValue, OperationReads,
    },
};
impl<S, O, I> ApplicationOperationBodyBuilder<S, O, I> {
    /// Require one present terminal field to equal an original captured scalar.
    /// Each guard-free path hop must select one distinct live endpoint at execution.
    pub fn expect_input_from_path<Root, Selected, A, F, V, W, Q, U>(
        mut self,
        path: ApplicationQueryRootPath<S, Root, Selected>,
        field: ApplicationFieldRef<S, Selected, A, F, V, W, Q, U>,
        member: &str,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        Selected: OperationReads<O>,
        F: OperationReads<O> + DeclaredApplicationFieldValue<Value = V>,
        U: ApplicationFieldUnit,
    {
        let source = ApplicationOperationBodyPathFieldSource {
            path: path.into_meaning(),
            field: ApplicationFieldBindingLocus::new(field.entity(), field.aspect(), field.field()),
            presence: F::PRESENCE,
        };
        if !source.has_supported_shape() || member.is_empty() || member.trim() != member {
            return Err(Denial::IncompatibleSourceBinding);
        }
        self.in_named_scope(source.path.start_entity())?;
        self.path_input_expectations.push(Guard {
            source,
            binding: F::Binding::IDENTITY,
            scalar_family: F::Binding::SCALAR_FAMILY,
            member: member.into(),
        });
        Ok(self)
    }
}
