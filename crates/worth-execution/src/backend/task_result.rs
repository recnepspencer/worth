use super::meter::{KernelFailure, KernelStop, NestedComputationStop, NestedStopOwner, RunLimits};

/// The settled result carries its stop owner in the existing result discriminant.
/// No per-partition side allocation or second failure latch is needed.
pub(crate) enum TaskResult<R, E> {
    Value(R),
    Domain(E),
    Panic,
    Capacity,
    Stop(KernelStop),
    ComputationalStop(NestedComputationStop),
    RequestDomain(E),
    RequestPanic,
    RequestCapacity,
    RequestCeiling,
}

impl<R, E> TaskResult<R, E> {
    pub(super) fn new(result: Result<R, KernelFailure<E>>, owner: Option<NestedStopOwner>) -> Self {
        let request = owner == Some(NestedStopOwner::Request);
        match result {
            Ok(value) => Self::Value(value),
            Err(KernelFailure::Domain(value)) if request => Self::RequestDomain(value),
            Err(KernelFailure::Domain(value)) => Self::Domain(value),
            Err(KernelFailure::Panic) if request => Self::RequestPanic,
            Err(KernelFailure::Panic) => Self::Panic,
            Err(KernelFailure::ResultCapacityExceeded) if request => Self::RequestCapacity,
            Err(KernelFailure::ResultCapacityExceeded) => Self::Capacity,
            Err(KernelFailure::Stop(KernelStop::NestedStopped)) => match owner {
                Some(NestedStopOwner::Computation(cause)) => Self::ComputationalStop(cause),
                Some(NestedStopOwner::Request) | None => Self::Stop(KernelStop::NestedStopped),
            },
            Err(KernelFailure::Stop(KernelStop::WorkCeiling)) if request => Self::RequestCeiling,
            Err(KernelFailure::Stop(stop)) => Self::Stop(stop),
        }
    }

    pub(super) fn into_parts(
        self,
        limits: &RunLimits,
    ) -> (Result<R, KernelFailure<E>>, Option<NestedStopOwner>) {
        use NestedComputationStop as Cause;
        use NestedStopOwner::{Computation, Request};
        let (failure, owner) = match self {
            Self::Value(value) => return (Ok(value), None),
            Self::Domain(value) => (
                KernelFailure::Domain(value),
                Computation(Cause::DomainFailure),
            ),
            Self::Panic => (KernelFailure::Panic, Computation(Cause::Panic)),
            Self::Capacity => (
                KernelFailure::ResultCapacityExceeded,
                Computation(Cause::ResultCapacityExceeded),
            ),
            Self::RequestDomain(value) => (KernelFailure::Domain(value), Request),
            Self::RequestPanic => (KernelFailure::Panic, Request),
            Self::RequestCapacity => (KernelFailure::ResultCapacityExceeded, Request),
            Self::RequestCeiling => (KernelFailure::Stop(KernelStop::WorkCeiling), Request),
            Self::ComputationalStop(cause) => (
                KernelFailure::Stop(KernelStop::NestedStopped),
                Computation(cause),
            ),
            Self::Stop(stop) => (
                KernelFailure::Stop(stop),
                match stop {
                    KernelStop::WorkCeiling => limits.work_stop_owner(),
                    KernelStop::Cancelled
                    | KernelStop::DeadlineElapsed
                    | KernelStop::WorkCounterOverflow
                    | KernelStop::NestedStopped => Request,
                },
            ),
        };
        (Err(failure), Some(owner))
    }
}
