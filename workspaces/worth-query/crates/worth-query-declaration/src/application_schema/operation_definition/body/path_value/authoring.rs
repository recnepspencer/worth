//! Typed graph targets bind to an already-authored computed value.
use super::ApplicationOperationBodyPathValueAssignment as Assignment;
use crate::{
    application_query::ApplicationQueryRootPath,
    application_schema::{
        ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
        ApplicationOperationBodyPathFieldSource, ApplicationOperationBodyValue, OperationReads,
    },
};
impl<S, O, I> ApplicationOperationBodyBuilder<S, O, I> {
    /// Assign a checked value to one live endpoint at every guard-free path hop.
    /// The path starts at the admitted scope; it grants no additional permission.
    pub fn assign_value_at_path<Root, E, F, B>(
        mut self,
        path: ApplicationQueryRootPath<S, Root, E>,
        field: F,
        value: &ApplicationOperationBodyValue<S, O, B>,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        E: OperationReads<O>,
        F: crate::application_schema::fragment::RecordSelection<Entity = E, Binding = B>,
        B: crate::application_schema::ApplicationReadableScalarValueBinding,
    {
        let field = super::super::record_authoring::resolve(field)?;
        if !field.writable() {
            return Err(Denial::IncompatibleSourceBinding);
        }
        let target = ApplicationOperationBodyPathFieldSource {
            path: path.into_meaning(),
            field: field.locus().clone(),
            presence: field.presence(),
        };
        if !target.has_supported_shape()
            || field.presence() != crate::application_schema::ApplicationFieldPresence::Required
        {
            return Err(Denial::IncompatibleSourceBinding);
        }
        self.in_named_scope(target.path.start_entity())?;
        self.retain_value(&value.program)?;
        self.path_value_assignments.push(Assignment {
            target,
            binding: B::IDENTITY,
            value: value.program.identity().into(),
        });
        Ok(self)
    }
}
