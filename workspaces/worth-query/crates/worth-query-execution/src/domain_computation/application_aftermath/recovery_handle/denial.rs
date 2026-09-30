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
    /// The effect already reached its one terminal completion. Safe retry
    /// admits no new physical attempt and the live handle is returned.
    AlreadyCompleted,
    /// The effect completed, but publishing that completion is still in flight
    /// or awaits its retry. No attempt is admitted and the live handle is
    /// returned; once the publication resolves, retry answers
    /// [`Self::AlreadyCompleted`].
    CompletionPublicationPending,
    /// The terminal completion index cannot answer for this effect, so no
    /// attempt is admitted and the live handle is returned. It answers again
    /// once a pending completion publication resolves or the index is repaired.
    TerminalIndexUnavailable,
    /// The live handle binding carries no co-committed dispatch outbox.
    DispatchOutboxMissing,
    /// No external-effect transport is installed on this runtime.
    TransportNotInstalled,
    /// Relational could not establish the exact committed dispatch owner row.
    DispatchOwnerReadDenied(
        crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial,
    ),
    /// This runtime could not mint a runtime-affine physical attempt.
    AttemptAdmissionDenied,
    /// Canonical derivation of the dispatch event identity failed.
    CanonicalDerivationDenied,
    /// The installed runtime clock could not classify the physical attempt.
    TimeObservationDenied,
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
