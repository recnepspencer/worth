//! One actual initial report and its own cleanup, including shared winners.
use super::*;
use crate::data::resource::{AdmittedResourceRequest, ResourceManagedQueueBinding};

pub(crate) enum SignalOwnedAsyncManagedAdmissionState {
    Ready {
        request: SignalOwnedAsyncRequestAdmission,
        queue: Option<ResourceManagedQueueBinding>,
    },
    CleanupRequired {
        report: ResourceRequestAdmissionReport,
        in_flight: Option<InFlightResourceRequest>,
        queue: Option<ResourceManagedQueueBinding>,
        cleanup: Option<SignalOwnedAsyncManagedWakeCleanup>,
        cause: SignalOwnedAsyncQueueDenial,
    },
}

impl SignalOwnedAsyncManagedAdmissionState {
    pub(crate) fn admitted_request(&self) -> AdmittedResourceRequest {
        match self {
            Self::Ready { request, .. } => request.report.admitted_request(),
            Self::CleanupRequired { report, .. } => report.admitted_request(),
        }
    }

    pub(crate) fn with_queue(
        self,
        result: Result<ResourceManagedQueueBinding, SignalOwnedAsyncQueueDenial>,
    ) -> Self {
        match (self, result) {
            (Self::Ready { request, .. }, Ok(queue)) => Self::Ready {
                request,
                queue: Some(queue),
            },
            (Self::Ready { request, .. }, Err(cause)) => Self::CleanupRequired {
                report: request.report,
                in_flight: Some(request.in_flight),
                queue: None,
                cleanup: None,
                cause,
            },
            (
                Self::CleanupRequired {
                    report,
                    in_flight,
                    cleanup,
                    cause,
                    ..
                },
                result,
            ) => Self::CleanupRequired {
                report,
                in_flight,
                queue: result.ok(),
                cleanup,
                cause,
            },
        }
    }
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Native host issuance without a managed queue. Bridge separately checks
    /// its genuine installed declaration before choosing this entrance.
    pub fn admit_unqueued_owned_async_request(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
    ) -> Result<SignalOwnedAsyncManagedAdmission, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        let state = self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.admit_conditional_unqueued_owned_async_request(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
            )
        })?;
        Ok(SignalOwnedAsyncManagedAdmission::from_initial(
            binding.clone(),
            state,
            false,
        ))
    }
}
