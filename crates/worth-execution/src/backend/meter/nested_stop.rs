/// A nested failure owned by the computation, rather than its request.
/// The child's outcome retains its specific domain value and identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestedComputationStop {
    /// The child returned its domain error.
    DomainFailure,
    /// The child kernel or owned input destruction panicked.
    Panic,
    /// A child value or domain error exceeded its declared heap capacity.
    ResultCapacityExceeded,
    /// A strictly narrower computation declaration refused work.
    /// A request ceiling, including a tie with it, cannot be taken.
    WorkCeiling,
}

impl crate::ChargedBytes for NestedComputationStop {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NestedStopOwner {
    Request,
    Computation(NestedComputationStop),
}

/// One latch retains the first computational cause until it is taken.
/// A request stop dominates permanently; no public operation can remove it.
pub(super) enum NestedStop {
    Clear,
    Computation(NestedComputationStop),
    Request,
}

impl NestedStop {
    pub(super) fn record(&mut self, owner: NestedStopOwner) {
        match (owner, &self) {
            (NestedStopOwner::Request, _) => *self = Self::Request,
            (NestedStopOwner::Computation(cause), Self::Clear) => {
                *self = Self::Computation(cause);
            }
            (NestedStopOwner::Computation(_), Self::Computation(_) | Self::Request) => {}
        }
    }

    pub(super) fn owner(&self) -> Option<NestedStopOwner> {
        match self {
            Self::Clear => None,
            Self::Request => Some(NestedStopOwner::Request),
            Self::Computation(cause) => Some(NestedStopOwner::Computation(*cause)),
        }
    }

    pub(super) fn take(&mut self) -> Option<NestedComputationStop> {
        let Self::Computation(cause) = self else {
            return None;
        };
        let cause = *cause;
        *self = Self::Clear;
        Some(cause)
    }
}

impl super::KernelContext<'_, '_> {
    /// Takes one computational cause already carried by a nested outcome.
    /// The child retains its domain value and identity. The pending cause
    /// must be taken before returning Ok or taking another checkpoint.
    /// Request stops cannot be taken, and joined work/span never change.
    pub fn take_nested_computation_stop(&mut self) -> Option<NestedComputationStop> {
        self.meter.borrow_mut().nested_stop.take()
    }

    pub(crate) fn failure_owner<E>(&self, failure: &super::KernelFailure<E>) -> NestedStopOwner {
        use super::{KernelFailure, KernelStop};
        let meter = self.meter.borrow();
        if meter.nested_stop.owner() == Some(NestedStopOwner::Request) {
            return NestedStopOwner::Request;
        }
        match failure {
            KernelFailure::Domain(_) => {
                NestedStopOwner::Computation(NestedComputationStop::DomainFailure)
            }
            KernelFailure::Panic => NestedStopOwner::Computation(NestedComputationStop::Panic),
            KernelFailure::ResultCapacityExceeded => {
                NestedStopOwner::Computation(NestedComputationStop::ResultCapacityExceeded)
            }
            KernelFailure::Stop(
                KernelStop::Cancelled
                | KernelStop::DeadlineElapsed
                | KernelStop::WorkCounterOverflow,
            ) => NestedStopOwner::Request,
            KernelFailure::Stop(KernelStop::WorkCeiling) => meter.limits.ceiling_owner,
            KernelFailure::Stop(KernelStop::NestedStopped) => meter
                .nested_stop
                .owner()
                .unwrap_or(NestedStopOwner::Request),
        }
    }
}
