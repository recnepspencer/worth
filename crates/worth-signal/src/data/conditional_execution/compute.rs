//! The computation an eligible conditional runs, given the output version
//! its node holds when the attempt begins.
//!
//! An output whose version carries semantic meaning returns that version and
//! is compared as-is. An output that only marks a fresh evaluation can derive
//! its version from the baseline, so it differs from whatever produced the
//! baseline (an earlier evaluation or an applied scoped change) and is never
//! read as a reverted-clean recompute by coincidence.
use crate::data::error::SignalError;
use crate::data::output::NodeEvaluationResult;

/// One conditional computation, consumed by the attempt that runs it.
///
/// Plain closures (`|| ...`) implement it and ignore the baseline;
/// [`SignalConditionalBaselineCompute`] receives it.
pub trait SignalConditionalCompute {
    fn compute(self, output_version_before: u64) -> Result<NodeEvaluationResult, SignalError>;
}

impl<F> SignalConditionalCompute for F
where
    F: FnOnce() -> Result<NodeEvaluationResult, SignalError>,
{
    fn compute(self, _: u64) -> Result<NodeEvaluationResult, SignalError> {
        self()
    }
}

/// A computation that reads the node's output version before the attempt:
/// the value the attempt's output comparator compares the result against.
pub struct SignalConditionalBaselineCompute<F>(pub F);

impl<F> SignalConditionalCompute for SignalConditionalBaselineCompute<F>
where
    F: FnOnce(u64) -> Result<NodeEvaluationResult, SignalError>,
{
    fn compute(self, output_version_before: u64) -> Result<NodeEvaluationResult, SignalError> {
        (self.0)(output_version_before)
    }
}
