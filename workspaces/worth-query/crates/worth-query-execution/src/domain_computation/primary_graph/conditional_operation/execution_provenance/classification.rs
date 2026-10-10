//! How a conditional operation's wake was decided and how its processing
//! ended.

/// How the signal graph decided a wake's condition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalSignalDecision {
    /// The condition changed; the operation may run.
    Eligible,
    /// No dependency changed, so the operation is suppressed.
    DependencyUnchanged,
    /// The condition was recomputed and reverted to its previous value, so the
    /// operation is suppressed.
    RevertedClean,
    /// The condition was suppressed before computing.
    Suppressed,
    /// The decision was deferred until a later condition, time, or demand.
    Deferred,
}

/// How processing one due wake ended during a clock observation.
///
/// Retained states keep the wake for a later observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalExecutionTerminal {
    /// The product branch moved before any effect; nothing took effect.
    ProductStale,
    /// The attempt failed to publish its product; its effects are held for
    /// recovery.
    ProductUnpublished,
    /// The operation ran and produced no effect.
    NoEffect,
    /// The wake is eligible and retained until its operation re-enters.
    EligibleRetained,
    /// The wake was suppressed and retained.
    SuppressedRetained,
    /// The wake was deferred, by the signal decision, capacity backpressure, or
    /// deferred settlement, and retained.
    DeferredRetained,
    /// The attempt failed in a way that can be retried.
    Retryable,
    /// The attempt was cancelled or timed out.
    ControlStopped,
    /// Whether the operation committed is not known yet.
    Indeterminate,
    /// The operation committed.
    Committed,
    /// The operation had already committed for this wake.
    AlreadyCommitted,
    /// The attempt failed and will not be retried.
    Failed,
}

/// Why processing a due wake ended as it did, when a specific cause is known.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalExecutionCause {
    /// A reentry retry or unresolved outcome retains the original typed cause.
    Reentry(super::super::WorthQueryConditionalReentryFailure),
    /// Bridge refused the retained conditional decision with this native cause.
    BridgeConditional(worth_runtime_bridge::facade::BridgeConditionalDenialKind),
    /// Reentry did not finish replacing its retained decision.
    InterruptedDuringReentry,
    /// The original execution refusal, whether retryable or terminal.
    ApplicationCommitDenied(
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind,
    ),
    /// Execution evidence whose HEAD application category and stage are preserved.
    ApplicationExecutionDenied {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        cause: Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    },
    /// Settlement remains owed after this typed publication refusal.
    SettlementExecutionDenied(crate::domain_computation::WorthQueryProviderSessionDenialKind),
    /// The product branch head changed.
    ProductHeadChanged,
    /// The operation produced no effect for this reason.
    NoEffect(crate::domain_computation::primary_graph::WorthQueryApplicationNoEffectCause),
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    /// No capacity remains to retain a basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// The commit position was contended; the commit was deferred.
    PatchPositionReservationContended,
    RelationalDeferred(worth_relational::facade::mvcc::RelationalPublicationDeferred),
    /// A source mark changed during same-image verification; retry with a new selected root.
    SourceCurrentnessRaced(worth_relational::facade::mvcc::CompanionCellEditStop),
    RequiredPrerequisitePending(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind,
    ),
    /// The provider's candidate limit was reached; the commit was deferred.
    CandidateCapacityExhausted {
        maximum_candidates: usize,
    },
    /// The provider's published-snapshot limit was reached; the commit was
    /// deferred.
    PublishedSnapshotCapacityExhausted {
        maximum_handles: usize,
    },
    /// The attempt was cancelled.
    Cancelled,
    /// The attempt timed out.
    TimedOut,
    /// Admission or commit refused the attempt with a terminal denial.
    TerminalFailure,
}

pub(in crate::domain_computation::primary_graph::conditional_operation) fn signal_decision(
    class: worth_signal::facade::SignalConditionalDecisionClass,
) -> WorthQueryConditionalSignalDecision {
    use worth_signal::facade::SignalConditionalDecisionClass as Class;
    match class {
        Class::ComputedChanged => WorthQueryConditionalSignalDecision::Eligible,
        Class::DependencyUnchanged => WorthQueryConditionalSignalDecision::DependencyUnchanged,
        Class::ComputedRevertedClean => WorthQueryConditionalSignalDecision::RevertedClean,
        Class::SuppressedBeforeCompute => WorthQueryConditionalSignalDecision::Suppressed,
        Class::DeferredByCondition | Class::DeferredTemporal | Class::DeferredOnDemand => {
            WorthQueryConditionalSignalDecision::Deferred
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_signal::facade::SignalConditionalDecisionClass as Class;

    #[test]
    fn query_provenance_preserves_every_signal_decision_class() {
        for (class, expected) in [
            (
                Class::ComputedChanged,
                WorthQueryConditionalSignalDecision::Eligible,
            ),
            (
                Class::DependencyUnchanged,
                WorthQueryConditionalSignalDecision::DependencyUnchanged,
            ),
            (
                Class::ComputedRevertedClean,
                WorthQueryConditionalSignalDecision::RevertedClean,
            ),
            (
                Class::SuppressedBeforeCompute,
                WorthQueryConditionalSignalDecision::Suppressed,
            ),
            (
                Class::DeferredByCondition,
                WorthQueryConditionalSignalDecision::Deferred,
            ),
            (
                Class::DeferredTemporal,
                WorthQueryConditionalSignalDecision::Deferred,
            ),
            (
                Class::DeferredOnDemand,
                WorthQueryConditionalSignalDecision::Deferred,
            ),
        ] {
            assert_eq!(signal_decision(class), expected);
        }
    }
}
