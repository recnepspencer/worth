//! Typed causes retained across temporal reentry retries.

/// The typed stage or native refusal retained across temporal reentry retries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalReentryFailure {
    PreconditionInvokerPanicked,
    ProjectionInvokerPanicked,
    EffectInvokerPanicked,
    IntentObsoleteBeforeProjection,
    IntentIdentityDecode(worth_query_installation::facade::ApplicationValueDecodeDenial),
    IntentRevisionUnrepresentable,
    IntentRevisionCannotAdvance,
    Invocation(super::operation_invocation::WorthQueryTemporalInvocationFailure),
    Principal(super::reconstruction_authority::WorthQueryTemporalPrincipalFailure),
    ApplicationAttempt(
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind,
    ),
    Idempotency(worth_foundational::facade::CanonicalDigestDerivationDenial),
    ProductSelection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    ManagedWake(worth_runtime_bridge::facade::BridgeManagedTemporalDenialKind),
    WakeRevisionExhausted,
    WakeDidNotRetire(worth_runtime_bridge::facade::BridgeManagedTemporalIntentReconciliation),
    CandidateLifetimeExpired {
        maximum_lifetime_millis: u64,
    },
    AbortedBeforeEffect,
    UnresolvedCommit(
        crate::domain_computation::primary_graph::WorthQueryApplicationUnresolvedCommitEvidence,
    ),
    SettlementIdempotencyAbsent,
    SettlementIdempotencyDrift,
}
