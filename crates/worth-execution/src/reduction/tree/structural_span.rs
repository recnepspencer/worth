//! Transport failed metric coverage without turning it into a request counter fact.
use super::{ReductionDenial, ReductionMetrics, ReductionRunFailure, ReductionRunStop};
use crate::backend::{KernelStop, StructuralSpanFailure};

impl ReductionRunFailure<KernelStop> {
    pub(crate) fn structural_span_failure(
        failure: StructuralSpanFailure,
        metrics: ReductionMetrics,
    ) -> Self {
        let reason = match failure {
            // The meter already latched the arithmetic overflow at its producer.
            StructuralSpanFailure::CounterOverflow => ReductionRunStop::WorkCounterOverflow,
            StructuralSpanFailure::MetricsMismatch => {
                ReductionRunStop::Denial(ReductionDenial::CoverageMismatch)
            }
        };
        Self { reason, metrics }
    }
}
