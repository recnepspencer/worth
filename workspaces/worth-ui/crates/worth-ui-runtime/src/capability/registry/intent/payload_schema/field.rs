use core::marker::PhantomData;
use std::sync::Arc;

pub const UI_INTENT_PAYLOAD_FIELD_LIMIT: usize = 64;
pub const UI_INTENT_PAYLOAD_TEXT_BYTE_LIMIT: usize = 65_536;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiIntentPayloadFieldKind {
    Text,
    Boolean,
    Unsigned64,
    Selection,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiIntentPayloadFieldDescriptor {
    slot: u8,
    stable_name: &'static str,
    kind: UiIntentPayloadFieldKind,
    byte_budget: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiIntentPayloadFieldSet {
    fields: &'static [UiIntentPayloadFieldDescriptor],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentPayloadSchemaViolation {
    TooManyFields { observed: usize, maximum: usize },
    NonCanonicalSlot { expected: u8, observed: u8 },
    InvalidFieldName { slot: u8 },
    DuplicateFieldName { first: u8, duplicate: u8 },
    InvalidTextByteBudget { slot: u8, observed: usize },
    UnexpectedByteBudget { slot: u8, observed: usize },
}

pub struct UiIntentPayloadField<P: super::UiIntentPayload, K: UiIntentPayloadValueKind> {
    descriptor: UiIntentPayloadFieldDescriptor,
    payload: PhantomData<fn() -> P>,
    kind: PhantomData<fn() -> K>,
}

pub enum UiIntentText {}
pub enum UiIntentBoolean {}
pub enum UiIntentUnsigned64 {}
pub enum UiIntentSelection {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiIntentSelectionValue {
    option: worth_ui_query_binding::UiProjectionOptionReference,
}

pub trait UiIntentPayloadValueKind: private::Sealed + Send + Sync + 'static {
    type Value: Send + 'static;

    const KIND: UiIntentPayloadFieldKind;

    fn take(
        value: super::UiIntentProjectedValue,
    ) -> Result<Self::Value, super::UiIntentPayloadProjectionViolation>;
}

impl UiIntentPayloadFieldSet {
    pub const EMPTY: Self = Self::new(&[]);

    pub const fn new(fields: &'static [UiIntentPayloadFieldDescriptor]) -> Self {
        Self { fields }
    }

    pub const fn fields(self) -> &'static [UiIntentPayloadFieldDescriptor] {
        self.fields
    }

    pub const fn len(self) -> usize {
        self.fields.len()
    }

    pub const fn is_empty(self) -> bool {
        self.fields.is_empty()
    }

    pub(crate) fn validate(self) -> Result<(), UiIntentPayloadSchemaViolation> {
        if self.fields.len() > UI_INTENT_PAYLOAD_FIELD_LIMIT {
            return Err(UiIntentPayloadSchemaViolation::TooManyFields {
                observed: self.fields.len(),
                maximum: UI_INTENT_PAYLOAD_FIELD_LIMIT,
            });
        }
        for (index, field) in self.fields.iter().enumerate() {
            let expected = u8::try_from(index).expect("the field limit, checked above, fits u8");
            if field.slot != expected {
                return Err(UiIntentPayloadSchemaViolation::NonCanonicalSlot {
                    expected,
                    observed: field.slot,
                });
            }
            field.validate()?;
            if let Some(first) = self.fields[..index]
                .iter()
                .find(|candidate| candidate.stable_name == field.stable_name)
            {
                return Err(UiIntentPayloadSchemaViolation::DuplicateFieldName {
                    first: first.slot,
                    duplicate: field.slot,
                });
            }
        }
        Ok(())
    }
}

impl UiIntentPayloadFieldDescriptor {
    pub const fn slot(self) -> u8 {
        self.slot
    }

    pub const fn stable_name(self) -> &'static str {
        self.stable_name
    }

    pub const fn kind(self) -> UiIntentPayloadFieldKind {
        self.kind
    }

    pub const fn byte_budget(self) -> usize {
        self.byte_budget
    }

    fn validate(self) -> Result<(), UiIntentPayloadSchemaViolation> {
        if !valid_field_name(self.stable_name) {
            return Err(UiIntentPayloadSchemaViolation::InvalidFieldName { slot: self.slot });
        }
        match self.kind {
            UiIntentPayloadFieldKind::Text
                if self.byte_budget == 0
                    || self.byte_budget > UI_INTENT_PAYLOAD_TEXT_BYTE_LIMIT =>
            {
                Err(UiIntentPayloadSchemaViolation::InvalidTextByteBudget {
                    slot: self.slot,
                    observed: self.byte_budget,
                })
            }
            UiIntentPayloadFieldKind::Boolean
            | UiIntentPayloadFieldKind::Unsigned64
            | UiIntentPayloadFieldKind::Selection
                if self.byte_budget != 0 =>
            {
                Err(UiIntentPayloadSchemaViolation::UnexpectedByteBudget {
                    slot: self.slot,
                    observed: self.byte_budget,
                })
            }
            UiIntentPayloadFieldKind::Text
            | UiIntentPayloadFieldKind::Boolean
            | UiIntentPayloadFieldKind::Unsigned64
            | UiIntentPayloadFieldKind::Selection => Ok(()),
        }
    }
}

impl UiIntentPayloadFieldKind {
    pub(crate) const fn digest_basis(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Boolean => "boolean",
            Self::Unsigned64 => "unsigned-64",
            Self::Selection => "selection",
        }
    }
}

impl<P: super::UiIntentPayload> UiIntentPayloadField<P, UiIntentText> {
    pub const fn text(slot: u8, stable_name: &'static str, byte_budget: usize) -> Self {
        Self::new(
            slot,
            stable_name,
            UiIntentPayloadFieldKind::Text,
            byte_budget,
        )
    }
}

impl<P: super::UiIntentPayload> UiIntentPayloadField<P, UiIntentBoolean> {
    pub const fn boolean(slot: u8, stable_name: &'static str) -> Self {
        Self::new(slot, stable_name, UiIntentPayloadFieldKind::Boolean, 0)
    }
}

impl<P: super::UiIntentPayload> UiIntentPayloadField<P, UiIntentUnsigned64> {
    pub const fn unsigned64(slot: u8, stable_name: &'static str) -> Self {
        Self::new(slot, stable_name, UiIntentPayloadFieldKind::Unsigned64, 0)
    }
}

impl<P: super::UiIntentPayload> UiIntentPayloadField<P, UiIntentSelection> {
    pub const fn selection(slot: u8, stable_name: &'static str) -> Self {
        Self::new(slot, stable_name, UiIntentPayloadFieldKind::Selection, 0)
    }
}

impl<P: super::UiIntentPayload, K: UiIntentPayloadValueKind> UiIntentPayloadField<P, K> {
    const fn new(
        slot: u8,
        stable_name: &'static str,
        kind: UiIntentPayloadFieldKind,
        byte_budget: usize,
    ) -> Self {
        Self {
            descriptor: UiIntentPayloadFieldDescriptor {
                slot,
                stable_name,
                kind,
                byte_budget,
            },
            payload: PhantomData,
            kind: PhantomData,
        }
    }

    pub const fn descriptor(self) -> UiIntentPayloadFieldDescriptor {
        self.descriptor
    }
}

impl<P: super::UiIntentPayload, K: UiIntentPayloadValueKind> Clone for UiIntentPayloadField<P, K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: super::UiIntentPayload, K: UiIntentPayloadValueKind> Copy for UiIntentPayloadField<P, K> {}

impl UiIntentSelectionValue {
    pub(crate) fn admitted(option: worth_ui_query_binding::UiProjectionOptionReference) -> Self {
        Self { option }
    }

    pub fn option(&self) -> &worth_ui_query_binding::UiProjectionOptionReference {
        &self.option
    }
}

impl UiIntentPayloadValueKind for UiIntentText {
    type Value = Arc<str>;

    const KIND: UiIntentPayloadFieldKind = UiIntentPayloadFieldKind::Text;

    fn take(
        value: super::UiIntentProjectedValue,
    ) -> Result<Self::Value, super::UiIntentPayloadProjectionViolation> {
        value
            .into_text()
            .ok_or(super::UiIntentPayloadProjectionViolation::ValueKindMismatch)
    }
}

impl UiIntentPayloadValueKind for UiIntentBoolean {
    type Value = bool;

    const KIND: UiIntentPayloadFieldKind = UiIntentPayloadFieldKind::Boolean;

    fn take(
        value: super::UiIntentProjectedValue,
    ) -> Result<Self::Value, super::UiIntentPayloadProjectionViolation> {
        value
            .into_boolean()
            .ok_or(super::UiIntentPayloadProjectionViolation::ValueKindMismatch)
    }
}

impl UiIntentPayloadValueKind for UiIntentUnsigned64 {
    type Value = u64;

    const KIND: UiIntentPayloadFieldKind = UiIntentPayloadFieldKind::Unsigned64;

    fn take(
        value: super::UiIntentProjectedValue,
    ) -> Result<Self::Value, super::UiIntentPayloadProjectionViolation> {
        value
            .into_unsigned64()
            .ok_or(super::UiIntentPayloadProjectionViolation::ValueKindMismatch)
    }
}

impl UiIntentPayloadValueKind for UiIntentSelection {
    type Value = UiIntentSelectionValue;

    const KIND: UiIntentPayloadFieldKind = UiIntentPayloadFieldKind::Selection;

    fn take(
        value: super::UiIntentProjectedValue,
    ) -> Result<Self::Value, super::UiIntentPayloadProjectionViolation> {
        value
            .into_selection()
            .ok_or(super::UiIntentPayloadProjectionViolation::ValueKindMismatch)
    }
}

const fn valid_field_name(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if !(byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')) {
            return false;
        }
        index += 1;
    }
    true
}

mod private {
    pub trait Sealed {}

    impl Sealed for super::UiIntentText {}
    impl Sealed for super::UiIntentBoolean {}
    impl Sealed for super::UiIntentUnsigned64 {}
    impl Sealed for super::UiIntentSelection {}
}
