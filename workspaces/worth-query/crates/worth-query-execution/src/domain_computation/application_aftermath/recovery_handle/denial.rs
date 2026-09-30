//! Distinct denial causes for recovery-handle mint and transition (R8.28).

/// Why minting or transitioning a recovery handle was denied.
///
/// Each binding axis and lifecycle failure has its own variant so drift attacks
/// cannot collapse into one shared string or enum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRecoveryHandleDenialKind {
    RecoveryNotAdmitted,
    /// This authoritative commit already opened its sole recovery handle.
    RecoveryAlreadyMinted,
    RuntimeMismatch,
    SchemaMismatch,
    BranchMismatch,
    ApplicationBindingGenerationMismatch,
    OperationMismatch,
    GovernedInputMismatch,
    AttemptMismatch,
    PrincipalScopeMismatch,
    IdempotencyMismatch,
    /// Handle is right; the admitted idempotency read was minted for a foreign
    /// binding. Distinct from [`Self::IdempotencyMismatch`] on the handle axis.
    ForeignIdempotencyRead,
    ProviderPostureMismatch,
    CorrelationMismatch,
    CompatibilityGenerationMismatch,
    Expired,
    AlreadyTerminal,
    ForeignPrincipal,
    ForeignRuntime,
    ForeignBranchEqualOrdinal,
    TransitionNotAdmitted,
    /// The effect already reached its one terminal completion. Safe retry
    /// admits no new physical attempt and the live handle is returned.
    AlreadyCompleted,
    /// Installed mechanism axis does not admit compensate (distinct from reconcile).
    CompensationNotAdmitted,
    /// Installed authority axis does not admit reconcile (distinct from compensate).
    ReconciliationNotAdmitted,
    FreshAuthorityDenied,
    DisclosureAdmissionRequired,
    CurrentPolicyDenied,
    UnresolvedExternalPosture,
}

/// Refusal to mint, inspect, or transition a recovery handle.
///
/// Read [`Self::kind`] for the exact binding axis or lifecycle check that
/// failed. A transition denied during admission relinquishes the handle rather
/// than consuming it; no compensation, reconciliation, or retry took effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRecoveryHandleDenial {
    kind: WorthQueryRecoveryHandleDenialKind,
}

impl WorthQueryRecoveryHandleDenial {
    pub const fn new(kind: WorthQueryRecoveryHandleDenialKind) -> Self {
        Self { kind }
    }

    pub const fn kind(&self) -> WorthQueryRecoveryHandleDenialKind {
        self.kind
    }
}
