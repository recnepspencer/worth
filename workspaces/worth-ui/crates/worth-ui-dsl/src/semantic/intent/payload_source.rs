#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiIntentPayloadSourceSpec {
    field: Box<str>,
    source: WorthUiIntentPayloadSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthUiIntentPayloadSource {
    ProjectionText {
        projection: Box<str>,
    },
    ProjectionSelection {
        projection: Box<str>,
    },
    CommittedDraft,
    ConstantText {
        value: Box<str>,
    },
    ConstantBoolean {
        value: bool,
    },
    ConstantUnsigned64 {
        value: u64,
    },
    ApplicationText {
        fact: Box<str>,
    },
    ApplicationBoolean {
        fact: Box<str>,
    },
    ApplicationUnsigned64 {
        fact: Box<str>,
    },
    /// The retained value of an admitted `derived` declaration.
    Derived {
        expression: Box<str>,
    },
    /// The retained outcome of an admitted `condition` declaration: the one
    /// computed Boolean form.
    Condition {
        expression: Box<str>,
    },
}

impl WorthUiIntentPayloadSourceSpec {
    pub fn projection_text(field: impl Into<Box<str>>, projection: impl Into<Box<str>>) -> Self {
        Self::new(
            field,
            WorthUiIntentPayloadSource::ProjectionText {
                projection: projection.into(),
            },
        )
    }

    pub fn projection_selection(
        field: impl Into<Box<str>>,
        projection: impl Into<Box<str>>,
    ) -> Self {
        Self::new(
            field,
            WorthUiIntentPayloadSource::ProjectionSelection {
                projection: projection.into(),
            },
        )
    }

    pub fn committed_draft(field: impl Into<Box<str>>) -> Self {
        Self::new(field, WorthUiIntentPayloadSource::CommittedDraft)
    }

    pub fn constant_text(field: impl Into<Box<str>>, value: impl Into<Box<str>>) -> Self {
        Self::new(
            field,
            WorthUiIntentPayloadSource::ConstantText {
                value: value.into(),
            },
        )
    }

    pub fn constant_boolean(field: impl Into<Box<str>>, value: bool) -> Self {
        Self::new(field, WorthUiIntentPayloadSource::ConstantBoolean { value })
    }

    pub fn constant_unsigned64(field: impl Into<Box<str>>, value: u64) -> Self {
        Self::new(
            field,
            WorthUiIntentPayloadSource::ConstantUnsigned64 { value },
        )
    }

    pub fn application_text(field: impl Into<Box<str>>, fact: impl Into<Box<str>>) -> Self {
        Self::new(
            field,
            WorthUiIntentPayloadSource::ApplicationText { fact: fact.into() },
        )
    }

    pub fn application_boolean(field: impl Into<Box<str>>, fact: impl Into<Box<str>>) -> Self {
        Self::new(
            field,
            WorthUiIntentPayloadSource::ApplicationBoolean { fact: fact.into() },
        )
    }

    pub fn application_unsigned64(field: impl Into<Box<str>>, fact: impl Into<Box<str>>) -> Self {
        Self::new(
            field,
            WorthUiIntentPayloadSource::ApplicationUnsigned64 { fact: fact.into() },
        )
    }

    pub fn derived(field: impl Into<Box<str>>, expression: impl Into<Box<str>>) -> Self {
        Self::new(
            field,
            WorthUiIntentPayloadSource::Derived {
                expression: expression.into(),
            },
        )
    }

    pub fn condition(field: impl Into<Box<str>>, expression: impl Into<Box<str>>) -> Self {
        Self::new(
            field,
            WorthUiIntentPayloadSource::Condition {
                expression: expression.into(),
            },
        )
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub const fn source(&self) -> &WorthUiIntentPayloadSource {
        &self.source
    }

    pub(crate) fn revision_token(&self) -> String {
        let (kind, value) = self.source.revision_parts();
        format!(
            "payload-source:{}:{}:{}:{}:{}",
            self.field.len(),
            self.field,
            kind,
            value.len(),
            value
        )
    }

    fn new(field: impl Into<Box<str>>, source: WorthUiIntentPayloadSource) -> Self {
        let field = field.into();
        assert!(!field.trim().is_empty(), "payload field cannot be empty");
        Self { field, source }
    }
}

impl WorthUiIntentPayloadSource {
    pub(crate) fn revision_parts(&self) -> (&'static str, String) {
        match self {
            Self::ProjectionText { projection } => ("projection-text", projection.to_string()),
            Self::ProjectionSelection { projection } => {
                ("projection-selection", projection.to_string())
            }
            Self::CommittedDraft => ("committed-draft", String::new()),
            Self::ConstantText { value } => ("constant-text", value.to_string()),
            Self::ConstantBoolean { value } => ("constant-boolean", value.to_string()),
            Self::ConstantUnsigned64 { value } => ("constant-unsigned64", value.to_string()),
            Self::ApplicationText { fact } => ("application-text", fact.to_string()),
            Self::ApplicationBoolean { fact } => ("application-boolean", fact.to_string()),
            Self::ApplicationUnsigned64 { fact } => ("application-unsigned64", fact.to_string()),
            Self::Derived { expression } => ("derived", expression.to_string()),
            Self::Condition { expression } => ("condition", expression.to_string()),
        }
    }
}
