use super::*;
use crate::data::resource::{
    ResourceManagedQueueBinding, ResourceManagedQueueDenial, ResourceManagedQueueMutationReport,
};

/// Post-admission refusal retains the actual occurrence and any bound queue.
/// Its caller must retain its original source binding and service port for cleanup.
pub enum SignalOwnedAsyncManagedAdmission {
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
        binding: &SignalOwnedAsyncSourceBinding,
        queue: &ResourceManagedQueueBinding,
    ) -> Result<
        crate::data::resource::ResourceSafePointObservationReport,
        SignalOwnedAsyncQueueDenial,
    > {
        self.require_owned_async_binding(binding)
            .map_err(SignalOwnedAsyncQueueDenial::Service)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.observe_conditional_owned_async_safe_point(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                queue,
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
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.admit_conditional_managed_owned_async_request(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                queue_capacity,
            )
        })
    }

    pub fn enqueue_owned_async_queue(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        queue: &ResourceManagedQueueBinding,
        width: u64,
    ) -> Result<ResourceManagedQueueMutationReport, SignalOwnedAsyncQueueDenial> {
        self.mutate_owned_async_queue(binding, queue, width, true)
    }

    pub fn dequeue_owned_async_queue(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        queue: &ResourceManagedQueueBinding,
        width: u64,
    ) -> Result<ResourceManagedQueueMutationReport, SignalOwnedAsyncQueueDenial> {
        self.mutate_owned_async_queue(binding, queue, width, false)
    }

    fn mutate_owned_async_queue(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        queue: &ResourceManagedQueueBinding,
        width: u64,
        enqueue: bool,
    ) -> Result<ResourceManagedQueueMutationReport, SignalOwnedAsyncQueueDenial> {
        self.require_owned_async_binding(binding)
            .map_err(SignalOwnedAsyncQueueDenial::Service)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.mutate_conditional_owned_async_queue(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                queue,
                width,
                enqueue,
            )
        })
        .map_err(SignalOwnedAsyncQueueDenial::Service)?
    }
}
