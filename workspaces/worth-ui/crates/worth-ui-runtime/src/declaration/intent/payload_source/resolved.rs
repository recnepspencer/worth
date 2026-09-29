use std::sync::Arc;

use crate::capability::UiIntentPayloadFieldDescriptor;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiResolvedIntentPayloadBinding {
    pub(super) field: UiIntentPayloadFieldDescriptor,
    pub(super) source: UiResolvedIntentPayloadSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiResolvedIntentProjectionSource {
    pub(super) identity: worth_ui_query_binding::WorthUiQueryViewIdentity,
    pub(super) slot: worth_ui_query_binding::UiProjectionInputSlot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiResolvedIntentApplicationSource {
    pub(super) identity: Box<str>,
    pub(super) slot: super::UiIntentApplicationFactSlot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiResolvedIntentPayloadSource {
    ProjectionText(UiResolvedIntentProjectionSource),
    ProjectionSelection(UiResolvedIntentProjectionSource),
    CommittedDraft,
    ConstantText(Arc<str>),
    ConstantBoolean(bool),
    ConstantUnsigned64(u64),
    ApplicationText(UiResolvedIntentApplicationSource),
    ApplicationBoolean(UiResolvedIntentApplicationSource),
    ApplicationUnsigned64(UiResolvedIntentApplicationSource),
    /// A text field read from an admitted `derived text` declaration.
    DerivedText(super::UiResolvedIntentExpressionSource),
    /// An unsigned 64-bit field read from an admitted `derived integer`
    /// declaration. The range is checked when the payload reads the value.
    DerivedInteger(super::UiResolvedIntentExpressionSource),
    /// A Boolean field read from an admitted condition.
    Condition(super::UiResolvedIntentExpressionSource),
}

impl UiResolvedIntentPayloadBinding {
    pub(crate) const fn field(&self) -> UiIntentPayloadFieldDescriptor {
        self.field
    }

    pub(crate) const fn source(&self) -> &UiResolvedIntentPayloadSource {
        &self.source
    }
}

impl UiResolvedIntentProjectionSource {
    pub(crate) fn identity(&self) -> &worth_ui_query_binding::WorthUiQueryViewIdentity {
        &self.identity
    }

    pub(crate) const fn slot(&self) -> worth_ui_query_binding::UiProjectionInputSlot {
        self.slot
    }
}

impl UiResolvedIntentApplicationSource {
    pub(crate) fn identity(&self) -> &str {
        &self.identity
    }

    pub(crate) const fn slot(&self) -> super::UiIntentApplicationFactSlot {
        self.slot
    }
}
