//! Closed vocabulary for external-effect causal stages.

/// Meaning of one external-effect causal event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalEffectPostureKind {
    ProviderCommit,
    EmittedApplicationCausality,
    DispatchAttempt,
    ExternalAcknowledgement,
    ExternalCompletion,
    Compensation,
    Reconciliation,
}
