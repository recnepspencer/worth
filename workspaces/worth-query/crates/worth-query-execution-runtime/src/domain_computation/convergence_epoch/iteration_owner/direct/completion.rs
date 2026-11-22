//! Direct-lane completion outcomes and their consuming continuation transition.

use super::super::{
    WorthQueryCancelled, WorthQueryConverged, WorthQueryDirectConvergenceTerminal,
    WorthQueryExhausted, WorthQueryFeasibleIncumbent, WorthQueryIndeterminate,
    WorthQueryOscillating, WorthQueryStableWithoutProof,
};
use super::WorthQueryIteratingDirectConvergenceEpoch;

pub enum WorthQueryDirectConvergenceIterationOutcome {
    Continue(WorthQueryIteratingDirectConvergenceEpoch),
    Converged(WorthQueryDirectConvergenceTerminal<WorthQueryConverged>),
    StableWithoutProof(WorthQueryDirectConvergenceTerminal<WorthQueryStableWithoutProof>),
    FeasibleIncumbent(WorthQueryDirectConvergenceTerminal<WorthQueryFeasibleIncumbent>),
    Oscillating(WorthQueryDirectConvergenceTerminal<WorthQueryOscillating>),
    Exhausted(WorthQueryDirectConvergenceTerminal<WorthQueryExhausted>),
    Cancelled(WorthQueryDirectConvergenceTerminal<WorthQueryCancelled>),
    Indeterminate(WorthQueryDirectConvergenceTerminal<WorthQueryIndeterminate>),
}
