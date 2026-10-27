use super::*;
use crate::branch::owner_services::conditional_execution::{
    SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncManagedProgress,
    SignalOwnedAsyncManagedQueueHandoff, SignalOwnedAsyncQueueDenial as Denial,
};
use crate::data::resource::{ResourceManagedQueueBinding, ResourceRequestHandle};

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn advance_managed_owned_async_retry(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        queue: Option<&ResourceManagedQueueBinding>,
        schedule: &SignalOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncManagedHandoff, Denial> {
        self.require_replacement_custody(node, handle, queue)?;
        let handoff = self
            .advance_owned_async_retry_handoff(node, handle, schedule, coordinate)
            .map_err(Denial::Service)?;
        Ok(match queue {
            Some(queue) => self.bind_managed_replacement_queue(queue.queue_capacity(), handoff),
            None => handoff,
        })
    }

    pub(crate) fn revalidate_managed_owned_async_request(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        queue: Option<&ResourceManagedQueueBinding>,
        intent: ResourceRevalidationIntent,
    ) -> Result<SignalOwnedAsyncManagedHandoff, Denial> {
        self.require_replacement_custody(node, handle, queue)?;
        let active = self.derived.resource.active_request_handle_for_node(node);
        if active.is_some_and(|active| active != handle) {
            return Err(Denial::Service(
                SignalConditionalServiceExecutionDenial::DefinitionMismatch,
            ));
        }
        if intent.expected_active() != Some(handle) {
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
        Ok(match queue {
            Some(queue) => self.bind_managed_replacement_queue(queue.queue_capacity(), handoff),
            None => handoff,
        })
    }

    fn require_replacement_custody(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        queue: Option<&ResourceManagedQueueBinding>,
    ) -> Result<(), Denial> {
        let prior = self
            .require_owned_async_request_instance(node, handle)
            .map_err(Denial::Service)?;
        let Some(queue) = queue else {
            return Ok(());
        };
        if queue.request() != handle || prior.attempt() != queue.attempt() {
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
