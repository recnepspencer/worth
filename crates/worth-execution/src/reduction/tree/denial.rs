use super::super::ReductionDenial;
use super::{ReductionMetrics, ReductionRunFailure, ReductionRunStop};

pub(super) fn failure<E>(reason: ReductionRunStop<E>) -> ReductionRunFailure<E> {
    ReductionRunFailure {
        reason,
        metrics: ReductionMetrics::default(),
    }
}

pub(super) fn pure_denial(failure: ReductionRunFailure<()>) -> ReductionDenial {
    match failure.reason {
        ReductionRunStop::Denial(denial) => denial,
        ReductionRunStop::Panic => ReductionDenial::ReducerPanic,
        ReductionRunStop::Hook(()) => unreachable!("infallible structural hook"),
        ReductionRunStop::ResultCapacityExceeded => ReductionDenial::ResultCapacityExceeded,
        ReductionRunStop::WorkCounterOverflow => ReductionDenial::WorkCounterOverflow,
    }
}
