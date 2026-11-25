//! Compile-time-typed input-field references for authored operation bodies.
//!
//! Each reference carries its issuing [`ApplicationStructuredValueBinding`]
//! identity alongside its canonical member name, verified both at selector
//! construction ([`ApplicationInputField::for_binding`],
//! [`ApplicationOptionalInputField::for_binding`]) and when applied to an
//! operation body via [`ApplicationOperationBodyBuilder`] /
//! [`super::TypedApplicationOperationBodyBuilder`] against
//! `Operation::InputBinding`. No free-string constructor exists.

use std::any::TypeId;
use std::fmt;
use std::marker::PhantomData;

use crate::application_query::ApplicationQueryRootPath;
use crate::application_schema::{
    fragment::RecordSelection, ApplicationIdentityScalarValueBinding,
    ApplicationOperationMarkerIdentity, ApplicationRelationRef, ApplicationStructuredValueBinding,
    OperationLinks, OperationReads, OperationUnlinks,
};

use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
    ApplicationOperationBodyCreationBuilder,
};

/// Verifies that `Binding` declares exactly one canonical input member matching
/// `(member, Value, optional)`.
pub(super) fn verify_declared_input_member<Binding, Value>(
    member: &str,
    optional: bool,
) -> Result<(), Denial>
where
    Binding: ApplicationStructuredValueBinding,
    Value: 'static,
{
    if member.is_empty() || member.trim() != member {
        return Err(Denial::InvalidInputMember);
    }
    let expected_type = TypeId::of::<Value>();
    let mut matched = false;
    for declared in Binding::DECLARED_INPUT_FIELDS {
        if declared.member() != member {
            continue;
        }
        if matched
            || declared.is_optional() != optional
            || declared.value_type_id() != expected_type
        {
            return Err(Denial::InvalidInputMember);
        }
        matched = true;
    }
    if matched {
        Ok(())
    } else {
        Err(Denial::InvalidInputMember)
    }
}

/// Verifies that a typed selector was issued by `OperationBinding` itself and
/// matches its authoritative declared field inventory.
fn verify_typed_input_selector<OperationBinding, Value>(
    member: &str,
    issuing_binding: fn() -> TypeId,
    optional: bool,
) -> Result<(), Denial>
where
    OperationBinding: ApplicationStructuredValueBinding,
    Value: 'static,
{
    if issuing_binding() != TypeId::of::<OperationBinding>() {
        return Err(Denial::InvalidInputMember);
    }
    verify_declared_input_member::<OperationBinding, Value>(member, optional)
}

/// A compile-time-typed reference to one required scalar field on `Input`.
///
/// Carries no runtime authority: only the canonical input member name and
/// issuing `InputBinding` identity verified against the bound input's
/// authoritative declared field inventory and checked against the target
/// operation's `InputBinding` and record column.
///
/// No free-string constructor exists:
/// ```compile_fail
/// use worth_query_declaration::facade::application_schema::ApplicationInputField;
/// let _ = ApplicationInputField::<(), String>::new("fabricated");
/// ```
///
/// A non-binding type cannot construct a typed input field:
/// ```compile_fail
/// use worth_query_declaration::facade::application_schema::ApplicationInputField;
/// let _ = ApplicationInputField::<(), String>::for_binding::<()>("fabricated");
/// ```
pub struct ApplicationInputField<Input, Value> {
    member: &'static str,
    binding_type_id: fn() -> TypeId,
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
        self.member == other.member && (self.binding_type_id)() == (other.binding_type_id)()
    }
}

impl<Input, Value> Eq for ApplicationInputField<Input, Value> {}

impl<Input, Value> ApplicationInputField<Input, Value> {
    /// The canonical serialized member name on `Input`.
    pub const fn member(self) -> &'static str {
        self.member
    }
}

impl<Input: 'static, Value: 'static> ApplicationInputField<Input, Value> {
    /// Resolves a required typed input field verified against `Binding`'s
    /// authoritative declared canonical member shape.
    pub fn for_binding<Binding>(member: &'static str) -> Result<Self, Denial>
    where
        Binding: ApplicationStructuredValueBinding<Value = Input>,
    {
        verify_declared_input_member::<Binding, Value>(member, false)?;
        Ok(Self {
            member,
            binding_type_id: TypeId::of::<Binding>,
            marker: PhantomData,
        })
    }
}

/// A compile-time-typed reference to one `Option<Value>` field on `Input`.
///
/// Carries no runtime authority: only the canonical input member name and
/// issuing `InputBinding` identity verified against the bound input's
/// authoritative declared field inventory and checked against the target
/// operation's `InputBinding` and optional record column.
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_schema::ApplicationOptionalInputField;
/// let _ = ApplicationOptionalInputField::<(), String>::new("fabricated");
/// ```
pub struct ApplicationOptionalInputField<Input, Value> {
    member: &'static str,
    binding_type_id: fn() -> TypeId,
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
        self.member == other.member && (self.binding_type_id)() == (other.binding_type_id)()
    }
}

impl<Input, Value> Eq for ApplicationOptionalInputField<Input, Value> {}

impl<Input, Value> ApplicationOptionalInputField<Input, Value> {
    /// The canonical serialized member name on `Input`.
    pub const fn member(self) -> &'static str {
        self.member
    }
}

impl<Input: 'static, Value: 'static> ApplicationOptionalInputField<Input, Value> {
    /// Resolves an optional typed input field verified against `Binding`'s
    /// authoritative declared canonical member shape.
    pub fn for_binding<Binding>(member: &'static str) -> Result<Self, Denial>
    where
        Binding: ApplicationStructuredValueBinding<Value = Input>,
    {
        verify_declared_input_member::<Binding, Value>(member, true)?;
        Ok(Self {
            member,
            binding_type_id: TypeId::of::<Binding>,
            marker: PhantomData,
        })
    }
}

impl<Schema, Operation, Input> ApplicationOperationBodyBuilder<Schema, Operation, Input>
where
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
{
    /// Requires a present scope field equal to one typed required input field.
    pub fn expect_typed_input<Field: RecordSelection>(
        self,
        field: Field,
        input: ApplicationInputField<Input, Field::Value>,
    ) -> Result<Self, Denial> {
        verify_typed_input_selector::<Operation::InputBinding, Field::Value>(
            input.member(),
            input.binding_type_id,
            false,
        )?;
        self.expect_input(field, input.member())
    }

    /// Assigns a scope scalar from one typed required input field.
    pub fn assign_typed_input<Field: RecordSelection>(
        self,
        field: Field,
        input: ApplicationInputField<Input, Field::Value>,
    ) -> Result<Self, Denial> {
        verify_typed_input_selector::<Operation::InputBinding, Field::Value>(
            input.member(),
            input.binding_type_id,
            false,
        )?;
        self.assign_input(field, input.member())
    }

    /// Sets an optional scope scalar from one typed required input field.
    pub fn set_typed_optional_input<Field: RecordSelection>(
        self,
        field: Field,
        input: ApplicationInputField<Input, Field::Value>,
    ) -> Result<Self, Denial> {
        verify_typed_input_selector::<Operation::InputBinding, Field::Value>(
            input.member(),
            input.binding_type_id,
            false,
        )?;
        self.set_optional_input(field, input.member())
    }

    /// Replaces an optional scope scalar from one typed `Option<T>` input field.
    pub fn assign_typed_optional_input<Field: RecordSelection>(
        self,
        field: Field,
        input: ApplicationOptionalInputField<Input, Field::Value>,
    ) -> Result<Self, Denial> {
        verify_typed_input_selector::<Operation::InputBinding, Field::Value>(
            input.member(),
            input.binding_type_id,
            true,
        )?;
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
        verify_typed_input_selector::<Operation::InputBinding, F::Value>(
            input.member(),
            input.binding_type_id,
            false,
        )?;
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
        verify_typed_input_selector::<Operation::InputBinding, F::Value>(
            input.member(),
            input.binding_type_id,
            true,
        )?;
        self.replace_relation_from_input(relation, identity, input.member(), key)
    }

    /// Links or unlinks one exact edge from a typed required identity and a typed bool input.
    pub fn toggle_relation_from_typed_input<R, Root, Selected, F>(
        self,
        relation: ApplicationRelationRef<Schema, R, Root, Selected>,
        identity: F,
        target: ApplicationInputField<Input, F::Value>,
        linked: ApplicationInputField<Input, bool>,
        key: &str,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<Operation>,
        Selected: OperationReads<Operation>,
        R: OperationReads<Operation> + OperationLinks<Operation> + OperationUnlinks<Operation>,
        F: RecordSelection<Entity = Selected>,
        F::Binding: ApplicationIdentityScalarValueBinding,
    {
        verify_typed_input_selector::<Operation::InputBinding, F::Value>(
            target.member(),
            target.binding_type_id,
            false,
        )?;
        verify_typed_input_selector::<Operation::InputBinding, bool>(
            linked.member(),
            linked.binding_type_id,
            false,
        )?;
        self.toggle_relation_from_input(relation, identity, target.member(), linked.member(), key)
    }
}

impl<Schema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
where
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
{
    /// Initializes a created field from one typed required input field.
    pub fn initialize_typed_input<Selection: RecordSelection<Entity = Entity>>(
        self,
        field: Selection,
        input: ApplicationInputField<Input, Selection::Value>,
    ) -> Result<Self, Denial> {
        verify_typed_input_selector::<Operation::InputBinding, Selection::Value>(
            input.member(),
            input.binding_type_id,
            false,
        )?;
        self.initialize_input(field, input.member())
    }

    /// Initializes an optional created field from one typed `Option<T>` input field.
    pub fn initialize_typed_optional_input<Selection: RecordSelection<Entity = Entity>>(
        self,
        field: Selection,
        input: ApplicationOptionalInputField<Input, Selection::Value>,
    ) -> Result<Self, Denial> {
        verify_typed_input_selector::<Operation::InputBinding, Selection::Value>(
            input.member(),
            input.binding_type_id,
            true,
        )?;
        self.initialize_optional_input(field, input.member())
    }
}
