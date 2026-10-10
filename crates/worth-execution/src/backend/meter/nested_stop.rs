//! A refused question is pending until the invoking kernel settles.
//! Success clears it; failure promotes it to the permanent request latch.
//!
//! | Refusal family | Marking door |
//! | --- | --- |
//! | Lease policy, lineage, reservation binding | `LeaseDenial::refused` |
//! | Lease, serial, or declared memory room | `MemoryLimitDenial::refused` |
//! | Extra worker slot | `ClaimSlot::busy` |
//! | Oracle encoding buffer admission | `ComparisonStop::refused` |
//! | Typed scope refusal | `KernelContext::refuse_request` |
//!
//! These doors use `request_refusal` or the supplied context's pending mark.
//! Cancellation, deadlines, request ceilings and counter overflows are facts,
//! not refusals: their producers record the latch directly (`request_stop`
//! serves producers without an explicit context).

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
            // A second computation cause leaves the first untaken cause intact.
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

    /// Kernel adapters preserve ownership before wrapping a typed cause as data.
    pub(crate) fn record_nested_stop_owner(&mut self, owner: NestedStopOwner) {
        self.meter.borrow_mut().nested_stop.record(owner);
    }

    pub(crate) fn refuse_request<E>(&self, cause: E) -> E {
        self.meter.borrow_mut().pending_request_refusal = true;
        cause
    }

    /// Settle the run's refused questions before classifying its final result.
    pub(crate) fn settle_result<R, E>(
        &self,
        result: &Result<R, E>,
        classify: impl FnOnce(&E) -> NestedStopOwner,
    ) -> Option<NestedStopOwner> {
        let mut meter = self.meter.borrow_mut();
        if std::mem::take(&mut meter.pending_request_refusal) && result.is_err() {
            meter.nested_stop.record(NestedStopOwner::Request);
        }
        drop(meter);
        result.as_ref().err().map(classify)
    }

    pub(crate) fn failure_owner<E>(&self, failure: &super::KernelFailure<E>) -> NestedStopOwner {
        use super::{KernelFailure, KernelStop};
        let meter = self.meter.borrow();
        if meter.nested_stop.owner() == Some(NestedStopOwner::Request) {
            return NestedStopOwner::Request;
        }
        match failure {
            KernelFailure::Domain(_) => {
                meter
                    .nested_stop
                    .owner()
                    .unwrap_or(NestedStopOwner::Computation(
                        NestedComputationStop::DomainFailure,
                    ))
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

/// Mark a refused question on the invoking run, without stopping its kernel.
pub(crate) fn request_refusal<E>(cause: E) -> E {
    super::ACTIVE_METER.with(|active| {
        if let Some(meter) = active.borrow().last() {
            meter.borrow_mut().pending_request_refusal = true;
        }
    });
    cause
}

/// Record an established request stop rather than a refused question.
pub(crate) fn request_stop<E>(cause: E) -> E {
    super::ACTIVE_METER.with(|active| {
        if let Some(meter) = active.borrow().last() {
            meter
                .borrow_mut()
                .nested_stop
                .record(NestedStopOwner::Request);
        }
    });
    cause
}
