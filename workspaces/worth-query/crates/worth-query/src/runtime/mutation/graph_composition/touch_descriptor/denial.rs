use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryGraphTouchDescriptorDenialKind {
    EvidenceIdentityDerivationDenied,
    ProgramComponentCountMismatch,
    BreadthProgramComponentCountMismatch,
    SymbolicEntityDeclarationCountMismatch,
    SymbolicRelationDeclarationCountMismatch,
    ProgramStepIndexOutOfBounds,
    DuplicateProgramStepIndex,
    EmptyReadCollection,
    EmptyDeclaredMutationCollection,
    InvalidAspectPath,
    ProgramCommandCollectionMismatch,
    ProgramCommandSymbolMismatch,
    ProgramCommandMutationFamilyMismatch,
}

impl WorthQueryGraphTouchDescriptorDenialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied => "evidence-identity-derivation-denied",
            Self::ProgramComponentCountMismatch => "program-component-count-mismatch",
            Self::BreadthProgramComponentCountMismatch => {
                "breadth-program-component-count-mismatch"
            }
            Self::SymbolicEntityDeclarationCountMismatch => {
                "symbolic-entity-declaration-count-mismatch"
            }
            Self::SymbolicRelationDeclarationCountMismatch => {
                "symbolic-relation-declaration-count-mismatch"
            }
            Self::ProgramStepIndexOutOfBounds => "program-step-index-out-of-bounds",
            Self::DuplicateProgramStepIndex => "duplicate-program-step-index",
            Self::EmptyReadCollection => "empty-read-collection",
            Self::EmptyDeclaredMutationCollection => "empty-declared-mutation-collection",
            Self::InvalidAspectPath => "invalid-aspect-path",
            Self::ProgramCommandCollectionMismatch => "program-command-collection-mismatch",
            Self::ProgramCommandSymbolMismatch => "program-command-symbol-mismatch",
            Self::ProgramCommandMutationFamilyMismatch => {
                "program-command-mutation-family-mismatch"
            }
        }
    }
}

impl std::fmt::Display for WorthQueryGraphTouchDescriptorDenialKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphTouchDescriptorDenial {
    identity_derivation_denial: Option<IdentityDerivationDenial>,
    kind: WorthQueryGraphTouchDescriptorDenialKind,
    message: String,
}

impl WorthQueryGraphTouchDescriptorDenial {
    pub fn identity_derivation_denial(&self) -> Option<&IdentityDerivationDenial> {
        self.identity_derivation_denial.as_ref()
    }

    pub(crate) fn new(
        kind: WorthQueryGraphTouchDescriptorDenialKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            identity_derivation_denial: None,
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryGraphTouchDescriptorDenialKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for WorthQueryGraphTouchDescriptorDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "graph touch descriptor denied for {}: {}",
            self.kind, self.message
        )
    }
}

impl std::error::Error for WorthQueryGraphTouchDescriptorDenial {}

impl From<IdentityDerivationDenial> for WorthQueryGraphTouchDescriptorDenial {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self {
            kind: WorthQueryGraphTouchDescriptorDenialKind::EvidenceIdentityDerivationDenied,
            message: "graph touch evidence identity exceeded its digest bounds".into(),
            identity_derivation_denial: Some(cause),
        }
    }
}
