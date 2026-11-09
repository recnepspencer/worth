//! Typed references attach shared value meaning, never caller-provided results.
use super::*;
use crate::application_schema::{
    ApplicationOperationBodyAuthoringDenial as AuthoringDenial, ApplicationOperationBodyBuilder,
    ApplicationOperationBodyCreatedField, ApplicationOperationBodyCreationBuilder,
    ApplicationOperationBodyScopeField,
};
impl<S, O, I> ApplicationOperationBodyBuilder<S, O, I> {
    pub fn assign_value<F, B>(
        mut self,
        field: F,
        value: &ApplicationOperationBodyValue<S, O, B>,
    ) -> Result<Self, AuthoringDenial>
    where
        F: crate::application_schema::fragment::RecordSelection<Binding = B>,
        B: crate::application_schema::ApplicationReadableScalarValueBinding,
    {
        let field = super::super::record_authoring::resolve(field)?;
        if !field.writable() {
            return Err(AuthoringDenial::IncompatibleSourceBinding);
        }
        self.retain_value(&value.program)?;
        self.steps
            .push(ApplicationOperationBodyScopeField::AssignValue {
                field: field.locus().clone(),
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
    pub fn initialize_value<F, B>(
        mut self,
        field: F,
        value: &ApplicationOperationBodyValue<S, O, B>,
    ) -> Result<Self, AuthoringDenial>
    where
        F: crate::application_schema::fragment::RecordSelection<Entity = E, Binding = B>,
        B: crate::application_schema::ApplicationReadableScalarValueBinding,
    {
        let field = super::super::record_authoring::resolve(field)?;
        self.parent.retain_value(&value.program)?;
        self.fields
            .push(ApplicationOperationBodyCreatedField::Computed {
                field: field.locus().clone(),
                binding: B::IDENTITY,
                value: value.program.identity.clone(),
            });
        Ok(self)
    }
}
