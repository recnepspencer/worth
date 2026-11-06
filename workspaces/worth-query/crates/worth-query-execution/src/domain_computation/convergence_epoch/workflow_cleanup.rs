use std::marker::PhantomData;

use super::core::WorthQueryConvergenceEpochCore;
use crate::domain_computation::{
    WorthQueryWorkflowRunCleanupFailure, WorthQueryWorkflowRunCleanupOutcome,
    WorthQueryWorkflowRunCleanupPending,
};
use {
    super::WorthQueryBoundConvergenceReport, super::WorthQueryConvergenceEpochCounters,
    super::WorthQueryConvergenceIndeterminateCause, super::WorthQueryConvergenceTerminalKind,
    super::WorthQueryConvergenceTerminalState, super::WorthQueryWorkflowConvergenceTerminal,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryRetainedConvergenceCandidateEvidence,
};

#[must_use = "workflow convergence cleanup must resolve Complete, Pending, or RecoveryRequired"]
pub enum WorthQueryWorkflowConvergenceCleanupOutcome<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    Complete(WorthQueryWorkflowConvergenceCleanupReceipt<State>),
    Pending(WorthQueryWorkflowConvergenceCleanupPending<State>),
    RecoveryRequired(WorthQueryWorkflowConvergenceCleanupFailure<State>),
}

impl<State> WorthQueryWorkflowConvergenceCleanupOutcome<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        match self {
            Self::Complete(receipt) => receipt.counters(),
            Self::Pending(pending) => pending.counters(),
            Self::RecoveryRequired(failure) => failure.counters(),
        }
    }
}

pub struct WorthQueryWorkflowConvergenceCleanupReceipt<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    core: WorthQueryConvergenceEpochCore,
    indeterminate_cause: Option<WorthQueryConvergenceIndeterminateCause>,
    terminal_state: PhantomData<State>,
}

impl<State> WorthQueryWorkflowConvergenceCleanupReceipt<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    pub fn identity(&self) -> &str {
        self.core.identity()
    }

    pub const fn kind(&self) -> WorthQueryConvergenceTerminalKind {
        State::KIND
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.core.counters()
    }

    pub fn incumbents(&self) -> &[WorthQueryRetainedConvergenceCandidateEvidence] {
        self.core.incumbents()
    }

    pub fn latest_report(&self) -> Option<&WorthQueryBoundConvergenceReport> {
        self.core.latest_report()
    }

    pub fn indeterminate_cause(&self) -> Option<&WorthQueryConvergenceIndeterminateCause> {
        self.indeterminate_cause.as_ref()
    }
}

#[must_use = "workflow convergence cleanup pending retains exact retry authority"]
pub struct WorthQueryWorkflowConvergenceCleanupPending<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    core: WorthQueryConvergenceEpochCore,
    indeterminate_cause: Option<WorthQueryConvergenceIndeterminateCause>,
    run_cleanup_pending: WorthQueryWorkflowRunCleanupPending,
    terminal_state: PhantomData<State>,
}

impl<State> WorthQueryWorkflowConvergenceCleanupPending<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    pub fn identity(&self) -> &str {
        self.core.identity()
    }

    pub const fn kind(&self) -> WorthQueryConvergenceTerminalKind {
        State::KIND
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.core.counters()
    }

    pub fn incumbents(&self) -> &[WorthQueryRetainedConvergenceCandidateEvidence] {
        self.core.incumbents()
    }

    pub fn latest_report(&self) -> Option<&WorthQueryBoundConvergenceReport> {
        self.core.latest_report()
    }

    pub fn indeterminate_cause(&self) -> Option<&WorthQueryConvergenceIndeterminateCause> {
        self.indeterminate_cause.as_ref()
    }

    #[must_use = "retry returns the next workflow convergence cleanup outcome"]
    pub fn retry(self) -> WorthQueryWorkflowConvergenceCleanupOutcome<State> {
        let Self {
            mut core,
            indeterminate_cause,
            run_cleanup_pending,
            terminal_state,
        } = self;
        core.record_lifecycle_event(WorkflowTerminalCleanupLifecycleEvent::attempted());
        admit_workflow_cleanup_outcome(
            core,
            indeterminate_cause,
            terminal_state,
            run_cleanup_pending.retry(),
        )
    }
}

#[must_use = "workflow convergence cleanup failure retains exact retry authority"]
pub struct WorthQueryWorkflowConvergenceCleanupFailure<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    core: WorthQueryConvergenceEpochCore,
    indeterminate_cause: Option<WorthQueryConvergenceIndeterminateCause>,
    run_cleanup_failure: WorthQueryWorkflowRunCleanupFailure,
    terminal_state: PhantomData<State>,
}

impl<State> WorthQueryWorkflowConvergenceCleanupFailure<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    pub fn identity(&self) -> &str {
        self.core.identity()
    }

    pub const fn kind(&self) -> WorthQueryConvergenceTerminalKind {
        State::KIND
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.core.counters()
    }

    pub fn incumbents(&self) -> &[WorthQueryRetainedConvergenceCandidateEvidence] {
        self.core.incumbents()
    }

    pub fn latest_report(&self) -> Option<&WorthQueryBoundConvergenceReport> {
        self.core.latest_report()
    }

    pub fn indeterminate_cause(&self) -> Option<&WorthQueryConvergenceIndeterminateCause> {
        self.indeterminate_cause.as_ref()
    }

    #[must_use = "retry returns the next workflow convergence cleanup outcome"]
    pub fn retry(self) -> WorthQueryWorkflowConvergenceCleanupOutcome<State> {
        let Self {
            mut core,
            indeterminate_cause,
            run_cleanup_failure,
            terminal_state,
        } = self;
        core.record_lifecycle_event(WorkflowTerminalCleanupLifecycleEvent::attempted());
        admit_workflow_cleanup_outcome(
            core,
            indeterminate_cause,
            terminal_state,
            run_cleanup_failure.retry(),
        )
    }
}

impl<State> WorthQueryWorkflowConvergenceTerminal<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    #[must_use = "cleanup returns workflow convergence authority that must be resolved"]
    pub fn cleanup(self) -> WorthQueryWorkflowConvergenceCleanupOutcome<State> {
        let WorthQueryWorkflowConvergenceTerminal {
            mut core,
            run_terminal,
            indeterminate_cause,
            terminal_state,
        } = self;
        core.record_lifecycle_event(WorkflowTerminalCleanupLifecycleEvent::attempted());
        admit_workflow_cleanup_outcome(
            core,
            indeterminate_cause,
            terminal_state,
            run_terminal.cleanup(),
        )
    }
}

fn admit_workflow_cleanup_outcome<State>(
    mut core: WorthQueryConvergenceEpochCore,
    indeterminate_cause: Option<WorthQueryConvergenceIndeterminateCause>,
    terminal_state: PhantomData<State>,
    outcome: WorthQueryWorkflowRunCleanupOutcome,
) -> WorthQueryWorkflowConvergenceCleanupOutcome<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    match outcome {
        WorthQueryWorkflowRunCleanupOutcome::Complete(_run_cleanup_receipt) => {
            core.record_lifecycle_event(WorkflowTerminalCleanupLifecycleEvent::completed());
            WorthQueryWorkflowConvergenceCleanupOutcome::Complete(
                WorthQueryWorkflowConvergenceCleanupReceipt {
                    core,
                    indeterminate_cause,
                    terminal_state,
                },
            )
        }
        WorthQueryWorkflowRunCleanupOutcome::Pending(run_cleanup_pending) => {
            WorthQueryWorkflowConvergenceCleanupOutcome::Pending(
                WorthQueryWorkflowConvergenceCleanupPending {
                    core,
                    indeterminate_cause,
                    run_cleanup_pending,
                    terminal_state,
                },
            )
        }
        WorthQueryWorkflowRunCleanupOutcome::RecoveryRequired(run_cleanup_failure) => {
            WorthQueryWorkflowConvergenceCleanupOutcome::RecoveryRequired(
                WorthQueryWorkflowConvergenceCleanupFailure {
                    core,
                    indeterminate_cause,
                    run_cleanup_failure,
                    terminal_state,
                },
            )
        }
    }
}

pub(in crate::domain_computation::convergence_epoch) struct WorkflowTerminalCleanupLifecycleEvent {
    kind: WorkflowTerminalCleanupLifecycleEventKind,
}

pub(in crate::domain_computation::convergence_epoch) enum WorkflowTerminalCleanupLifecycleEventKind
{
    Attempted,
    Completed,
}

impl WorkflowTerminalCleanupLifecycleEvent {
    fn attempted() -> Self {
        Self {
            kind: WorkflowTerminalCleanupLifecycleEventKind::Attempted,
        }
    }

    fn completed() -> Self {
        Self {
            kind: WorkflowTerminalCleanupLifecycleEventKind::Completed,
        }
    }

    pub(in crate::domain_computation::convergence_epoch) fn into_kind(
        self,
    ) -> WorkflowTerminalCleanupLifecycleEventKind {
        self.kind
    }
}
