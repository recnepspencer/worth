use super::*;
use crate::data::resource::{AdmittedResourceRequest, ResourceManagedQueueBinding};
use crate::data::temporal::{
    ReadyTemporalWake, RetiredTemporalWake, RuntimeClockBasis, TemporalWakeId,
    TemporalWakeRetirementReason,
};

/// The actual native progress of one managed operation, never an admission proof.
#[derive(Debug)]
pub enum SignalOwnedAsyncManagedReport {
    Admission(ResourceRequestAdmissionReport),
    Retry(ResourceRetryAdmissionReport),
    RetrySchedule(ResourceRetryScheduleReport),
    Timeout(ResourceTimeoutReport),
    Revalidation(ResourceRevalidationReport),
    TemporalDeferred(RuntimeClockBasis),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalOwnedAsyncManagedProgress {
    Denied,
    Coalesced,
    Replaced,
    TemporalDeferred,
    Scheduled,
    TimedOut,
}

/// Sole custody of the current managed native occurrence and its bounded cleanup.
/// Reports and local request selectors cannot construct or replace this owner.
pub struct SignalOwnedAsyncManagedAdmission {
    pub(super) origin: SignalOwnedAsyncSourceBinding,
    pub(super) admitted: AdmittedResourceRequest,
    pub(super) in_flight: Option<InFlightResourceRequest>,
    pub(super) queue: Option<SignalOwnedAsyncManagedQueueBinding>,
    pub(super) report: SignalOwnedAsyncManagedReport,
    pub(super) cleanup: Option<SignalOwnedAsyncManagedWakeCleanup>,
    pub(super) refusal: Option<SignalOwnedAsyncQueueDenial>,
}

/// One actual outstanding native wake handoff. An already retired wake is never
/// retired again, and retry cleanup cannot clear another pending occurrence.
pub(crate) enum SignalOwnedAsyncManagedWakeCleanup {
    ReadyRetry {
        previous: ResourceRequestHandle,
        wake: ReadyTemporalWake,
        reason: TemporalWakeRetirementReason,
    },
    RetiredRetry {
        previous: ResourceRequestHandle,
        wake: RetiredTemporalWake,
        reason: TemporalWakeRetirementReason,
    },
    Retirement {
        wake: TemporalWakeId,
        reason: TemporalWakeRetirementReason,
    },
    ReadyTimeout {
        wake: ReadyTemporalWake,
        reason: TemporalWakeRetirementReason,
    },
}

pub(crate) enum SignalOwnedAsyncManagedQueueHandoff {
    Preserve,
    Replace(Option<ResourceManagedQueueBinding>),
}

pub(crate) struct SignalOwnedAsyncManagedHandoff {
    pub(crate) progress: SignalOwnedAsyncManagedProgress,
    pub(crate) report: SignalOwnedAsyncManagedReport,
    pub(crate) admitted: Option<AdmittedResourceRequest>,
    pub(crate) in_flight: Option<InFlightResourceRequest>,
    pub(crate) queue: SignalOwnedAsyncManagedQueueHandoff,
    pub(crate) cleanup: Option<SignalOwnedAsyncManagedWakeCleanup>,
    pub(crate) refusal: Option<SignalOwnedAsyncQueueDenial>,
}

impl SignalOwnedAsyncManagedAdmission {
    pub(super) fn from_initial(
        origin: SignalOwnedAsyncSourceBinding,
        state: SignalOwnedAsyncManagedAdmissionState,
    ) -> Self {
        let (report, in_flight, queue, refusal) = match state {
            SignalOwnedAsyncManagedAdmissionState::Ready { request, queue } => {
                (request.report, Some(request.in_flight), Some(queue), None)
            }
            SignalOwnedAsyncManagedAdmissionState::CleanupRequired {
                report,
                queue,
                cause,
            } => (report, None, queue, Some(cause)),
        };
        Self {
            admitted: report.admitted_request(),
            in_flight,
            queue: queue.map(|binding| SignalOwnedAsyncManagedQueueBinding {
                origin: origin.clone(),
                binding,
            }),
            origin,
            report: SignalOwnedAsyncManagedReport::Admission(report),
            cleanup: None,
            refusal,
        }
    }

    pub fn admitted_request(&self) -> AdmittedResourceRequest {
        self.admitted
    }

    pub fn in_flight(&self) -> Option<&InFlightResourceRequest> {
        self.in_flight.as_ref()
    }

    pub fn queue(&self) -> Option<&SignalOwnedAsyncManagedQueueBinding> {
        self.queue.as_ref()
    }

    pub fn source_binding(&self) -> &SignalOwnedAsyncSourceBinding {
        &self.origin
    }

    pub fn report(&self) -> &SignalOwnedAsyncManagedReport {
        &self.report
    }

    pub fn refusal(&self) -> Option<&SignalOwnedAsyncQueueDenial> {
        self.refusal.as_ref()
    }

    pub fn wake_cleanup_pending(&self) -> bool {
        self.cleanup.is_some()
    }

    pub(super) fn retain_handoff(
        &mut self,
        origin: &SignalOwnedAsyncSourceBinding,
        handoff: SignalOwnedAsyncManagedHandoff,
    ) -> SignalOwnedAsyncManagedProgress {
        self.origin = origin.clone();
        if let Some(admitted) = handoff.admitted {
            self.admitted = admitted;
        }
        if handoff.admitted.is_some() || handoff.in_flight.is_some() {
            self.in_flight = handoff.in_flight;
        }
        match handoff.queue {
            SignalOwnedAsyncManagedQueueHandoff::Preserve => {
                if let Some(queue) = &mut self.queue {
                    queue.origin = origin.clone();
                }
            }
            SignalOwnedAsyncManagedQueueHandoff::Replace(queue) => {
                self.queue = queue.map(|binding| SignalOwnedAsyncManagedQueueBinding {
                    origin: origin.clone(),
                    binding,
                });
            }
        }
        self.report = handoff.report;
        self.cleanup = handoff.cleanup;
        self.refusal = handoff.refusal;
        handoff.progress
    }
}
