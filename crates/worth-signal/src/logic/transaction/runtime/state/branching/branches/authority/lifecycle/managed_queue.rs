use super::*;
use crate::branch::owner_services::conditional_execution::{
    SignalOwnedAsyncManagedAdmissionState, SignalOwnedAsyncQueueDenial,
};
use crate::data::resource::{ResourceManagedQueueBinding, ResourceManagedQueueMutationReport};

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn observe_owned_async_safe_point(
        &mut self,
        node: ResourceNodeId,
        queue: &ResourceManagedQueueBinding,
    ) -> Result<
        Result<
            crate::data::resource::ResourceSafePointObservationReport,
            SignalOwnedAsyncQueueDenial,
        >,
        SignalConditionalServiceExecutionDenial,
    > {
        self.require_owned_async_handle(node, queue.request())?;
        let capture = self.captures_telemetry();
        Ok(self
            .derived
            .resource
            .observe_safe_point(
                queue,
                capture.then_some(&mut self.derived.telemetry.resource),
            )
            .map_err(SignalOwnedAsyncQueueDenial::SafePoint))
    }

    pub(crate) fn admit_managed_owned_async_request(
        &mut self,
        node: ResourceNodeId,
        capacity: u64,
    ) -> Result<SignalOwnedAsyncManagedAdmissionState, SignalConditionalServiceExecutionDenial>
    {
        self.admit_owned_async_source(node)?;
        if capacity == 0
            || self
                .derived
                .resource
                .active_request_handle_for_node(node)
                .is_some()
        {
            return Err(SignalConditionalServiceExecutionDenial::SlotAdmission(
                crate::data::error::SignalError::invalid_input(
                    "managed owned async admission requires nonzero capacity and a free node",
                ),
            ));
        }
        let state = self.admit_owned_async_request_handoff(node, false)?;
        let capture = self.captures_telemetry();
        let queue = self.derived.resource.bind_managed_queue(
            state.admitted_request(),
            capacity,
            capture.then_some(&mut self.derived.telemetry.resource),
        );
        Ok(state.with_queue(queue.map_err(SignalOwnedAsyncQueueDenial::Queue)))
    }

    pub(crate) fn mutate_owned_async_queue(
        &mut self,
        node: ResourceNodeId,
        queue: &ResourceManagedQueueBinding,
        width: u64,
        enqueue: bool,
    ) -> Result<
        Result<ResourceManagedQueueMutationReport, SignalOwnedAsyncQueueDenial>,
        SignalConditionalServiceExecutionDenial,
    > {
        self.require_owned_async_handle(node, queue.request())?;
        let capture = self.captures_telemetry();
        let telemetry = capture.then_some(&mut self.derived.telemetry.resource);
        Ok(if enqueue {
            self.derived
                .resource
                .enqueue_managed_queue(queue, width, telemetry)
        } else {
            self.derived
                .resource
                .dequeue_managed_queue(queue, width, telemetry)
        }
        .map_err(SignalOwnedAsyncQueueDenial::Queue))
    }
}
