//! Typed authored-program validation diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationProgramValidationDenialKind {
    InvalidIdentity,
    DuplicateFeature,
    DuplicateAction,
    DuplicateConnection,
    DanglingFeature,
    MissingRequiredInput,
    DuplicateInputBinding,
    DuplicateOutput,
    DuplicateDerivedArtifact,
    AmbiguousDerivedArtifactProducer,
    UndeclaredOutput,
    UndeclaredArtifactOutput,
    UndeclaredInput,
    UnexportedCrossInstanceConnection,
    CyclicConnection,
    DuplicateRule,
    IncompleteActionChangeContract,
    InvalidActionDelivery,
    UngovernedDerivedOutput,
    DuplicateManagedComputation,
    MissingManagedComputationArtifact,
    MismatchedManagedComputationPartition,
    InvalidManagedComputationResources,
    DuplicateDerivedCollection,
    CanonicalRevisionBudgetExceeded,
    CanonicalRevisionUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationProgramValidationDenial {
    kind: ApplicationProgramValidationDenialKind,
    subject: String,
}

impl ApplicationProgramValidationDenial {
    pub const fn kind(&self) -> ApplicationProgramValidationDenialKind {
        self.kind
    }
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for ApplicationProgramValidationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application program denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for ApplicationProgramValidationDenial {}

pub(super) fn denial(
    kind: ApplicationProgramValidationDenialKind,
    subject: impl Into<String>,
) -> ApplicationProgramValidationDenial {
    ApplicationProgramValidationDenial {
        kind,
        subject: subject.into(),
    }
}
