use worth_foundational::facade::{AspectValue, ScalarAspectType};

use crate::application_schema::{
    ApplicationEncodedScalarValue, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationScalarValueBinding, DeclaredApplicationFieldValue, EqualityPredicate, WritePosture,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryRootPathGuard {
    after_step: usize,
    entity: String,
    aspect: String,
    field: String,
    scalar_family: ScalarAspectType,
    value_type: WorthQueryPortableTypeIdentity,
    expected: AspectValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationQueryRootPathGuardParts {
    pub after_step: usize,
    pub entity: String,
    pub aspect: String,
    pub field: String,
    pub scalar_family: ScalarAspectType,
    pub value_type: WorthQueryPortableTypeIdentity,
    pub expected: AspectValue,
}

impl ApplicationQueryRootPathGuard {
    pub fn from_untrusted_parts(
        parts: WorthQueryPortableApplicationQueryRootPathGuardParts,
    ) -> Self {
        Self {
            after_step: parts.after_step,
            entity: parts.entity,
            aspect: parts.aspect,
            field: parts.field,
            scalar_family: parts.scalar_family,
            value_type: parts.value_type,
            expected: parts.expected,
        }
    }

    pub(super) fn new<Schema, Entity, Aspect, Field, Value, Write, Unit>(
        after_step: usize,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        expected: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        Self {
            after_step,
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
            scalar_family: field.scalar_family(),
            value_type: Field::Binding::IDENTITY,
            expected: expected.into_foundational_value(),
        }
    }

    pub const fn after_step(&self) -> usize {
        self.after_step
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn aspect(&self) -> &str {
        &self.aspect
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    pub const fn value_type(&self) -> &str {
        self.value_type.as_str()
    }

    pub const fn expected(&self) -> &AspectValue {
        &self.expected
    }
}
