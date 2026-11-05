//! Aftermath installation denials.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAftermathInstallationDenialKind {
    MechanismRequired,
    MechanismPresentForNotCorrectable,
    ReconciliationRequired,
    ReconciliationForbidden,
    ExternalEffectRejectsReversible,
    PreImageDemandNotCoveredByDeclaredReads,
    PreImageDemandNotCoveredByAuthoredAnchors,
    PreImageDemandExceedsBound,
    CanonicalEntryLimitExceeded,
    CanonicalByteLimitExceeded,
    CanonicalDigestSlotRejected,
    MissingDeclaredReadsCoverage,
    /// Declared lowering slot does not resolve in the install-time catalog (R8.9).
    LoweringCorrespondenceUnresolved,
    /// Resolved correspondence belongs to a different compatibility generation.
    LoweringCorrespondenceWrongGeneration,
    /// Resolved correspondence does not match the operation's graph participation.
    LoweringCorrespondenceMismatchedGraphParticipation,
    /// Multiple catalog entries claim the same diagnostic slot.
    LoweringCorrespondenceAmbiguous,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAftermathInstallationDenial {
    kind: WorthQueryAftermathInstallationDenialKind,
    subject: String,
}

impl WorthQueryAftermathInstallationDenial {
    pub(crate) fn new(
        kind: WorthQueryAftermathInstallationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryAftermathInstallationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}
