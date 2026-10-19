//! Typed graph targets bind to an already-authored computed value.
use super::ApplicationOperationBodyPathValueAssignment as Assignment;
use crate::{
    application_query::ApplicationQueryRootPath,
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationFieldRef,
        ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
        ApplicationOperationBodyPathFieldSource, ApplicationOperationBodyValue,
        ApplicationScalarValueBinding, DeclaredApplicationFieldValue, NoApplicationUnit,
        OperationReads, OperationWrites, WritableCapability,
    },
};
impl<S, O, I> ApplicationOperationBodyBuilder<S, O, I> {
    /// Assign a checked value to one live endpoint at every guard-free path hop.
    /// The path starts at the admitted scope; it grants no additional permission.
    pub fn assign_value_at_path<Root, E, A, F, V, W, Q, B>(
        mut self,
        path: ApplicationQueryRootPath<S, Root, E>,
        field: ApplicationFieldRef<S, E, A, F, V, W, Q, NoApplicationUnit>,
        value: &ApplicationOperationBodyValue<S, O, B>,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        E: OperationReads<O>,
        F: OperationReads<O>
            + OperationWrites<O>
            + DeclaredApplicationFieldValue<Value = V, Binding = B>,
        W: WritableCapability,
        B: ApplicationScalarValueBinding<Value = V>,
    {
        let target = ApplicationOperationBodyPathFieldSource {
            path: path.into_meaning(),
            field: ApplicationFieldBindingLocus::new(field.entity(), field.aspect(), field.field()),
            presence: F::PRESENCE,
        };
        if !target.has_supported_shape()
            || F::PRESENCE != crate::application_schema::ApplicationFieldPresence::Required
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
