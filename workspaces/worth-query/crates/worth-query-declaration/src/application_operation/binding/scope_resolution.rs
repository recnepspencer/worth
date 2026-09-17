use crate::application_schema::{
    ApplicationFieldRef, ApplicationFieldUnit, DeclaredApplicationFieldValue, EqualityPredicate,
    WritePosture,
};

use super::{
    ApplicationMutationFieldScope, ApplicationMutationPrincipalScope,
    ApplicationMutationScopeBinding,
};

type MutationResolvedFieldParts<Schema, Binding> = (
    ApplicationFieldRef<
        Schema,
        <Binding as ApplicationMutationScopeBinding<Schema>>::Scope,
        <Binding as ApplicationMutationScopeBinding<Schema>>::Aspect,
        <Binding as ApplicationMutationScopeBinding<Schema>>::Field,
        <Binding as ApplicationMutationScopeBinding<Schema>>::Value,
        <Binding as ApplicationMutationScopeBinding<Schema>>::Write,
        EqualityPredicate,
        <Binding as ApplicationMutationScopeBinding<Schema>>::Unit,
    >,
    <Binding as ApplicationMutationScopeBinding<Schema>>::Value,
);

/// Resolves a declared mutation scope using the current borrowed principal.
pub trait ApplicationMutationScopeResolution<Schema, PrincipalIdentity>:
    ApplicationMutationScopeBinding<Schema>
{
    fn into_field_parts(
        self,
        principal_identity: &PrincipalIdentity,
    ) -> MutationResolvedFieldParts<Schema, Self>;
}

impl<Schema, Scope, Aspect, Field, Value, Write, Unit, PrincipalIdentity>
    ApplicationMutationScopeResolution<Schema, PrincipalIdentity>
    for ApplicationMutationFieldScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
where
    Field: DeclaredApplicationFieldValue<Value = Value>,
    Write: WritePosture,
    Unit: ApplicationFieldUnit,
{
    fn into_field_parts(
        self,
        _principal_identity: &PrincipalIdentity,
    ) -> (
        ApplicationFieldRef<Schema, Scope, Aspect, Field, Value, Write, EqualityPredicate, Unit>,
        Value,
    ) {
        (self.field, self.value)
    }
}

impl<Schema, Scope, Aspect, Field, Value, Write, Unit>
    ApplicationMutationScopeResolution<Schema, Value>
    for ApplicationMutationPrincipalScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
where
    Field: DeclaredApplicationFieldValue<Value = Value>,
    Value: Clone,
    Write: WritePosture,
    Unit: ApplicationFieldUnit,
{
    fn into_field_parts(
        self,
        principal_identity: &Value,
    ) -> (
        ApplicationFieldRef<Schema, Scope, Aspect, Field, Value, Write, EqualityPredicate, Unit>,
        Value,
    ) {
        (self.field, principal_identity.clone())
    }
}
