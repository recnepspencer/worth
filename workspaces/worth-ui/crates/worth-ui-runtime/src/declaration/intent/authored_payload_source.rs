use core::marker::PhantomData;

use crate::capability::{
    UiIntentBoolean, UiIntentPayloadValueKind, UiIntentSelection, UiIntentText, UiIntentUnsigned64,
};

pub struct UiIntentPayloadSource<K: UiIntentPayloadValueKind> {
    source: UiAuthoredIntentPayloadSource,
    kind: PhantomData<fn() -> K>,
}

enum UiAuthoredIntentPayloadSource {
    ProjectionText(Box<str>),
    ProjectionSelection(Box<str>),
    CommittedDraft,
    ConstantText(Box<str>),
    ConstantBoolean(bool),
    ConstantUnsigned64(u64),
    ApplicationText(Box<str>),
    ApplicationBoolean(Box<str>),
    ApplicationUnsigned64(Box<str>),
    Derived(Box<str>),
    Condition(Box<str>),
}

impl UiIntentPayloadSource<UiIntentText> {
    pub fn projection(identity: &worth_ui_query_binding::WorthUiQueryViewIdentity) -> Self {
        Self::new(UiAuthoredIntentPayloadSource::ProjectionText(
            identity.as_str().into(),
        ))
    }

    pub fn committed_draft() -> Self {
        Self::new(UiAuthoredIntentPayloadSource::CommittedDraft)
    }

    pub fn constant(value: impl Into<Box<str>>) -> Self {
        Self::new(UiAuthoredIntentPayloadSource::ConstantText(value.into()))
    }

    pub fn application_fact(fact: &super::UiIntentApplicationFact<UiIntentText>) -> Self {
        Self::new(UiAuthoredIntentPayloadSource::ApplicationText(
            fact.identity().into(),
        ))
    }

    /// Text read from the `derived text` declaration named `identity`. The
    /// package refuses an undeclared expression, a condition, and a derived
    /// token; the intent catalog refuses any other result type.
    pub fn derived(identity: impl Into<Box<str>>) -> Self {
        Self::new(UiAuthoredIntentPayloadSource::Derived(identity.into()))
    }
}

impl UiIntentPayloadSource<UiIntentBoolean> {
    pub fn constant(value: bool) -> Self {
        Self::new(UiAuthoredIntentPayloadSource::ConstantBoolean(value))
    }

    pub fn application_fact(fact: &super::UiIntentApplicationFact<UiIntentBoolean>) -> Self {
        Self::new(UiAuthoredIntentPayloadSource::ApplicationBoolean(
            fact.identity().into(),
        ))
    }

    /// The Boolean the condition named `identity` holds: a computed Boolean
    /// is always a condition. The package refuses an undeclared expression
    /// and a derived one.
    pub fn condition(identity: impl Into<Box<str>>) -> Self {
        Self::new(UiAuthoredIntentPayloadSource::Condition(identity.into()))
    }
}

impl UiIntentPayloadSource<UiIntentUnsigned64> {
    pub fn constant(value: u64) -> Self {
        Self::new(UiAuthoredIntentPayloadSource::ConstantUnsigned64(value))
    }

    pub fn application_fact(fact: &super::UiIntentApplicationFact<UiIntentUnsigned64>) -> Self {
        Self::new(UiAuthoredIntentPayloadSource::ApplicationUnsigned64(
            fact.identity().into(),
        ))
    }
}

impl UiIntentPayloadSource<UiIntentSelection> {
    pub fn projection(identity: &worth_ui_query_binding::WorthUiQueryViewIdentity) -> Self {
        Self::new(UiAuthoredIntentPayloadSource::ProjectionSelection(
            identity.as_str().into(),
        ))
    }
}

impl<K: UiIntentPayloadValueKind> UiIntentPayloadSource<K> {
    fn new(source: UiAuthoredIntentPayloadSource) -> Self {
        Self {
            source,
            kind: PhantomData,
        }
    }

    pub(super) fn into_dsl(
        self,
        field: &'static str,
    ) -> worth_ui_dsl::WorthUiIntentPayloadSourceSpec {
        match self.source {
            UiAuthoredIntentPayloadSource::ProjectionText(projection) => {
                worth_ui_dsl::WorthUiIntentPayloadSourceSpec::projection_text(field, projection)
            }
            UiAuthoredIntentPayloadSource::ProjectionSelection(projection) => {
                worth_ui_dsl::WorthUiIntentPayloadSourceSpec::projection_selection(
                    field, projection,
                )
            }
            UiAuthoredIntentPayloadSource::CommittedDraft => {
                worth_ui_dsl::WorthUiIntentPayloadSourceSpec::committed_draft(field)
            }
            UiAuthoredIntentPayloadSource::ConstantText(value) => {
                worth_ui_dsl::WorthUiIntentPayloadSourceSpec::constant_text(field, value)
            }
            UiAuthoredIntentPayloadSource::ConstantBoolean(value) => {
                worth_ui_dsl::WorthUiIntentPayloadSourceSpec::constant_boolean(field, value)
            }
            UiAuthoredIntentPayloadSource::ConstantUnsigned64(value) => {
                worth_ui_dsl::WorthUiIntentPayloadSourceSpec::constant_unsigned64(field, value)
            }
            UiAuthoredIntentPayloadSource::ApplicationText(fact) => {
                worth_ui_dsl::WorthUiIntentPayloadSourceSpec::application_text(field, fact)
            }
            UiAuthoredIntentPayloadSource::ApplicationBoolean(fact) => {
                worth_ui_dsl::WorthUiIntentPayloadSourceSpec::application_boolean(field, fact)
            }
            UiAuthoredIntentPayloadSource::ApplicationUnsigned64(fact) => {
                worth_ui_dsl::WorthUiIntentPayloadSourceSpec::application_unsigned64(field, fact)
            }
            UiAuthoredIntentPayloadSource::Derived(expression) => {
                worth_ui_dsl::WorthUiIntentPayloadSourceSpec::derived(field, expression)
            }
            UiAuthoredIntentPayloadSource::Condition(expression) => {
                worth_ui_dsl::WorthUiIntentPayloadSourceSpec::condition(field, expression)
            }
        }
    }
}
