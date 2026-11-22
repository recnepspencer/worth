use std::marker::PhantomData;

use super::core::WorthQueryConvergenceEpochCore;
use super::{
    WorthQueryBoundConvergenceReport, WorthQueryConvergenceEpochCounters,
    WorthQueryConvergenceIndeterminateCause, WorthQueryRetainedConvergenceCandidateEvidence,
};
use crate::domain_computation::{WorthQueryDirectRunTerminal, WorthQueryWorkflowRunTerminal};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConvergenceTerminalKind {
    Converged,
    StableWithoutProof,
    FeasibleIncumbent,
    Oscillating,
    Exhausted,
    Cancelled,
    Indeterminate,
}

mod sealed {
    pub trait Sealed {}
}

pub trait WorthQueryConvergenceTerminalState: sealed::Sealed {
    const KIND: WorthQueryConvergenceTerminalKind;
}

macro_rules! convergence_terminal_state {
    ($name:ident, $kind:ident) => {
        #[derive(Debug)]
        pub struct $name;

        impl sealed::Sealed for $name {}

        impl WorthQueryConvergenceTerminalState for $name {
            const KIND: WorthQueryConvergenceTerminalKind =
                WorthQueryConvergenceTerminalKind::$kind;
        }
    };
}

convergence_terminal_state!(WorthQueryConverged, Converged);
convergence_terminal_state!(WorthQueryStableWithoutProof, StableWithoutProof);
convergence_terminal_state!(WorthQueryFeasibleIncumbent, FeasibleIncumbent);
convergence_terminal_state!(WorthQueryOscillating, Oscillating);
convergence_terminal_state!(WorthQueryExhausted, Exhausted);
convergence_terminal_state!(WorthQueryCancelled, Cancelled);
convergence_terminal_state!(WorthQueryIndeterminate, Indeterminate);

pub struct WorthQueryDirectConvergenceTerminal<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    pub(super) core: WorthQueryConvergenceEpochCore,
    pub(super) run_terminal: WorthQueryDirectRunTerminal,
    pub(super) indeterminate_cause: Option<WorthQueryConvergenceIndeterminateCause>,
    pub(super) terminal_state: PhantomData<State>,
}

impl<State> WorthQueryDirectConvergenceTerminal<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    pub(super) fn new(
        core: WorthQueryConvergenceEpochCore,
        run_terminal: WorthQueryDirectRunTerminal,
        indeterminate_cause: Option<WorthQueryConvergenceIndeterminateCause>,
    ) -> Self {
        Self {
            core,
            run_terminal,
            indeterminate_cause,
            terminal_state: PhantomData,
        }
    }

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

#[must_use = "a workflow convergence terminal retains resources that must be cleaned up"]
pub struct WorthQueryWorkflowConvergenceTerminal<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    pub(super) core: WorthQueryConvergenceEpochCore,
    pub(super) run_terminal: WorthQueryWorkflowRunTerminal,
    pub(super) indeterminate_cause: Option<WorthQueryConvergenceIndeterminateCause>,
    pub(super) terminal_state: PhantomData<State>,
}

impl<State> WorthQueryWorkflowConvergenceTerminal<State>
where
    State: WorthQueryConvergenceTerminalState,
{
    pub(super) fn new(
        core: WorthQueryConvergenceEpochCore,
        run_terminal: WorthQueryWorkflowRunTerminal,
        indeterminate_cause: Option<WorthQueryConvergenceIndeterminateCause>,
    ) -> Self {
        Self {
            core,
            run_terminal,
            indeterminate_cause,
            terminal_state: PhantomData,
        }
    }

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
