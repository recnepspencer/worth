//! Borrow only a captured domain refusal; resource owners retain the other cases.
use crate::domain_computation::vocabulary::primary_graph::admitted_capture::CaptureStop;
pub(super) fn captured<E>(stop: &CaptureStop<E>) -> Option<&E> {
    match stop {
        CaptureStop::Kernel(worth_execution::ScopeStop::Failure(
            worth_execution::MapKernelFailure::Domain(cause),
        )) => Some(cause),
        CaptureStop::Request(_)
        | CaptureStop::Kernel(worth_execution::ScopeStop::Admission(_))
        | CaptureStop::Kernel(worth_execution::ScopeStop::Failure(
            worth_execution::MapKernelFailure::Stop(_)
            | worth_execution::MapKernelFailure::Panic
            | worth_execution::MapKernelFailure::ResultCapacityExceeded,
        )) => None,
    }
}
