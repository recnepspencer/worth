//! Typed refusals from installed schema authoring.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationSchemaAuthoringDenialKind {
    UnknownEntity,
    UnknownField,
    FieldFamilyMismatch,
    FieldValueTypeMismatch,
    FieldUnitMismatch,
    FieldNotWritable,
    FieldNotEqualityQueryable,
    UnknownOperation,
    OperationProvenanceMismatch,
    OperationInputTypeMismatch,
    UnknownEffect,
    EffectProvenanceMismatch,
    EffectPayloadTypeMismatch,
    UnknownRelation,
    OperationProgramNotInstalled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSchemaAuthoringDenial {
    kind: ApplicationSchemaAuthoringDenialKind,
    subject: String,
}

impl ApplicationSchemaAuthoringDenial {
    pub(crate) fn new(
        kind: ApplicationSchemaAuthoringDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> ApplicationSchemaAuthoringDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for ApplicationSchemaAuthoringDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application schema authoring denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for ApplicationSchemaAuthoringDenial {}
