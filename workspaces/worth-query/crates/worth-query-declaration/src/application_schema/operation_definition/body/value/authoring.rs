//! Typed references attach shared value meaning, never caller-provided results.
use super::*;
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldRef,
    ApplicationOperationBodyAuthoringDenial as AuthoringDenial, ApplicationOperationBodyBuilder,
    ApplicationOperationBodyCreatedField, ApplicationOperationBodyCreationBuilder,
    ApplicationOperationBodyScopeField, DeclaredApplicationFieldValue, NoApplicationUnit,
    OperationWrites, WritableCapability,
};
impl<S, O, I> ApplicationOperationBodyBuilder<S, O, I> {
    pub fn assign_value<E, A, F, V, W, Q, B>(
        mut self,
        field: ApplicationFieldRef<S, E, A, F, V, W, Q, NoApplicationUnit>,
        value: &ApplicationOperationBodyValue<S, O, B>,
    ) -> Result<Self, AuthoringDenial>
    where
        F: OperationWrites<O> + DeclaredApplicationFieldValue<Value = V, Binding = B>,
        W: WritableCapability,
        B: ApplicationScalarValueBinding<Value = V>,
    {
        self.retain_value(&value.program)?;
        self.steps
            .push(ApplicationOperationBodyScopeField::AssignValue {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: B::IDENTITY,
                value: value.program.identity.clone(),
            });
        Ok(self)
    }
    pub(in crate::application_schema::operation_definition::body) fn retain_value(
        &mut self,
        value: &ApplicationOperationBodyValueProgram,
    ) -> Result<(), AuthoringDenial> {
        if let Some(retained) = self
            .values
            .iter()
            .find(|retained| retained.identity == value.identity)
        {
            if retained != value {
                return Err(AuthoringDenial::DuplicateAttachment);
            }
        } else {
            self.values.push(value.clone());
        }
        Ok(())
    }
}
impl<S, O, I, E> ApplicationOperationBodyCreationBuilder<S, O, I, E> {
    pub fn initialize_value<A, F, V, W, Q, B>(
        mut self,
        field: ApplicationFieldRef<S, E, A, F, V, W, Q, NoApplicationUnit>,
        value: &ApplicationOperationBodyValue<S, O, B>,
    ) -> Result<Self, AuthoringDenial>
    where
        F: OperationWrites<O> + DeclaredApplicationFieldValue<Value = V, Binding = B>,
        B: ApplicationScalarValueBinding<Value = V>,
    {
        self.parent.retain_value(&value.program)?;
        self.fields
            .push(ApplicationOperationBodyCreatedField::Computed {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: B::IDENTITY,
                value: value.program.identity.clone(),
            });
        Ok(self)
    }
}
