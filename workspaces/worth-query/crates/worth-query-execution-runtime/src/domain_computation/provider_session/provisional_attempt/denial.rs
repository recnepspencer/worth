use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProvisionalDenialKind {
    InvalidProgram,
    UndeclaredEffectFamily,
    UndeclaredTarget,
    UndeclaredArtifactDependency,
    SymbolAlreadyDefined,
    ProposedFactIdentityAlreadyDefined,
    UnknownSymbolicReference,
    SessionBindingMismatch,
    ProposalBasisMismatch,
    ProviderUnsupported,
    ProviderRejected,
    ProviderPanicked,
    ProviderEvidenceSubstitution,
    ProviderProgramMismatch,
    DiscardFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProvisionalFailure {
    kind: WorthQueryProvisionalDenialKind,
    recovery_posture:
        crate::domain_computation::provider_session::WorthQueryProviderSessionRecoveryPosture,
    detail: Arc<str>,
}

impl WorthQueryProvisionalFailure {
    pub fn new(kind: WorthQueryProvisionalDenialKind, detail: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            recovery_posture:
                crate::domain_computation::provider_session::WorthQueryProviderSessionRecoveryPosture::Closed,
            detail: detail.into(),
        }
    }

    pub fn invalid_program(detail: &'static str) -> Self {
        Self::new(WorthQueryProvisionalDenialKind::InvalidProgram, detail)
    }

    pub fn kind(&self) -> WorthQueryProvisionalDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn recovery_posture(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionRecoveryPosture {
        self.recovery_posture
    }

    pub(super) fn with_recovery_posture(
        mut self,
        posture: crate::domain_computation::provider_session::WorthQueryProviderSessionRecoveryPosture,
    ) -> Self {
        self.recovery_posture = posture;
        self
    }
}
