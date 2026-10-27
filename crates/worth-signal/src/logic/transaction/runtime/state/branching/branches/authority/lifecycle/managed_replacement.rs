use super::*;
use crate::branch::owner_services::conditional_execution::{
    SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncManagedProgress,
    SignalOwnedAsyncManagedQueueHandoff, SignalOwnedAsyncManagedReport,
    SignalOwnedAsyncQueueDenial as Denial, SignalOwnedAsyncRetentionObligation,
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
        let previous_timeout = self
            .require_in_flight(handle)
            .map_err(Denial::Service)?
            .timeout_wake_id();
        let mut handoff = self
            .advance_owned_async_retry_handoff(node, handle, schedule, coordinate)
            .map_err(Denial::Service)?;
        if let Some(retention) = &mut handoff.retention {
            // Acquired retry cleanup moves the genuine pending schedule into
            // retained lineage even when the retry itself was denied/deferred.
            retention.retry = schedule.report().scheduled_retry().cloned();
        }
        if let SignalOwnedAsyncManagedReport::Retry(report) = &handoff.report {
            if let Some(admitted) = report.admitted_retry() {
                let retry = admitted.scheduled();
                handoff.retention = Some(SignalOwnedAsyncRetentionObligation {
                    previous: Some(handle),
                    wakes: [previous_timeout, Some(retry.backoff_wake_id())],
                    retry: Some(retry),
                });
            }
        }
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
        process_local: bool,
    ) -> Result<SignalOwnedAsyncManagedHandoff, Denial> {
        self.require_replacement_custody(node, handle, queue)?;
        let previous = self.require_in_flight(handle).map_err(Denial::Service)?;
        if process_local {
            self.require_process_local_owned_async(node)
                .map_err(Denial::Service)?;
            if previous.status() == crate::data::resource::ResourceInFlightStatus::Fulfilled {
                return Err(Denial::ReleaseRequired);
            }
        }
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
        let mut handoff = self
            .revalidate_owned_async_request_handoff(node, intent)
            .map_err(Denial::Service)?;
        if process_local {
            if let SignalOwnedAsyncManagedReport::Revalidation(report) = &handoff.report {
                if let Some(admitted) = report.admitted_revalidation() {
                    self.derived
                        .resource
                        .mark_process_local_request(admitted.admitted_request().handle());
                    if let Some(coalesced) = admitted.coalescing() {
                        self.derived
                            .resource
                            .mark_process_local_request(coalesced.coalesced_request().handle());
                    }
                    if let Some(current) = &mut handoff.in_flight {
                        current.mark_process_local();
                    }
                }
            }
        }
        if let SignalOwnedAsyncManagedReport::Revalidation(report) = &handoff.report {
            if let Some(admitted) = report.admitted_revalidation() {
                let selected = admitted
                    .coalescing()
                    .map(|value| value.coalesced_request().handle())
                    .or_else(|| (admitted.admitted_request().handle() != handle).then_some(handle));
                handoff.retention = Some(SignalOwnedAsyncRetentionObligation {
                    previous: selected,
                    retry: None,
                    wakes: [previous.timeout_wake_id(), None],
                });
            }
        }
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
