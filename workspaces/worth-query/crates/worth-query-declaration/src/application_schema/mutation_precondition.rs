use super::fragment;
use std::marker::PhantomData;

use worth_foundational::facade::AspectValue;

use super::ApplicationEncodedScalarValue;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationMutationPreconditionFamily {
    ExpectedVersion,
    ExpectedFact,
}

impl ApplicationMutationPreconditionFamily {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::ExpectedVersion => "expected-version",
            Self::ExpectedFact => "expected-fact",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationMutationPreconditionTarget {
    family: ApplicationMutationPreconditionFamily,
    entity: String,
    aspect: String,
    field: String,
}

impl ApplicationMutationPreconditionTarget {
    /// Reconstructs authority-free target meaning from an untrusted package.
    ///
    /// This constructor deliberately performs no identifier or schema-closure
    /// validation. The enclosing application-schema readmission boundary owns
    /// those decisions.
    pub fn from_untrusted_fields(
        family: ApplicationMutationPreconditionFamily,
        entity: String,
        aspect: String,
        field: String,
    ) -> Self {
        Self {
            family,
            entity,
            aspect,
            field,
        }
    }

    pub(crate) fn field(
        family: ApplicationMutationPreconditionFamily,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Self {
        Self::from_untrusted_fields(
            family,
            entity.to_owned(),
            aspect.to_owned(),
            field.to_owned(),
        )
    }

    pub const fn family(&self) -> ApplicationMutationPreconditionFamily {
        self.family
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn aspect(&self) -> &str {
        &self.aspect
    }

    pub fn field_name(&self) -> &str {
        &self.field
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedMutationPrecondition {
    target: ApplicationMutationPreconditionTarget,
    expected_value: AspectValue,
}

impl TypedMutationPrecondition {
    pub const fn target(&self) -> &ApplicationMutationPreconditionTarget {
        &self.target
    }

    pub const fn expected_value(&self) -> &AspectValue {
        &self.expected_value
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TypedMutationPreconditions<Schema, Operation, Scope> {
    entries: Vec<TypedMutationPrecondition>,
    _marker: PhantomData<fn() -> (Schema, Operation, Scope)>,
}

impl<Schema, Operation, Scope> Clone for TypedMutationPreconditions<Schema, Operation, Scope> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Schema, Operation, Scope> Default for TypedMutationPreconditions<Schema, Operation, Scope> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Schema, Operation, Scope> TypedMutationPreconditions<Schema, Operation, Scope> {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn expect_version<Aspect, Field, Value>(
        self,
        field: Field,
        expected: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Result<Self, super::ApplicationSchemaDeclarationDenial>
    where
        Field: fragment::RecordSelection<Entity = Scope, Aspect = Aspect, Value = Value>,
    {
        self.push(
            ApplicationMutationPreconditionFamily::ExpectedVersion,
            field,
            expected,
        )
    }

    pub fn expect_fact<Aspect, Field, Value>(
        self,
        field: Field,
        expected: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Result<Self, super::ApplicationSchemaDeclarationDenial>
    where
        Field: fragment::RecordSelection<Entity = Scope, Aspect = Aspect, Value = Value>,
    {
        self.push(
            ApplicationMutationPreconditionFamily::ExpectedFact,
            field,
            expected,
        )
    }

    pub fn entries(&self) -> &[TypedMutationPrecondition] {
        &self.entries
    }

    #[doc(hidden)]
    pub fn into_entries(self) -> Vec<TypedMutationPrecondition> {
        self.entries
    }

    fn push<Aspect, Field, Value>(
        mut self,
        family: ApplicationMutationPreconditionFamily,
        field: Field,
        expected: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Result<Self, super::ApplicationSchemaDeclarationDenial>
    where
        Field: fragment::RecordSelection<Entity = Scope, Aspect = Aspect, Value = Value>,
    {
        field.checked()?;

        let expected_value = expected.into_foundational_value();
        self.entries.push(TypedMutationPrecondition {
            target: ApplicationMutationPreconditionTarget::field(
                family,
                field.entity(),
                field.aspect(),
                field.field(),
            ),
            expected_value,
        });
        Ok(self)
    }
}
