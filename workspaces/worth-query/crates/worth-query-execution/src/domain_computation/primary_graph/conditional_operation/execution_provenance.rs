use super::signal_decision_reentry::{
    WorthQueryRetainedConditionalDecision, WorthQueryRetainedConditionalWake,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalSignalDecision {
    Eligible,
    DependencyUnchanged,
    RevertedClean,
    Suppressed,
    Deferred,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalExecutionTerminal {
    ProductStale,
    ProductUnpublished,
    EligibleRetained,
    SuppressedRetained,
    DeferredRetained,
    Retryable,
    ControlStopped,
    Indeterminate,
    Committed,
    AlreadyCommitted,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalExecutionCause {
    ProductHeadChanged,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    PatchPositionReservationContended,
    CandidateCapacityExhausted { maximum_candidates: usize },
    PublishedSnapshotCapacityExhausted { maximum_handles: usize },
    Cancelled,
    TimedOut,
    TerminalFailure,
}

/// Descriptive, non-authorizing lineage for one temporal wake processed by an
/// observed clock reading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConditionalExecutionProvenance {
    intent_identity: String,
    intent_revision: u64,
    due_coordinate: u64,
    signal_scheduled_ordinal: u64,
    signal_ready_ordinal: u64,
    signal_decision: Option<WorthQueryConditionalSignalDecision>,
    application_attempt_ordinal: Option<u64>,
    terminal: WorthQueryConditionalExecutionTerminal,
    cause: Option<WorthQueryConditionalExecutionCause>,
    canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases,
}

impl WorthQueryConditionalExecutionProvenance {
    pub fn intent_identity(&self) -> &str {
        &self.intent_identity
    }
    pub fn intent_revision(&self) -> u64 {
        self.intent_revision
    }
    pub fn due_coordinate(&self) -> u64 {
        self.due_coordinate
    }
    pub fn signal_scheduled_ordinal(&self) -> u64 {
        self.signal_scheduled_ordinal
    }
    pub fn signal_ready_ordinal(&self) -> u64 {
        self.signal_ready_ordinal
    }
    pub fn signal_decision(&self) -> Option<WorthQueryConditionalSignalDecision> {
        self.signal_decision
    }
    pub fn application_attempt_ordinal(&self) -> Option<u64> {
        self.application_attempt_ordinal
    }
    pub fn terminal(&self) -> WorthQueryConditionalExecutionTerminal {
        self.terminal
    }

    pub fn cause(&self) -> Option<WorthQueryConditionalExecutionCause> {
        self.cause
    }

    pub const fn canonical_work(
        &self,
    ) -> worth_query_installation::facade::WorthQueryCanonicalWorkPhases {
        self.canonical_work
    }
}

pub(super) fn execution_provenance(
    wakes: &[WorthQueryRetainedConditionalWake],
) -> Vec<WorthQueryConditionalExecutionProvenance> {
    wakes
        .iter()
        .map(|wake| WorthQueryConditionalExecutionProvenance {
            intent_identity: wake.due.intent_identity().as_str().to_string(),
            intent_revision: wake.due.revision(),
            due_coordinate: wake.due.due_coordinate(),
            signal_scheduled_ordinal: wake.due.signal_scheduled_ordinal(),
            signal_ready_ordinal: wake.due.signal_ready_ordinal(),
            signal_decision: wake.last_signal_decision,
            application_attempt_ordinal: wake.application_attempted.then_some(wake.attempt),
            terminal: terminal(&wake.decision),
            cause: cause(&wake.decision),
            canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkPhases::new(
                worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
                wake.application_admission_canonical_work,
                worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
                worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
                worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
            ),
        })
        .collect()
}

fn cause(
    decision: &WorthQueryRetainedConditionalDecision,
) -> Option<WorthQueryConditionalExecutionCause> {
    use super::application_operation_reentry::{
        WorthQueryTemporalAdmissionTerminalFailure as Admission,
        WorthQueryTemporalControlStop as Control, WorthQueryTemporalTerminalFailure as Terminal,
    };
    use super::signal_decision_reentry::WorthQueryOperationBackpressureCause as Backpressure;
    match decision {
        WorthQueryRetainedConditionalDecision::OperationProductStale(_, _) => {
            Some(WorthQueryConditionalExecutionCause::ProductHeadChanged)
        }
        WorthQueryRetainedConditionalDecision::OperationBackpressured(_, cause) => match cause {
            Backpressure::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Some(WorthQueryConditionalExecutionCause::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: *maximum_active_snapshots,
            }),
            Backpressure::RetentionCapacityExhausted => {
                Some(WorthQueryConditionalExecutionCause::RetentionCapacityExhausted)
            }
            Backpressure::ProviderCommit(kind) => provider_commit_cause(*kind),
        },
        WorthQueryRetainedConditionalDecision::OperationControlStopped(_, Control::Cancelled) => {
            Some(WorthQueryConditionalExecutionCause::Cancelled)
        }
        WorthQueryRetainedConditionalDecision::OperationControlStopped(_, Control::TimedOut) => {
            Some(WorthQueryConditionalExecutionCause::TimedOut)
        }
        WorthQueryRetainedConditionalDecision::OperationTerminalFailure(
            _,
            Terminal::ApplicationCommit(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::RetentionIdentityExhausted,
            ),
        ) => Some(WorthQueryConditionalExecutionCause::RetentionIdentityExhausted),
        WorthQueryRetainedConditionalDecision::OperationTerminalFailure(_, Terminal::Admission(failure))
            if admission_retention_identity_exhausted(*failure) =>
        {
            Some(WorthQueryConditionalExecutionCause::RetentionIdentityExhausted)
        }
        WorthQueryRetainedConditionalDecision::OperationTerminalFailure(
            _,
            Terminal::ApplicationCommit(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::SnapshotIdentityExhausted,
            ),
        ) => Some(WorthQueryConditionalExecutionCause::SnapshotIdentityExhausted),
        WorthQueryRetainedConditionalDecision::OperationTerminalFailure(_, Terminal::Admission(failure))
            if admission_snapshot_identity_exhausted(*failure) =>
        {
            Some(WorthQueryConditionalExecutionCause::SnapshotIdentityExhausted)
        }
        WorthQueryRetainedConditionalDecision::OperationTerminalFailure(
            _,
            Terminal::Admission(Admission::Principal(_)
                | Admission::Entity(_)
                | Admission::Authorization(_)
                | Admission::Projection(_)
                | Admission::Invariant(_)),
        )
        | WorthQueryRetainedConditionalDecision::OperationTerminalFailure(
            _,
            Terminal::ApplicationCommit(_),
        ) => Some(WorthQueryConditionalExecutionCause::TerminalFailure),
        _ => None,
    }
}

fn provider_commit_cause(
    kind: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDeferredKind,
) -> Option<WorthQueryConditionalExecutionCause> {
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitDeferredKind as Kind;
    match kind {
        Kind::RetentionCapacityExhausted => {
            Some(WorthQueryConditionalExecutionCause::RetentionCapacityExhausted)
        }
        Kind::PatchPositionReservationContended => {
            Some(WorthQueryConditionalExecutionCause::PatchPositionReservationContended)
        }
        Kind::CandidateCapacityExhausted { maximum_candidates } => Some(
            WorthQueryConditionalExecutionCause::CandidateCapacityExhausted { maximum_candidates },
        ),
        Kind::PublishedSnapshotCapacityExhausted { maximum_handles } => Some(
            WorthQueryConditionalExecutionCause::PublishedSnapshotCapacityExhausted {
                maximum_handles,
            },
        ),
        Kind::CandidateLifetimeExpired { .. } => None,
    }
}

fn admission_retention_identity_exhausted(
    failure: super::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure,
) -> bool {
    use super::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure as Admission;
    match failure {
        Admission::Principal(kind) => matches!(
            kind,
            crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind::RetentionIdentityExhausted
        ),
        Admission::Entity(kind) => matches!(
            kind,
            crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::RetentionIdentityExhausted
        ),
        Admission::Authorization(kind) => matches!(
            kind,
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::RetentionIdentityExhausted
        ),
        Admission::Projection(kind) => matches!(
            kind,
            crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenialKind::Authorization(
                crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::RetentionIdentityExhausted
            ) | crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenialKind::InvariantAdmission(
                crate::domain_computation::primary_graph::WorthQueryInvariantProjectionDenialKind::RetentionIdentityExhausted
            )
        ),
        Admission::Invariant(kind) => matches!(
            kind,
            crate::domain_computation::primary_graph::WorthQueryInvariantProjectionDenialKind::RetentionIdentityExhausted
        ),
    }
}

fn admission_snapshot_identity_exhausted(
    failure: super::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure,
) -> bool {
    use super::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure as Admission;
    match failure {
        Admission::Principal(kind) => matches!(
            kind,
            crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind::SnapshotIdentityExhausted
        ),
        Admission::Entity(kind) => matches!(
            kind,
            crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::SnapshotIdentityExhausted
        ),
        Admission::Authorization(kind) => matches!(
            kind,
            crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::SnapshotIdentityExhausted
        ),
        Admission::Projection(kind) => matches!(
            kind,
            crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenialKind::Authorization(
                crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::SnapshotIdentityExhausted
            ) | crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenialKind::InvariantAdmission(
                crate::domain_computation::primary_graph::WorthQueryInvariantProjectionDenialKind::SnapshotIdentityExhausted
            )
        ),
        Admission::Invariant(kind) => matches!(
            kind,
            crate::domain_computation::primary_graph::WorthQueryInvariantProjectionDenialKind::SnapshotIdentityExhausted
        ),
    }
}

fn terminal(
    decision: &WorthQueryRetainedConditionalDecision,
) -> WorthQueryConditionalExecutionTerminal {
    match decision {
        WorthQueryRetainedConditionalDecision::Eligible(_) => {
            WorthQueryConditionalExecutionTerminal::EligibleRetained
        }
        WorthQueryRetainedConditionalDecision::Suppressed(_) => {
            WorthQueryConditionalExecutionTerminal::SuppressedRetained
        }
        WorthQueryRetainedConditionalDecision::Deferred(_) => {
            WorthQueryConditionalExecutionTerminal::DeferredRetained
        }
        WorthQueryRetainedConditionalDecision::OperationRetryable(_, _) => {
            WorthQueryConditionalExecutionTerminal::Retryable
        }
        WorthQueryRetainedConditionalDecision::OperationBackpressured(_, _) => {
            WorthQueryConditionalExecutionTerminal::DeferredRetained
        }
        WorthQueryRetainedConditionalDecision::OperationControlStopped(_, _) => {
            WorthQueryConditionalExecutionTerminal::ControlStopped
        }
        WorthQueryRetainedConditionalDecision::OperationTerminalFailure(_, _) => {
            WorthQueryConditionalExecutionTerminal::Failed
        }
        WorthQueryRetainedConditionalDecision::OperationIndeterminate(_, _) => {
            WorthQueryConditionalExecutionTerminal::Indeterminate
        }
        WorthQueryRetainedConditionalDecision::OperationProductUnpublished(_, _) => {
            WorthQueryConditionalExecutionTerminal::ProductUnpublished
        }
        WorthQueryRetainedConditionalDecision::OperationProductStale(_, _) => {
            WorthQueryConditionalExecutionTerminal::ProductStale
        }
        WorthQueryRetainedConditionalDecision::OperationSettlementDeferred(_, _) => {
            WorthQueryConditionalExecutionTerminal::DeferredRetained
        }
        WorthQueryRetainedConditionalDecision::OperationCommitted(_) => {
            WorthQueryConditionalExecutionTerminal::Committed
        }
        WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(_) => {
            WorthQueryConditionalExecutionTerminal::AlreadyCommitted
        }
        WorthQueryRetainedConditionalDecision::Failed(_) => {
            WorthQueryConditionalExecutionTerminal::Failed
        }
    }
}

pub(super) fn signal_decision(
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
