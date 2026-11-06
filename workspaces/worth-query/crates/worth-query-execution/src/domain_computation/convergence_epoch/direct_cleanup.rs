use std::marker::PhantomData;

use super::core::WorthQueryConvergenceEpochCore;
use crate::domain_computation::WorthQueryDirectRunCleanupFailure;
use {
    super::WorthQueryBoundConvergenceReport, super::WorthQueryConvergenceEpochCounters,
    super::WorthQueryConvergenceIndeterminateCause, super::WorthQueryConvergenceTerminalKind,
    super::WorthQueryConvergenceTerminalState, super::WorthQueryDirectConvergenceTerminal,
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryRetainedConvergenceCandidateEvidence,
};

impl<State> WorthQueryDirectConvergenceTerminal<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    pub fn cleanup(
        self,
    ) -> Result<
        WorthQueryDirectConvergenceCleanupReceipt<State>,
        WorthQueryDirectConvergenceCleanupFailure<State>,
    > {
        let WorthQueryDirectConvergenceTerminal {
            mut core,
            run_terminal,
            indeterminate_cause,
            terminal_state,
        } = self;
        core.record_lifecycle_event(DirectTerminalCleanupLifecycleEvent::attempted());
        match run_terminal.cleanup() {
            Ok(_run_cleanup_receipt) => {
                core.record_lifecycle_event(DirectTerminalCleanupLifecycleEvent::completed());
                Ok(WorthQueryDirectConvergenceCleanupReceipt {
                    core,
                    indeterminate_cause,
                    terminal_state,
                })
            }
            Err(run_cleanup_failure) => Err(WorthQueryDirectConvergenceCleanupFailure {
                core,
                indeterminate_cause,
                run_cleanup_failure,
                terminal_state,
            }),
        }
    }
}

pub(in crate::domain_computation::convergence_epoch) struct DirectTerminalCleanupLifecycleEvent {
    kind: DirectTerminalCleanupLifecycleEventKind,
}

pub(in crate::domain_computation::convergence_epoch) enum DirectTerminalCleanupLifecycleEventKind {
    Attempted,
    Completed,
}

impl DirectTerminalCleanupLifecycleEvent {
    fn attempted() -> Self {
        Self {
            kind: DirectTerminalCleanupLifecycleEventKind::Attempted,
        }
    }

    fn completed() -> Self {
        Self {
            kind: DirectTerminalCleanupLifecycleEventKind::Completed,
        }
    }

    pub(in crate::domain_computation::convergence_epoch) fn into_kind(
        self,
    ) -> DirectTerminalCleanupLifecycleEventKind {
        self.kind
    }
}

pub struct WorthQueryDirectConvergenceCleanupReceipt<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    core: WorthQueryConvergenceEpochCore,
    indeterminate_cause: Option<WorthQueryConvergenceIndeterminateCause>,
    terminal_state: PhantomData<State>,
}

impl<State> WorthQueryDirectConvergenceCleanupReceipt<State>
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

pub struct WorthQueryDirectConvergenceCleanupFailure<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    core: WorthQueryConvergenceEpochCore,
    indeterminate_cause: Option<WorthQueryConvergenceIndeterminateCause>,
    run_cleanup_failure: WorthQueryDirectRunCleanupFailure,
    terminal_state: PhantomData<State>,
}

impl<State> WorthQueryDirectConvergenceCleanupFailure<State>
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

    pub fn retry(self) -> Result<WorthQueryDirectConvergenceCleanupReceipt<State>, Self> {
        WorthQueryDirectConvergenceTerminal::<State>::new(
            self.core,
            self.run_cleanup_failure.into_terminal(),
            self.indeterminate_cause,
        )
        .cleanup()
    }
}
