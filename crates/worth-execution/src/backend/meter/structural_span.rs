//! A computation's metric proof is distinct from overflowing the request counter.
use super::{KernelContext, NestedStopOwner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StructuralSpanFailure {
    CounterOverflow,
    MetricsMismatch,
}

impl KernelContext<'_, '_> {
    /// Replace a completed stage's sequential span with its proven dependency
    /// span. Work remains the exact checkpoint count.
    pub(crate) fn apply_structural_span(
        &mut self,
        before: (u64, u64),
        stage_work: u64,
        stage_span: u64,
    ) -> Result<(), StructuralSpanFailure> {
        let mut meter = self.meter.borrow_mut();
        let sums = before
            .0
            .checked_add(stage_work)
            .zip(before.1.checked_add(stage_work))
            .zip(before.1.checked_add(stage_span));
        let Some(((expected_work, expected_span), next_span)) = sums else {
            meter.nested_stop.record(NestedStopOwner::Request);
            return Err(StructuralSpanFailure::CounterOverflow);
        };
        if meter.work != expected_work || meter.span != expected_span || stage_span > stage_work {
            return Err(StructuralSpanFailure::MetricsMismatch);
        }
        meter.span = next_span;
        Ok(())
    }
}
