use super::application_operation_reentry as reentry;
use super::signal_decision_reentry::{
    WorthQueryRetainedConditionalDecision, WorthQueryRetainedConditionalWake,
};
use crate::domain_computation::primary_graph as graph;
use classification::WorthQueryConditionalExecutionCause as Cause;
use graph::{
    WorthQueryInvariantProjectionDenialKind as InvariantKind,
    WorthQueryOperationAuthorizationDenialKind as AuthorizationKind,
    WorthQueryPrincipalResolutionDenialKind as PrincipalKind,
};

mod classification;
mod commit_cause;
pub(in crate::domain_computation::primary_graph::conditional_operation) use classification::signal_decision;
pub use classification::{
    WorthQueryConditionalExecutionCause, WorthQueryConditionalExecutionTerminal,
    WorthQueryConditionalSignalDecision,
};
use commit_cause::application_commit_cause;

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
        self.cause.clone()
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
    use super::signal_decision_reentry::WorthQueryRetainedConditionalDecision as Decision;
    use graph::{
        WorthQueryApplicationCommitDenialKind as CommitKind,
        WorthQueryApplicationCommitDenialStage as Stage,
    };
    match decision {
        Decision::OperationExecutionControlRetryable(_, kind) => {
            Some(Cause::ApplicationExecutionDenied {
                stage: Stage::ProviderCommit,
                cause: Err(*kind),
            })
        }
        Decision::OperationTerminalFailure(_, Terminal::ApplicationExecution { stage, cause }) => {
            Some(Cause::ApplicationExecutionDenied {
                stage: *stage,
                cause: *cause,
            })
        }
        Decision::OperationCommitRetryable(_, kind) => Some(Cause::ApplicationCommitDenied(*kind)),
        Decision::OperationSettlementExecutionDenied(_, _, kind) => {
            Some(Cause::SettlementExecutionDenied(*kind))
        }
        Decision::OperationProductStale(_, _) => Some(Cause::ProductHeadChanged),
        Decision::OperationNoEffect(_, cause) => Some(Cause::NoEffect(*cause)),
        Decision::OperationBackpressured(_, cause) => match cause {
            Backpressure::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Some(Cause::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: *maximum_active_snapshots,
            }),
            Backpressure::RetentionCapacityExhausted => Some(Cause::RetentionCapacityExhausted),
            Backpressure::ProviderCommit(kind) => provider_commit_cause(kind.clone()),
        },
        Decision::OperationControlStopped(_, Control::Cancelled) => Some(Cause::Cancelled),
        Decision::OperationControlStopped(_, Control::TimedOut) => Some(Cause::TimedOut),
        Decision::OperationTerminalFailure(
            _,
            Terminal::ApplicationCommit(CommitKind::RetentionIdentityExhausted),
        ) => Some(Cause::RetentionIdentityExhausted),
        Decision::OperationTerminalFailure(_, Terminal::Admission(failure))
            if admission_retention_identity_exhausted(*failure) =>
        {
            Some(Cause::RetentionIdentityExhausted)
        }
        Decision::OperationTerminalFailure(
            _,
            Terminal::ApplicationCommit(CommitKind::SnapshotIdentityExhausted),
        ) => Some(Cause::SnapshotIdentityExhausted),
        Decision::OperationTerminalFailure(_, Terminal::Admission(failure))
            if admission_snapshot_identity_exhausted(*failure) =>
        {
            Some(Cause::SnapshotIdentityExhausted)
        }
        Decision::OperationTerminalFailure(
            _,
            Terminal::Admission(
                Admission::Principal(_)
                | Admission::Entity(_)
                | Admission::Authorization(_)
                | Admission::Projection(_)
                | Admission::Invariant(_),
            ),
        ) => Some(Cause::TerminalFailure),
        Decision::OperationTerminalFailure(_, Terminal::ApplicationCommit(kind)) => {
            Some(application_commit_cause(*kind))
        }
        Decision::Eligible(_)
        | Decision::Suppressed(_)
        | Decision::Deferred(_)
        | Decision::OperationProductUnpublished(_, _)
        | Decision::OperationSettlementDeferred(_, _)
        | Decision::OperationCommitted(_)
        | Decision::OperationAlreadyCommitted(_) => None,
        Decision::OperationRetryable(_, cause) | Decision::OperationIndeterminate(_, cause) => {
            Some(Cause::Reentry(cause.clone()))
        }
        Decision::Failed(denial) => Some(Cause::BridgeConditional(denial.kind())),
        Decision::InterruptedDuringReentry => Some(Cause::InterruptedDuringReentry),
    }
}

fn provider_commit_cause(kind: graph::WorthQueryApplicationCommitDeferredKind) -> Option<Cause> {
    use graph::WorthQueryApplicationCommitDeferredKind as Kind;
    match kind {
        Kind::RelationalDeferred(deferred) => Some(Cause::RelationalDeferred(deferred)),
        Kind::RetentionCapacityExhausted => Some(Cause::RetentionCapacityExhausted),
        Kind::PatchPositionReservationContended => Some(Cause::PatchPositionReservationContended),
        Kind::CandidateCapacityExhausted { maximum_candidates } => {
            Some(Cause::CandidateCapacityExhausted { maximum_candidates })
        }
        Kind::PublishedSnapshotCapacityExhausted { maximum_handles } => {
            Some(Cause::PublishedSnapshotCapacityExhausted { maximum_handles })
        }
        Kind::SourceCurrentnessRaced(stop) => Some(Cause::SourceCurrentnessRaced(stop)),
        Kind::RequiredPrerequisitePending(kind) => Some(Cause::RequiredPrerequisitePending(kind)),
        Kind::CandidateLifetimeExpired { .. } => None,
    }
}

fn admission_retention_identity_exhausted(
    failure: super::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure,
) -> bool {
    use reentry::WorthQueryTemporalAdmissionTerminalFailure as Admission;
    match failure {
        Admission::Principal(kind) => matches!(kind, PrincipalKind::RetentionIdentityExhausted),
        Admission::Entity(kind) => matches!(
            kind,
            graph::WorthQueryEntityResolutionDenialKind::RetentionIdentityExhausted
        ),
        Admission::Authorization(kind) => {
            matches!(kind, AuthorizationKind::RetentionIdentityExhausted)
        }
        Admission::Projection(kind) => matches!(
            kind,
            graph::WorthQueryOperationProjectionDenialKind::Authorization(
                AuthorizationKind::RetentionIdentityExhausted
            ) | graph::WorthQueryOperationProjectionDenialKind::InvariantAdmission(
                InvariantKind::RetentionIdentityExhausted
            )
        ),
        Admission::Invariant(kind) => matches!(kind, InvariantKind::RetentionIdentityExhausted),
    }
}

fn admission_snapshot_identity_exhausted(
    failure: super::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure,
) -> bool {
    use reentry::WorthQueryTemporalAdmissionTerminalFailure as Admission;
    match failure {
        Admission::Principal(kind) => matches!(kind, PrincipalKind::SnapshotIdentityExhausted),
        Admission::Entity(kind) => matches!(
            kind,
            graph::WorthQueryEntityResolutionDenialKind::SnapshotIdentityExhausted
        ),
        Admission::Authorization(kind) => {
            matches!(kind, AuthorizationKind::SnapshotIdentityExhausted)
        }
        Admission::Projection(kind) => matches!(
            kind,
            graph::WorthQueryOperationProjectionDenialKind::Authorization(
                AuthorizationKind::SnapshotIdentityExhausted
            ) | graph::WorthQueryOperationProjectionDenialKind::InvariantAdmission(
                InvariantKind::SnapshotIdentityExhausted
            )
        ),
        Admission::Invariant(kind) => matches!(kind, InvariantKind::SnapshotIdentityExhausted),
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
        WorthQueryRetainedConditionalDecision::OperationExecutionControlRetryable(_, _)
        | WorthQueryRetainedConditionalDecision::OperationCommitRetryable(_, _)
        | WorthQueryRetainedConditionalDecision::OperationRetryable(_, _) => {
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
        WorthQueryRetainedConditionalDecision::OperationNoEffect(_, _) => {
            WorthQueryConditionalExecutionTerminal::NoEffect
        }
        WorthQueryRetainedConditionalDecision::OperationSettlementExecutionDenied(_, _, _)
        | WorthQueryRetainedConditionalDecision::OperationSettlementDeferred(_, _) => {
            WorthQueryConditionalExecutionTerminal::DeferredRetained
        }
        WorthQueryRetainedConditionalDecision::OperationCommitted(_) => {
            WorthQueryConditionalExecutionTerminal::Committed
        }
        WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(_) => {
            WorthQueryConditionalExecutionTerminal::AlreadyCommitted
        }
        WorthQueryRetainedConditionalDecision::Failed(_)
        | WorthQueryRetainedConditionalDecision::InterruptedDuringReentry => {
            WorthQueryConditionalExecutionTerminal::Failed
        }
    }
}
