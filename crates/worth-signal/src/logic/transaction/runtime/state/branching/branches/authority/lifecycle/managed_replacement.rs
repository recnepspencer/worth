use super::*;
use crate::branch::owner_services::conditional_execution::{
    SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncManagedProgress,
    SignalOwnedAsyncManagedQueueHandoff, SignalOwnedAsyncQueueDenial as Denial,
};
use crate::data::resource::ResourceManagedQueueBinding;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn advance_managed_owned_async_retry(
        &mut self,
        node: ResourceNodeId,
        queue: &ResourceManagedQueueBinding,
        schedule: &SignalOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncManagedHandoff, Denial> {
        self.require_managed_replacement_queue(node, queue)?;
        let handoff = self
            .advance_owned_async_retry_handoff(node, queue.request(), schedule, coordinate)
            .map_err(Denial::Service)?;
        Ok(self.bind_managed_replacement_queue(queue.queue_capacity(), handoff))
    }

    pub(crate) fn revalidate_managed_owned_async_request(
        &mut self,
        node: ResourceNodeId,
        queue: &ResourceManagedQueueBinding,
        intent: ResourceRevalidationIntent,
    ) -> Result<SignalOwnedAsyncManagedHandoff, Denial> {
        self.require_managed_replacement_queue(node, queue)?;
        let active = self.derived.resource.active_request_handle_for_node(node);
        if active.is_some_and(|active| active != queue.request()) {
            return Err(Denial::Service(
                SignalConditionalServiceExecutionDenial::DefinitionMismatch,
            ));
        }
        if intent.expected_active() != Some(queue.request()) {
            return Err(Denial::Service(
                SignalConditionalServiceExecutionDenial::DefinitionMismatch,
            ));
        }
        if self
            .derived
            .resource
            .pending_retry_wake_for_node(node)
            .is_some()
        {
            return Err(Denial::CleanupPending);
        }
        let handoff = self
            .revalidate_owned_async_request_handoff(node, intent)
            .map_err(Denial::Service)?;
        Ok(self.bind_managed_replacement_queue(queue.queue_capacity(), handoff))
    }

    fn require_managed_replacement_queue(
        &mut self,
        node: ResourceNodeId,
        queue: &ResourceManagedQueueBinding,
    ) -> Result<(), Denial> {
        self.admit_owned_async_source(node)
            .map_err(Denial::Service)?;
        self.require_owned_async_handle(node, queue.request())
            .map_err(Denial::Service)?;
        let prior = self
            .require_in_flight(queue.request())
            .map_err(Denial::Service)?;
        let descriptor = self
            .derived
            .resource
            .descriptor_for_node(node)
            .expect("source admission retains the current descriptor");
        if prior.attempt() != queue.attempt() || prior.descriptor_id() != descriptor.descriptor_id()
        {
            return Err(Denial::Service(
                SignalConditionalServiceExecutionDenial::DefinitionMismatch,
            ));
        }
        let report = self
            .observe_owned_async_safe_point(node, queue)
            .map_err(Denial::Service)??;
        let depth = report.pressure().queue_depth();
        if depth != 0 {
            return Err(Denial::QueueNotDrained { depth });
        }
        Ok(())
    }

    fn bind_managed_replacement_queue(
        &mut self,
        capacity: u64,
        mut handoff: SignalOwnedAsyncManagedHandoff,
    ) -> SignalOwnedAsyncManagedHandoff {
        if handoff.progress != SignalOwnedAsyncManagedProgress::Replaced
            || handoff.refusal.is_some()
        {
            return handoff;
        }
        let admitted = handoff
            .admitted
            .expect("a replacement retains its actual occurrence");
        let capture = self.captures_telemetry();
        match self.derived.resource.bind_managed_queue(
            admitted,
            capacity,
            capture.then_some(&mut self.derived.telemetry.resource),
        ) {
            Ok(queue) => handoff.queue = SignalOwnedAsyncManagedQueueHandoff::Replace(Some(queue)),
            Err(cause) => handoff.refusal = Some(Denial::Queue(cause)),
        }
        handoff
    }
}
