//! Compile-time-typed input-field references for authored operation bodies.
//!
//! Each reference is constructed exclusively from a declared field slot on
//! `Input` (`ApplicationInputFields` + `ApplicationInputFieldSlot<INDEX>`,
//! emitted by `#[derive(WorthQueryAction)]`), carrying the owning `Input` type,
//! scalar `Value` type, and canonical serialized member name without exposing
//! any free-string constructor. Methods on [`ApplicationOperationBodyBuilder`]
//! and [`ApplicationOperationBodyCreationBuilder`] delegate directly to the
//! underlying member-name methods so typed and hand-written bodies produce
//! identical canonical programs.

use std::fmt;
use std::marker::PhantomData;

use crate::application_query::ApplicationQueryRootPath;
use crate::application_schema::{
    fragment::RecordSelection, ApplicationIdentityScalarValueBinding, ApplicationRelationRef,
    OperationLinks, OperationReads, OperationUnlinks,
};

use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
    ApplicationOperationBodyCreationBuilder,
};

/// Canonical metadata for one declared serialized field on an action `Input`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationDeclaredInputFieldSpec {
    member: &'static str,
    optional: bool,
}

impl ApplicationDeclaredInputFieldSpec {
    /// Macro-only constructor for a required field spec backed by a compile-time
    /// field projection witness on `Input`.
    #[doc(hidden)]
    pub const fn required<Input, Value: 'static>(
        member: &'static str,
        _witness: fn(&Input) -> &Value,
    ) -> Self {
        Self {
            member,
            optional: false,
        }
    }

    /// Macro-only constructor for an optional field spec backed by a compile-time
    /// field projection witness on `Input`.
    #[doc(hidden)]
    pub const fn optional<Input, Value: 'static>(
        member: &'static str,
        _witness: fn(&Input) -> &Option<Value>,
    ) -> Self {
        Self {
            member,
            optional: true,
        }
    }

    /// The canonical serialized member name on `Input`.
    pub const fn member(self) -> &'static str {
        self.member
    }

    /// Whether this declared field carries `Option<Value>`.
    pub const fn is_optional(self) -> bool {
        self.optional
    }
}

/// Complete declared serialized input-field inventory owned by `Input`.
///
/// Implemented by `#[derive(WorthQueryAction)]`.
pub trait ApplicationInputFields: 'static {
    const FIELDS: &'static [ApplicationDeclaredInputFieldSpec];
}

/// Compile-time binding of declared field slot `INDEX` on `Input` to its exact
/// scalar `Value` type and optionality posture.
pub trait ApplicationInputFieldSlot<const INDEX: usize>: ApplicationInputFields {
    type Value: 'static;
    const OPTIONAL: bool;
}

/// A compile-time-typed reference to one required scalar field on `Input`.
///
/// Carries no runtime authority: only the canonical input member name verified
/// against `Input`'s declared field slot and checked against the target
/// operation and record column.
///
/// An undeclared type cannot fabricate a typed input field:
/// ```compile_fail
/// use worth_query_declaration::facade::application_schema::ApplicationInputField;
/// let _ = ApplicationInputField::<(), String>::from_slot::<0>();
/// ```
///
/// No free-string constructor exists:
/// ```compile_fail
/// use worth_query_declaration::facade::application_schema::ApplicationInputField;
/// let _ = ApplicationInputField::<(), String>::new("fabricated");
/// ```
pub struct ApplicationInputField<Input, Value> {
    member: &'static str,
    marker: PhantomData<fn(&Input) -> Value>,
}

impl<Input, Value> Copy for ApplicationInputField<Input, Value> {}

impl<Input, Value> Clone for ApplicationInputField<Input, Value> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Input, Value> fmt::Debug for ApplicationInputField<Input, Value> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApplicationInputField")
            .field("member", &self.member)
            .finish()
    }
}

impl<Input, Value> PartialEq for ApplicationInputField<Input, Value> {
    fn eq(&self, other: &Self) -> bool {
        self.member == other.member
    }
}

impl<Input, Value> Eq for ApplicationInputField<Input, Value> {}

impl<Input, Value: 'static> ApplicationInputField<Input, Value> {
    /// Resolves the typed field reference for declared required slot `INDEX` on
    /// `Input`. Fails at compile time if `Input` does not declare slot `INDEX`
    /// with exact type `Value` and required (`!OPTIONAL`) posture.
    pub const fn from_slot<const INDEX: usize>() -> Self
    where
        Input: ApplicationInputFieldSlot<INDEX, Value = Value>,
    {
        const {
            assert!(
                INDEX < Input::FIELDS.len(),
                "declared input field slot out of bounds"
            );
            assert!(
                !Input::OPTIONAL,
                "declared input field slot is optional; use ApplicationOptionalInputField"
            );
            assert!(
                !Input::FIELDS[INDEX].is_optional(),
                "declared input field spec is optional"
            );
        }
        Self {
            member: Input::FIELDS[INDEX].member(),
            marker: PhantomData,
        }
    }

    /// The canonical serialized member name on `Input`.
    pub const fn member(self) -> &'static str {
        self.member
    }
}

/// A compile-time-typed reference to one `Option<Value>` field on `Input`.
///
/// Carries no runtime authority: only the canonical input member name verified
/// against `Input`'s declared field slot and checked against the target
/// operation and optional record column.
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_schema::ApplicationOptionalInputField;
/// let _ = ApplicationOptionalInputField::<(), String>::from_slot::<0>();
/// ```
pub struct ApplicationOptionalInputField<Input, Value> {
    member: &'static str,
    marker: PhantomData<fn(&Input) -> Option<Value>>,
}

impl<Input, Value> Copy for ApplicationOptionalInputField<Input, Value> {}

impl<Input, Value> Clone for ApplicationOptionalInputField<Input, Value> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Input, Value> fmt::Debug for ApplicationOptionalInputField<Input, Value> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApplicationOptionalInputField")
            .field("member", &self.member)
            .finish()
    }
}

impl<Input, Value> PartialEq for ApplicationOptionalInputField<Input, Value> {
    fn eq(&self, other: &Self) -> bool {
        self.member == other.member
    }
}

impl<Input, Value> Eq for ApplicationOptionalInputField<Input, Value> {}

impl<Input, Value: 'static> ApplicationOptionalInputField<Input, Value> {
    /// Resolves the typed field reference for declared optional slot `INDEX` on
    /// `Input`. Fails at compile time if `Input` does not declare slot `INDEX`
    /// with exact inner type `Value` and optional (`OPTIONAL`) posture.
    pub const fn from_slot<const INDEX: usize>() -> Self
    where
        Input: ApplicationInputFieldSlot<INDEX, Value = Value>,
    {
        const {
            assert!(
                INDEX < Input::FIELDS.len(),
                "declared input field slot out of bounds"
            );
            assert!(
                Input::OPTIONAL,
                "declared input field slot is required; use ApplicationInputField"
            );
            assert!(
                Input::FIELDS[INDEX].is_optional(),
                "declared input field spec is required"
            );
        }
        Self {
            member: Input::FIELDS[INDEX].member(),
            marker: PhantomData,
        }
    }

    /// The canonical serialized member name on `Input`.
    pub const fn member(self) -> &'static str {
        self.member
    }
}

impl<Schema, Operation, Input> ApplicationOperationBodyBuilder<Schema, Operation, Input> {
    /// Requires a present scope field equal to one typed required input field.
    pub fn expect_typed_input<Field: RecordSelection>(
        self,
        field: Field,
        input: ApplicationInputField<Input, Field::Value>,
    ) -> Result<Self, Denial> {
        self.expect_input(field, input.member())
    }

    /// Assigns a scope scalar from one typed required input field.
    pub fn assign_typed_input<Field: RecordSelection>(
        self,
        field: Field,
        input: ApplicationInputField<Input, Field::Value>,
    ) -> Result<Self, Denial> {
        self.assign_input(field, input.member())
    }

    /// Sets an optional scope scalar from one typed required input field.
    pub fn set_typed_optional_input<Field: RecordSelection>(
        self,
        field: Field,
        input: ApplicationInputField<Input, Field::Value>,
    ) -> Result<Self, Denial> {
        self.set_optional_input(field, input.member())
    }

    /// Replaces an optional scope scalar from one typed `Option<T>` input field.
    pub fn assign_typed_optional_input<Field: RecordSelection>(
        self,
        field: Field,
        input: ApplicationOptionalInputField<Input, Field::Value>,
    ) -> Result<Self, Denial> {
        self.assign_optional_input(field, input.member())
    }

    /// Requires a present terminal field along `path` equal to one typed input field.
    pub fn expect_typed_input_from_path<Root, Selected, F>(
        self,
        path: ApplicationQueryRootPath<Schema, Root, Selected>,
        field: F,
        input: ApplicationInputField<Input, F::Value>,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<Operation>,
        Selected: OperationReads<Operation>,
        F: RecordSelection<Entity = Selected>,
    {
        self.expect_input_from_path(path, field, input.member())
    }

    /// Replaces a nullable at-most-one outgoing reference from a typed `Option<T>` input field.
    pub fn replace_relation_from_typed_input<R, Root, Selected, F>(
        self,
        relation: ApplicationRelationRef<Schema, R, Root, Selected>,
        identity: F,
        input: ApplicationOptionalInputField<Input, F::Value>,
        key: &str,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<Operation>,
        Selected: OperationReads<Operation>,
        R: OperationReads<Operation> + OperationLinks<Operation> + OperationUnlinks<Operation>,
        F: RecordSelection<Entity = Selected>,
        F::Binding: ApplicationIdentityScalarValueBinding,
    {
        self.replace_relation_from_input(relation, identity, input.member(), key)
    }
}

impl<Schema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    /// Initializes a created field from one typed required input field.
    pub fn initialize_typed_input<Selection: RecordSelection<Entity = Entity>>(
        self,
        field: Selection,
        input: ApplicationInputField<Input, Selection::Value>,
    ) -> Result<Self, Denial> {
        self.initialize_input(field, input.member())
    }

    /// Initializes an optional created field from one typed `Option<T>` input field.
    pub fn initialize_typed_optional_input<Selection: RecordSelection<Entity = Entity>>(
        self,
        field: Selection,
        input: ApplicationOptionalInputField<Input, Selection::Value>,
    ) -> Result<Self, Denial> {
        self.initialize_optional_input(field, input.member())
    }
}
