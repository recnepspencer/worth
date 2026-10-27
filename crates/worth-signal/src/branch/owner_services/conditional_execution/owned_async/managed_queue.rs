use super::*;
use crate::data::resource::{
    ResourceManagedQueueBinding, ResourceManagedQueueDenial, ResourceManagedQueueMutationReport,
};

/// Post-admission refusal retains the actual occurrence and any bound queue.
/// Its caller must retain its original source binding and service port for cleanup.
pub enum SignalOwnedAsyncManagedAdmission {
    Ready {
        request: SignalOwnedAsyncRequestAdmission,
        queue: SignalOwnedAsyncManagedQueueBinding,
    },
    CleanupRequired {
        report: ResourceRequestAdmissionReport,
        queue: Option<SignalOwnedAsyncManagedQueueBinding>,
        cause: SignalOwnedAsyncQueueDenial,
    },
}

pub(crate) enum SignalOwnedAsyncManagedAdmissionState {
    Ready {
        request: SignalOwnedAsyncRequestAdmission,
        queue: ResourceManagedQueueBinding,
    },
    CleanupRequired {
        report: ResourceRequestAdmissionReport,
        queue: Option<ResourceManagedQueueBinding>,
        cause: SignalOwnedAsyncQueueDenial,
    },
}

/// Exact native queue custody, bound to its original live service/source.
/// Local request selectors alone cannot reconstruct this owner.
pub struct SignalOwnedAsyncManagedQueueBinding {
    origin: SignalOwnedAsyncSourceBinding,
    binding: ResourceManagedQueueBinding,
}

impl SignalOwnedAsyncManagedQueueBinding {
    pub fn request(&self) -> ResourceRequestHandle {
        self.binding.request()
    }
    pub fn attempt(&self) -> crate::data::resource::ResourceAttemptId {
        self.binding.attempt()
    }
    pub fn queue_capacity(&self) -> u64 {
        self.binding.queue_capacity()
    }
}

#[derive(Debug)]
pub enum SignalOwnedAsyncQueueDenial {
    Service(SignalConditionalServiceExecutionDenial),
    Queue(ResourceManagedQueueDenial),
    SafePoint(crate::data::resource::ResourceSafePointObservationDenial),
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn observe_owned_async_safe_point(
        &self,
        queue: &SignalOwnedAsyncManagedQueueBinding,
    ) -> Result<
        crate::data::resource::ResourceSafePointObservationReport,
        SignalOwnedAsyncQueueDenial,
    > {
        let binding = &queue.origin;
        self.require_owned_async_binding(binding)
            .map_err(SignalOwnedAsyncQueueDenial::Service)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.observe_conditional_owned_async_safe_point(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                &queue.binding,
            )
        })
        .map_err(SignalOwnedAsyncQueueDenial::Service)?
    }

    pub fn admit_managed_owned_async_request(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        queue_capacity: u64,
    ) -> Result<SignalOwnedAsyncManagedAdmission, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        let state = self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.admit_conditional_managed_owned_async_request(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                queue_capacity,
            )
        })?;
        let wrap = |queue| SignalOwnedAsyncManagedQueueBinding {
            origin: binding.clone(),
            binding: queue,
        };
        Ok(match state {
            SignalOwnedAsyncManagedAdmissionState::Ready { request, queue } => {
                SignalOwnedAsyncManagedAdmission::Ready {
                    request,
                    queue: wrap(queue),
                }
            }
            SignalOwnedAsyncManagedAdmissionState::CleanupRequired {
                report,
                queue,
                cause,
            } => SignalOwnedAsyncManagedAdmission::CleanupRequired {
                report,
                queue: queue.map(wrap),
                cause,
            },
        })
    }

    pub fn enqueue_owned_async_queue(
        &self,
        queue: &SignalOwnedAsyncManagedQueueBinding,
        width: u64,
    ) -> Result<ResourceManagedQueueMutationReport, SignalOwnedAsyncQueueDenial> {
        self.mutate_owned_async_queue(queue, width, true)
    }

    pub fn dequeue_owned_async_queue(
        &self,
        queue: &SignalOwnedAsyncManagedQueueBinding,
        width: u64,
    ) -> Result<ResourceManagedQueueMutationReport, SignalOwnedAsyncQueueDenial> {
        self.mutate_owned_async_queue(queue, width, false)
    }

    fn mutate_owned_async_queue(
        &self,
        queue: &SignalOwnedAsyncManagedQueueBinding,
        width: u64,
        enqueue: bool,
    ) -> Result<ResourceManagedQueueMutationReport, SignalOwnedAsyncQueueDenial> {
        let binding = &queue.origin;
        self.require_owned_async_binding(binding)
            .map_err(SignalOwnedAsyncQueueDenial::Service)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.mutate_conditional_owned_async_queue(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                &queue.binding,
                width,
                enqueue,
            )
        })
        .map_err(SignalOwnedAsyncQueueDenial::Service)?
    }
}
