use crate::application_schema::{
    ApplicationFieldRef, ApplicationFieldUnit, DeclaredApplicationFieldValue, EqualityPredicate,
    WritePosture,
};

use super::ApplicationQueryScopeResolutionMode;

/// Declaration-only description of how an intent locates its application scope.
pub trait ApplicationQueryScopeBinding<Schema> {
    type Scope;
    type Aspect;
    type Field: DeclaredApplicationFieldValue<Value = Self::Value>;
    type Value;
    type Write: WritePosture;
    type Unit: ApplicationFieldUnit;

    const RESOLUTION_MODE: ApplicationQueryScopeResolutionMode;
}

pub struct ApplicationQueryFieldScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
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
    ApplicationQueryFieldScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
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

impl<Schema, Scope, Aspect, Field, Value, Write, Unit> ApplicationQueryScopeBinding<Schema>
    for ApplicationQueryFieldScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
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
    const RESOLUTION_MODE: ApplicationQueryScopeResolutionMode =
        ApplicationQueryScopeResolutionMode::InputField;
}

pub struct ApplicationQueryPrincipalScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
where
    Field: DeclaredApplicationFieldValue<Value = Value>,
    Write: WritePosture,
    Unit: ApplicationFieldUnit,
{
    pub(super) field:
        ApplicationFieldRef<Schema, Scope, Aspect, Field, Value, Write, EqualityPredicate, Unit>,
}

impl<Schema, Scope, Aspect, Field, Value, Write, Unit>
    ApplicationQueryPrincipalScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
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

impl<Schema, Scope, Aspect, Field, Value, Write, Unit> ApplicationQueryScopeBinding<Schema>
    for ApplicationQueryPrincipalScope<Schema, Scope, Aspect, Field, Value, Write, Unit>
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
    const RESOLUTION_MODE: ApplicationQueryScopeResolutionMode =
        ApplicationQueryScopeResolutionMode::PrincipalIdentity;
}
