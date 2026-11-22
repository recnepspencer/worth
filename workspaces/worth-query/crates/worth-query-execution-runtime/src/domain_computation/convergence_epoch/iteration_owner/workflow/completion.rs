//! Workflow-lane completion outcomes and their consuming continuation transition.

use super::super::{
    WorthQueryCancelled, WorthQueryConverged, WorthQueryExhausted, WorthQueryFeasibleIncumbent,
    WorthQueryIndeterminate, WorthQueryOscillating, WorthQueryStableWithoutProof,
    WorthQueryWorkflowConvergenceTerminal,
};
use super::WorthQueryIteratingWorkflowConvergenceEpoch;

pub enum WorthQueryWorkflowConvergenceIterationOutcome {
    Continue(WorthQueryIteratingWorkflowConvergenceEpoch),
    Converged(WorthQueryWorkflowConvergenceTerminal<WorthQueryConverged>),
    StableWithoutProof(WorthQueryWorkflowConvergenceTerminal<WorthQueryStableWithoutProof>),
    FeasibleIncumbent(WorthQueryWorkflowConvergenceTerminal<WorthQueryFeasibleIncumbent>),
    Oscillating(WorthQueryWorkflowConvergenceTerminal<WorthQueryOscillating>),
    Exhausted(WorthQueryWorkflowConvergenceTerminal<WorthQueryExhausted>),
    Cancelled(WorthQueryWorkflowConvergenceTerminal<WorthQueryCancelled>),
    Indeterminate(WorthQueryWorkflowConvergenceTerminal<WorthQueryIndeterminate>),
}
