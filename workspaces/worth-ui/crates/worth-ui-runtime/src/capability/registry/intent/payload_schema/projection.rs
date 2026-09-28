use core::marker::PhantomData;
use std::sync::Arc;

use super::{
    UiIntentPayload, UiIntentPayloadField, UiIntentPayloadFieldKind, UiIntentPayloadValueKind,
    UiIntentSelectionValue,
};

pub struct UiIntentProjectedValue {
    value: UiIntentProjectedValueInner,
}

enum UiIntentProjectedValueInner {
    Text(Arc<str>),
    Boolean(bool),
    Unsigned64(u64),
    Selection(UiIntentSelectionValue),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentPayloadProjectionViolation {
    FieldOutsideSchema { slot: u8 },
    FieldHandleMismatch { slot: u8 },
    FieldAlreadyConsumed { slot: u8 },
    ValueKindMismatch,
    UnconsumedField { slot: u8 },
    MalformedField { slot: u8 },
}

pub struct UiIntentPayloadProjection<P: UiIntentPayload> {
    values: Vec<Option<UiIntentProjectedValue>>,
    payload: PhantomData<fn() -> P>,
}

impl<P: UiIntentPayload> UiIntentPayloadProjection<P> {
    pub(crate) fn new(values: Vec<UiIntentProjectedValue>) -> Self {
        Self {
            values: values.into_iter().map(Some).collect(),
            payload: PhantomData,
        }
    }

    pub fn take<K: UiIntentPayloadValueKind>(
        &mut self,
        field: UiIntentPayloadField<P, K>,
    ) -> Result<K::Value, UiIntentPayloadProjectionViolation> {
        let descriptor = field.descriptor();
        let slot = descriptor.slot();
        let expected = P::FIELDS
            .fields()
            .get(slot as usize)
            .ok_or(UiIntentPayloadProjectionViolation::FieldOutsideSchema { slot })?;
        if expected != &descriptor || expected.kind() != K::KIND {
            return Err(UiIntentPayloadProjectionViolation::FieldHandleMismatch { slot });
        }
        let value = self
            .values
            .get_mut(slot as usize)
            .and_then(Option::take)
            .ok_or(UiIntentPayloadProjectionViolation::FieldAlreadyConsumed { slot })?;
        if value.kind() != K::KIND {
            return Err(UiIntentPayloadProjectionViolation::ValueKindMismatch);
        }
        K::take(value)
    }

    pub fn malformed<K: UiIntentPayloadValueKind>(
        field: UiIntentPayloadField<P, K>,
    ) -> UiIntentPayloadProjectionViolation {
        UiIntentPayloadProjectionViolation::MalformedField {
            slot: field.descriptor().slot(),
        }
    }

    pub(crate) fn finish(self) -> Result<(), UiIntentPayloadProjectionViolation> {
        self.values
            .iter()
            .position(Option::is_some)
            .map(|slot| UiIntentPayloadProjectionViolation::UnconsumedField {
                slot: u8::try_from(slot).expect("a projected value's slot is a u8 field slot"),
            })
            .map_or(Ok(()), Err)
    }
}

impl UiIntentProjectedValue {
    pub(crate) fn text(value: Arc<str>) -> Self {
        Self {
            value: UiIntentProjectedValueInner::Text(value),
        }
    }

    pub(crate) const fn boolean(value: bool) -> Self {
        Self {
            value: UiIntentProjectedValueInner::Boolean(value),
        }
    }

    pub(crate) const fn unsigned64(value: u64) -> Self {
        Self {
            value: UiIntentProjectedValueInner::Unsigned64(value),
        }
    }

    pub(crate) fn selection(value: UiIntentSelectionValue) -> Self {
        Self {
            value: UiIntentProjectedValueInner::Selection(value),
        }
    }

    pub(crate) const fn kind(&self) -> UiIntentPayloadFieldKind {
        match self.value {
            UiIntentProjectedValueInner::Text(_) => UiIntentPayloadFieldKind::Text,
            UiIntentProjectedValueInner::Boolean(_) => UiIntentPayloadFieldKind::Boolean,
            UiIntentProjectedValueInner::Unsigned64(_) => UiIntentPayloadFieldKind::Unsigned64,
            UiIntentProjectedValueInner::Selection(_) => UiIntentPayloadFieldKind::Selection,
        }
    }

    pub(crate) fn into_text(self) -> Option<Arc<str>> {
        match self.value {
            UiIntentProjectedValueInner::Text(value) => Some(value),
            UiIntentProjectedValueInner::Boolean(_)
            | UiIntentProjectedValueInner::Unsigned64(_)
            | UiIntentProjectedValueInner::Selection(_) => None,
        }
    }

    pub(crate) fn into_boolean(self) -> Option<bool> {
        match self.value {
            UiIntentProjectedValueInner::Boolean(value) => Some(value),
            UiIntentProjectedValueInner::Text(_)
            | UiIntentProjectedValueInner::Unsigned64(_)
            | UiIntentProjectedValueInner::Selection(_) => None,
        }
    }

    pub(crate) fn into_unsigned64(self) -> Option<u64> {
        match self.value {
            UiIntentProjectedValueInner::Unsigned64(value) => Some(value),
            UiIntentProjectedValueInner::Text(_)
            | UiIntentProjectedValueInner::Boolean(_)
            | UiIntentProjectedValueInner::Selection(_) => None,
        }
    }

    pub(crate) fn into_selection(self) -> Option<UiIntentSelectionValue> {
        match self.value {
            UiIntentProjectedValueInner::Selection(value) => Some(value),
            UiIntentProjectedValueInner::Text(_)
            | UiIntentProjectedValueInner::Boolean(_)
            | UiIntentProjectedValueInner::Unsigned64(_) => None,
        }
    }
}
