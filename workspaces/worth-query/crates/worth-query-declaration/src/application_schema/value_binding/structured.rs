use std::any::TypeId;

use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::ApplicationValueValidationDenial;

/// Authoritative canonical input-member metadata declared on an
/// [`ApplicationStructuredValueBinding`].
///
/// Carries the canonical serialized wire member name, optionality posture
/// (`Option<T>` vs `T`), and exact inner Rust [`TypeId`] (behind `fn() -> TypeId`
/// so private field types on public action structs never leak into public
/// signatures).
///
/// Exact field-type witnesses reject deref coercions such as `Box<u64>` for
/// `required("field", field: u64)`:
/// ```compile_fail,E0308
/// use worth_query_declaration::worth_query_structured_value_binding;
/// struct BoxedField {
///     field: Box<u64>,
/// }
/// worth_query_structured_value_binding!(BoxedBinding for BoxedField {
///     identity: "example.boxed-field.v1",
///     fields: [required("field", field: u64)],
/// });
/// ```
///
/// Deref coercions are likewise rejected for optional fields (`Box<Option<u64>>`
/// for `optional("opt", opt: u64)`):
/// ```compile_fail,E0308
/// use worth_query_declaration::worth_query_structured_value_binding;
/// struct BoxedOptional {
///     opt: Box<Option<u64>>,
/// }
/// worth_query_structured_value_binding!(BoxedOptBinding for BoxedOptional {
///     identity: "example.boxed-opt.v1",
///     fields: [optional("opt", opt: u64)],
/// });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ApplicationDeclaredInputMember {
    member: &'static str,
    optional: bool,
    value_type_id: fn() -> TypeId,
}

impl ApplicationDeclaredInputMember {
    /// Macro-only constructor for a required declared input member backed by an
    /// exact-type raw-pointer field witness on `Input` (sized `*const T` has no
    /// deref coercions).
    #[doc(hidden)]
    pub const fn required_from_binding_macro<Input: 'static, Value: 'static>(
        member: &'static str,
        _exact_witness: fn(&Input) -> *const Value,
    ) -> Self {
        Self {
            member,
            optional: false,
            value_type_id: TypeId::of::<Value>,
        }
    }

    /// Macro-only constructor for an optional declared input member backed by an
    /// exact-type `*const Option<Value>` field witness on `Input` (sized
    /// `*const Option<T>` has no deref coercions).
    #[doc(hidden)]
    pub const fn optional_from_binding_macro<Input: 'static, Value: 'static>(
        member: &'static str,
        _exact_witness: fn(&Input) -> *const Option<Value>,
    ) -> Self {
        Self {
            member,
            optional: true,
            value_type_id: TypeId::of::<Value>,
        }
    }

    /// The canonical serialized member name on the bound input.
    pub const fn member(self) -> &'static str {
        self.member
    }

    /// Whether this declared member carries `Option<Value>`.
    pub const fn is_optional(self) -> bool {
        self.optional
    }

    /// The exact [`TypeId`] of the inner value type `Value`.
    pub fn value_type_id(self) -> TypeId {
        (self.value_type_id)()
    }
}

/// Entry-owned identity and validation for a representation-neutral value.
///
/// Structured values are decomposed by their query, operation, or effect
/// binding. They are never coerced into one Foundational scalar.
pub trait ApplicationStructuredValueBinding: 'static {
    type Value: 'static;

    const IDENTITY_NAME: &'static str;
    const IDENTITY: WorthQueryPortableTypeIdentity =
        WorthQueryPortableTypeIdentity::declared(Self::IDENTITY_NAME);

    /// Retain the input's canonical serialization from its one identity pass.
    /// Native body input selectors require this capture; it grants no authority.
    const RETAIN_CANONICAL_INPUT: bool = false;

    /// Authoritative declared serialized input members for typed body authoring.
    /// Populated by `worth_query_structured_value_binding!(..., fields: [...])`
    /// and `#[derive(WorthQueryAction)]`; empty when no closed field shape is
    /// declared.
    const DECLARED_INPUT_FIELDS: &'static [ApplicationDeclaredInputMember] = &[];

    fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial>;
}

crate::worth_query_structured_value_binding!(pub UnitApplicationValueBinding for () {
    identity: "worth.rust.unit"
});
