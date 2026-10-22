//! A computation's metric proof is distinct from overflowing the request counter.
use super::{KernelContext, NestedStopOwner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StructuralSpanFailure {
    CounterOverflow,
    MetricsMismatch,
}

impl KernelContext<'_, '_> {
    /// The work and structural span already admitted on this kernel.
    pub fn charged_cost(&self) -> (u64, u64) {
        if let Some(capture) = self.meter.borrow_mut().capture_mut() {
            capture.invalidate();
        }
        assert!(
            self.has_accounting(),
            "charged cost requires accounted execution"
        );
        self.cost()
    }

    /// Admit a retained or declared stage's work and its dependency span.
    /// The same checkpoint enforces the work ceiling before either advances.
    pub fn checkpoint_cost(&mut self, work: u64, span: u64) -> Result<(), super::KernelStop> {
        if span > work {
            return Err(super::KernelStop::work_counter_overflow());
        }
        let before = self.cost();
        self.checkpoint(work)?;
        self.apply_structural_span(before, work, span)
            .map_err(|_| super::KernelStop::work_counter_overflow())
    }

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
        let Some(accounting) = meter.accounting.as_mut() else {
            return Err(StructuralSpanFailure::MetricsMismatch);
        };
        if accounting.work != expected_work
            || accounting.span != expected_span
            || stage_span > stage_work
        {
            return Err(StructuralSpanFailure::MetricsMismatch);
        }
        accounting.span = next_span;
        Ok(())
    }
}
