use crate::application_schema::{
    ApplicationFieldRef, ApplicationFieldUnit, DeclaredApplicationFieldValue, EqualityPredicate,
    WritePosture,
};

use super::ApplicationMutationScopeResolutionMode;

/// Declaration-only description of how a mutation intent locates its scope.
pub trait ApplicationMutationScopeBinding<Schema> {
    type Scope;
    type Aspect;
    type Field: DeclaredApplicationFieldValue<Value = Self::Value>;
    type Value;
    type Write: WritePosture;
    type Unit: ApplicationFieldUnit;

    const RESOLUTION_MODE: ApplicationMutationScopeResolutionMode;
}

pub struct ApplicationMutationFieldScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
where
    Field: DeclaredApplicationFieldValue<Value = Value>,
    Write: WritePosture,
    Unit: ApplicationFieldUnit,
{
    pub(super) field:
        ApplicationFieldRef<Schema, Scope, Aspect, Field, Value, Write, EqualityPredicate, Unit>,
    pub(super) value: Value,
}

impl<Schema, Scope, Aspect, Field, Value, Write, Unit>
    ApplicationMutationFieldScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
where
    Field: DeclaredApplicationFieldValue<Value = Value>,
    Write: WritePosture,
    Unit: ApplicationFieldUnit,
{
    pub const fn new(
        field: ApplicationFieldRef<
            Schema,
            Scope,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
    ) -> Self {
        Self { field, value }
    }
}

impl<Schema, Scope, Aspect, Field, Value, Write, Unit> ApplicationMutationScopeBinding<Schema>
    for ApplicationMutationFieldScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
where
    Field: DeclaredApplicationFieldValue<Value = Value>,
    Write: WritePosture,
    Unit: ApplicationFieldUnit,
{
    type Scope = Scope;
    type Aspect = Aspect;
    type Field = Field;
    type Value = Value;
    type Write = Write;
    type Unit = Unit;
    const RESOLUTION_MODE: ApplicationMutationScopeResolutionMode =
        ApplicationMutationScopeResolutionMode::InputField;
}

pub struct ApplicationMutationPrincipalScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
where
    Field: DeclaredApplicationFieldValue<Value = Value>,
    Write: WritePosture,
    Unit: ApplicationFieldUnit,
{
    pub(super) field:
        ApplicationFieldRef<Schema, Scope, Aspect, Field, Value, Write, EqualityPredicate, Unit>,
}

impl<Schema, Scope, Aspect, Field, Value, Write, Unit>
    ApplicationMutationPrincipalScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
where
    Field: DeclaredApplicationFieldValue<Value = Value>,
    Write: WritePosture,
    Unit: ApplicationFieldUnit,
{
    pub const fn new(
        field: ApplicationFieldRef<
            Schema,
            Scope,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
    ) -> Self {
        Self { field }
    }
}

impl<Schema, Scope, Aspect, Field, Value, Write, Unit> ApplicationMutationScopeBinding<Schema>
    for ApplicationMutationPrincipalScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
where
    Field: DeclaredApplicationFieldValue<Value = Value>,
    Write: WritePosture,
    Unit: ApplicationFieldUnit,
{
    type Scope = Scope;
    type Aspect = Aspect;
    type Field = Field;
    type Value = Value;
    type Write = Write;
    type Unit = Unit;
    const RESOLUTION_MODE: ApplicationMutationScopeResolutionMode =
        ApplicationMutationScopeResolutionMode::PrincipalIdentity;
}
