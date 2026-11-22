use super::core::WorthQueryConvergenceEpochCore;
use super::{
    WorthQueryCancelled, WorthQueryConverged, WorthQueryConvergenceDisposition,
    WorthQueryConvergenceIndeterminateCause, WorthQueryConvergenceTerminalKind,
    WorthQueryDirectConvergenceIterationOutcome, WorthQueryDirectConvergenceTerminal,
    WorthQueryExhausted, WorthQueryFeasibleIncumbent, WorthQueryIndeterminate,
    WorthQueryOscillating, WorthQueryStableWithoutProof,
    WorthQueryWorkflowConvergenceIterationOutcome, WorthQueryWorkflowConvergenceTerminal,
};
use crate::domain_computation::{WorthQueryDirectRunTerminal, WorthQueryWorkflowRunTerminal};

pub(super) const fn semantic_terminal_kind(
    disposition: WorthQueryConvergenceDisposition,
) -> WorthQueryConvergenceTerminalKind {
    match disposition {
        WorthQueryConvergenceDisposition::Converged => WorthQueryConvergenceTerminalKind::Converged,
        WorthQueryConvergenceDisposition::StableWithoutProof => {
            WorthQueryConvergenceTerminalKind::StableWithoutProof
        }
        WorthQueryConvergenceDisposition::FeasibleIncumbent => {
            WorthQueryConvergenceTerminalKind::FeasibleIncumbent
        }
        WorthQueryConvergenceDisposition::Oscillating => {
            WorthQueryConvergenceTerminalKind::Oscillating
        }
        WorthQueryConvergenceDisposition::Indeterminate
        | WorthQueryConvergenceDisposition::Continue => {
            WorthQueryConvergenceTerminalKind::Indeterminate
        }
    }
}

pub(super) fn direct_terminal_outcome(
    core: WorthQueryConvergenceEpochCore,
    run_terminal: WorthQueryDirectRunTerminal,
    kind: WorthQueryConvergenceTerminalKind,
    indeterminate_cause: Option<WorthQueryConvergenceIndeterminateCause>,
) -> WorthQueryDirectConvergenceIterationOutcome {
    match kind {
        WorthQueryConvergenceTerminalKind::Converged => {
            WorthQueryDirectConvergenceIterationOutcome::Converged(
                WorthQueryDirectConvergenceTerminal::<WorthQueryConverged>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
        WorthQueryConvergenceTerminalKind::StableWithoutProof => {
            WorthQueryDirectConvergenceIterationOutcome::StableWithoutProof(
                WorthQueryDirectConvergenceTerminal::<WorthQueryStableWithoutProof>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
        WorthQueryConvergenceTerminalKind::FeasibleIncumbent => {
            WorthQueryDirectConvergenceIterationOutcome::FeasibleIncumbent(
                WorthQueryDirectConvergenceTerminal::<WorthQueryFeasibleIncumbent>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
        WorthQueryConvergenceTerminalKind::Oscillating => {
            WorthQueryDirectConvergenceIterationOutcome::Oscillating(
                WorthQueryDirectConvergenceTerminal::<WorthQueryOscillating>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
        WorthQueryConvergenceTerminalKind::Exhausted => {
            WorthQueryDirectConvergenceIterationOutcome::Exhausted(
                WorthQueryDirectConvergenceTerminal::<WorthQueryExhausted>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
        WorthQueryConvergenceTerminalKind::Cancelled => {
            WorthQueryDirectConvergenceIterationOutcome::Cancelled(
                WorthQueryDirectConvergenceTerminal::<WorthQueryCancelled>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
        WorthQueryConvergenceTerminalKind::Indeterminate => {
            WorthQueryDirectConvergenceIterationOutcome::Indeterminate(
                WorthQueryDirectConvergenceTerminal::<WorthQueryIndeterminate>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
    }
}

pub(super) fn workflow_terminal_outcome(
    core: WorthQueryConvergenceEpochCore,
    run_terminal: WorthQueryWorkflowRunTerminal,
    kind: WorthQueryConvergenceTerminalKind,
    indeterminate_cause: Option<WorthQueryConvergenceIndeterminateCause>,
) -> WorthQueryWorkflowConvergenceIterationOutcome {
    match kind {
        WorthQueryConvergenceTerminalKind::Converged => {
            WorthQueryWorkflowConvergenceIterationOutcome::Converged(
                WorthQueryWorkflowConvergenceTerminal::<WorthQueryConverged>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
        WorthQueryConvergenceTerminalKind::StableWithoutProof => {
            WorthQueryWorkflowConvergenceIterationOutcome::StableWithoutProof(
                WorthQueryWorkflowConvergenceTerminal::<WorthQueryStableWithoutProof>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
        WorthQueryConvergenceTerminalKind::FeasibleIncumbent => {
            WorthQueryWorkflowConvergenceIterationOutcome::FeasibleIncumbent(
                WorthQueryWorkflowConvergenceTerminal::<WorthQueryFeasibleIncumbent>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
        WorthQueryConvergenceTerminalKind::Oscillating => {
            WorthQueryWorkflowConvergenceIterationOutcome::Oscillating(
                WorthQueryWorkflowConvergenceTerminal::<WorthQueryOscillating>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
        WorthQueryConvergenceTerminalKind::Exhausted => {
            WorthQueryWorkflowConvergenceIterationOutcome::Exhausted(
                WorthQueryWorkflowConvergenceTerminal::<WorthQueryExhausted>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
        WorthQueryConvergenceTerminalKind::Cancelled => {
            WorthQueryWorkflowConvergenceIterationOutcome::Cancelled(
                WorthQueryWorkflowConvergenceTerminal::<WorthQueryCancelled>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
        WorthQueryConvergenceTerminalKind::Indeterminate => {
            WorthQueryWorkflowConvergenceIterationOutcome::Indeterminate(
                WorthQueryWorkflowConvergenceTerminal::<WorthQueryIndeterminate>::new(
                    core,
                    run_terminal,
                    indeterminate_cause,
                ),
            )
        }
    }
}
