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
#[derive(Clone, Copy, Debug)]
pub struct ApplicationDeclaredInputMember {
    member: &'static str,
    optional: bool,
    value_type_id: fn() -> TypeId,
}

impl ApplicationDeclaredInputMember {
    /// Macro-only constructor emitted by `worth_query_structured_value_binding!`
    /// alongside a compile-time field projection check on the bound input struct.
    #[doc(hidden)]
    pub const fn from_binding_macro<Value: 'static>(member: &'static str, optional: bool) -> Self {
        Self {
            member,
            optional,
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
