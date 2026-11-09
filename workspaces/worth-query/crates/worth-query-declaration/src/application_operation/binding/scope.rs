//! Scope strategies carry a record selection; storage facts stay on its binding.
use super::ApplicationMutationScopeResolutionMode;
use crate::application_schema::fragment::RecordSelection;

pub trait ApplicationMutationScopeBinding<Schema> {
    type Scope;
    type Value: 'static;
    type Field: RecordSelection<Entity = Self::Scope, Value = Self::Value>;
    const RESOLUTION_MODE: ApplicationMutationScopeResolutionMode;
}

pub struct ApplicationMutationFieldScope<Field: RecordSelection> {
    pub(super) field: Field,
    pub(super) value: Field::Value,
}
impl<Field: RecordSelection> ApplicationMutationFieldScope<Field> {
    pub const fn new(field: Field, value: Field::Value) -> Self {
        Self { field, value }
    }
    pub fn into_field_parts<Principal>(self, _: &Principal) -> (Field, Field::Value) {
        (self.field, self.value)
    }
}
impl<Schema, Field: RecordSelection> ApplicationMutationScopeBinding<Schema>
    for ApplicationMutationFieldScope<Field>
{
    type Scope = Field::Entity;
    type Value = Field::Value;
    type Field = Field;
    const RESOLUTION_MODE: ApplicationMutationScopeResolutionMode =
        ApplicationMutationScopeResolutionMode::InputField;
}

pub struct ApplicationMutationPrincipalScope<Field: RecordSelection> {
    pub(super) field: Field,
}
impl<Field: RecordSelection> ApplicationMutationPrincipalScope<Field> {
    pub const fn new(field: Field) -> Self {
        Self { field }
    }
    pub fn into_field_parts(self, principal: &Field::Value) -> (Field, Field::Value)
    where
        Field::Value: Clone,
    {
        (self.field, principal.clone())
    }
}
impl<Schema, Field: RecordSelection> ApplicationMutationScopeBinding<Schema>
    for ApplicationMutationPrincipalScope<Field>
{
    type Scope = Field::Entity;
    type Value = Field::Value;
    type Field = Field;
    const RESOLUTION_MODE: ApplicationMutationScopeResolutionMode =
        ApplicationMutationScopeResolutionMode::PrincipalIdentity;
}
