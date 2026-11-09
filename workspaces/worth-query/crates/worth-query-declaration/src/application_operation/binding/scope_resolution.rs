//! Scope resolution retains the field selection until installed access.
use super::{
    ApplicationMutationFieldScope, ApplicationMutationPrincipalScope,
    ApplicationMutationScopeBinding,
};
use crate::application_schema::fragment::RecordSelection;

type ResolvedFieldParts<Schema, Binding> = (
    <Binding as ApplicationMutationScopeBinding<Schema>>::Field,
    <Binding as ApplicationMutationScopeBinding<Schema>>::Value,
);
pub trait ApplicationMutationScopeResolution<Schema, PrincipalIdentity>:
    ApplicationMutationScopeBinding<Schema>
{
    fn into_field_parts(
        self,
        principal_identity: &PrincipalIdentity,
    ) -> ResolvedFieldParts<Schema, Self>;
}
impl<Schema, Field: RecordSelection, PrincipalIdentity>
    ApplicationMutationScopeResolution<Schema, PrincipalIdentity>
    for ApplicationMutationFieldScope<Field>
{
    fn into_field_parts(self, principal: &PrincipalIdentity) -> (Field, Field::Value) {
        ApplicationMutationFieldScope::into_field_parts(self, principal)
    }
}
impl<Schema, Field: RecordSelection> ApplicationMutationScopeResolution<Schema, Field::Value>
    for ApplicationMutationPrincipalScope<Field>
where
    Field::Value: Clone,
{
    fn into_field_parts(self, principal: &Field::Value) -> (Field, Field::Value) {
        ApplicationMutationPrincipalScope::into_field_parts(self, principal)
    }
}
