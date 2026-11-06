//! Scope resolution retains the field selection until installed access.
use super::{
    ApplicationQueryFieldScope, ApplicationQueryPrincipalScope, ApplicationQueryScopeBinding,
};
use crate::application_schema::fragment::RecordSelection;

type ResolvedFieldParts<Schema, Binding> = (
    <Binding as ApplicationQueryScopeBinding<Schema>>::Field,
    <Binding as ApplicationQueryScopeBinding<Schema>>::Value,
);
pub trait ApplicationQueryScopeResolution<Schema, PrincipalIdentity>:
    ApplicationQueryScopeBinding<Schema>
{
    fn into_field_parts(
        self,
        principal_identity: &PrincipalIdentity,
    ) -> ResolvedFieldParts<Schema, Self>;
}
impl<Schema, Field: RecordSelection, PrincipalIdentity>
    ApplicationQueryScopeResolution<Schema, PrincipalIdentity>
    for ApplicationQueryFieldScope<Field>
{
    fn into_field_parts(self, principal: &PrincipalIdentity) -> (Field, Field::Value) {
        self.into_field_parts(principal)
    }
}
impl<Schema, Field: RecordSelection> ApplicationQueryScopeResolution<Schema, Field::Value>
    for ApplicationQueryPrincipalScope<Field>
where
    Field::Value: Clone,
{
    fn into_field_parts(self, principal: &Field::Value) -> (Field, Field::Value) {
        self.into_field_parts(principal)
    }
}
