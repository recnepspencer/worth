use super::{
    ApplicationQueryFieldScope, ApplicationQueryPrincipalScope, ApplicationQueryScopeBinding,
};
use crate::application_schema::{
    ApplicationFieldRef, ApplicationFieldUnit, DeclaredApplicationFieldValue, EqualityPredicate,
    WritePosture,
};

/// Resolves a declared scope strategy using the current borrowed request.
pub trait ApplicationQueryScopeResolution<Schema, PrincipalIdentity>:
    ApplicationQueryScopeBinding<Schema>
{
    fn into_field_parts(
        self,
        principal_identity: &PrincipalIdentity,
    ) -> (
        ApplicationFieldRef<
            Schema,
            Self::Scope,
            Self::Aspect,
            Self::Field,
            Self::Value,
            Self::Write,
            EqualityPredicate,
            Self::Unit,
        >,
        Self::Value,
    );
}

impl<Schema, Scope, Aspect, Field, Value, Write, Unit, PrincipalIdentity>
    ApplicationQueryScopeResolution<Schema, PrincipalIdentity>
    for ApplicationQueryFieldScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
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
    ApplicationQueryScopeResolution<Schema, Value>
    for ApplicationQueryPrincipalScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
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
