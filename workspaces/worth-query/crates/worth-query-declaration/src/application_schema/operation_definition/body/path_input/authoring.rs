//! A path guard never changes the body's scope to the terminal field's entity.
use super::ApplicationOperationBodyPathInputExpectation as Guard;
use crate::{
    application_query::ApplicationQueryRootPath,
    application_schema::{
        ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
        ApplicationOperationBodyPathFieldSource, ApplicationScalarValueBinding, OperationReads,
    },
};
impl<S, O, I> ApplicationOperationBodyBuilder<S, O, I> {
    /// Require one present terminal field to equal an original captured scalar.
    /// Each guard-free path hop must select one distinct live endpoint at execution.
    pub fn expect_input_from_path<Root, Selected, F>(
        mut self,
        path: ApplicationQueryRootPath<S, Root, Selected>,
        field: F,
        member: &str,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        Selected: OperationReads<O>,
        F: crate::application_schema::fragment::RecordSelection<Entity = Selected>,
    {
        let field = super::super::record_authoring::resolve(field)?;
        let source = ApplicationOperationBodyPathFieldSource {
            path: path.into_meaning(),
            field: field.locus().clone(),
            presence: field.presence(),
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
