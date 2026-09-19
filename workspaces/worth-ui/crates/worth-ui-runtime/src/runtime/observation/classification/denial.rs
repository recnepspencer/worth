#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAuthoredFactDeclarationSide {
    Predecessor,
    Candidate,
}

#[derive(Debug)]
pub enum UiChangeClassificationDenial {
    ForeignSession,
    ForeignSourceBasis,
    ForeignApplicationGeneration,
    StalePointerPresenceTransition,
    MissingHostReport {
        family: worth_ui_host_contract::UiHostObservationFamily,
    },
    SourcePreparation(Box<crate::facade::lifecycle::WorthUiApplicationPreparationDenial>),
    CandidateAdmission(Box<crate::runtime::WorthUiCandidateAdmissionReport>),
    ArtifactComparison(crate::runtime::WorthUiRuntimeArtifactComparisonDenial),
    ReplacementPlanning(Box<crate::runtime::WorthUiReplacementLoweringDenial>),
    IdentityLifecycle(Box<crate::runtime::rebind::UiIdentityLifecycleDenial>),
    MissingAuthoredFactDeclaration {
        side: UiAuthoredFactDeclarationSide,
        provenance_digest: u64,
    },
    AmbiguousAuthoredFactDeclaration {
        side: UiAuthoredFactDeclarationSide,
        provenance_digest: u64,
        matches: usize,
    },
    InconsistentAppearanceAttachment {
        side: UiAuthoredFactDeclarationSide,
        declaration: Box<str>,
    },
    AuthoredFactDeclarationIdentityMismatch {
        predecessor: Box<str>,
        candidate: Box<str>,
    },
    ChangedFactCapacityExceeded {
        limit: usize,
        observed: usize,
    },
}
