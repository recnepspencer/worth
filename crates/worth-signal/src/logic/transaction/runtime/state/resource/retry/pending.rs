use super::super::ResourceRuntimeState;
use crate::data::resource::{ResourceNodeId, ResourceRequestHandle, ScheduledResourceRetry};
use crate::data::temporal::TemporalWakeId;

impl ResourceRuntimeState {
    pub fn pending_retry_wake_for_handle(
        &self,
        handle: ResourceRequestHandle,
    ) -> Option<TemporalWakeId> {
        self.pending_retry_by_request
            .get(&handle.request_id())
            .filter(|scheduled| scheduled.previous() == handle)
            .map(|scheduled| scheduled.backoff_wake_id())
    }

    pub fn pending_retry_wake_for_node(&self, node: ResourceNodeId) -> Option<TemporalWakeId> {
        self.pending_retry_by_node
            .get(&node)
            .map(|scheduled| scheduled.backoff_wake_id())
    }

    pub fn clear_pending_retry_for_node(
        &mut self,
        node: ResourceNodeId,
    ) -> Option<ScheduledResourceRetry> {
        let scheduled = self.pending_retry_by_node.get(&node)?.clone();
        Some(self.take_pending_retry_record(node, scheduled))
    }

    pub(in crate::logic::transaction::runtime) fn pending_retry_matches(
        &self,
        node: ResourceNodeId,
        scheduled: &ScheduledResourceRetry,
    ) -> bool {
        self.in_flight_by_request
            .get(&scheduled.previous().request_id())
            .is_some_and(|request| {
                request.node() == node && request.handle() == scheduled.previous()
            })
            && self
                .pending_retry_by_request
                .get(&scheduled.previous().request_id())
                == Some(scheduled)
            && self.pending_retry_by_wake.get(&scheduled.backoff_wake_id())
                == Some(&scheduled.previous().request_id())
    }

    pub(in crate::logic::transaction::runtime) fn clear_pending_retry_exact(
        &mut self,
        node: ResourceNodeId,
        scheduled: &ScheduledResourceRetry,
    ) -> Option<ScheduledResourceRetry> {
        if !self.pending_retry_matches(node, scheduled) {
            return None;
        }
        let scheduled = self
            .pending_retry_by_request
            .get(&scheduled.previous().request_id())?
            .clone();
        Some(self.take_pending_retry_record(node, scheduled))
    }

    pub(in crate::logic::transaction::runtime) fn clear_pending_retry_for_retired_wake(
        &mut self,
        node: ResourceNodeId,
        previous: ResourceRequestHandle,
        retired: TemporalWakeId,
    ) -> Option<ScheduledResourceRetry> {
        let scheduled = self
            .pending_retry_by_request
            .get(&previous.request_id())
            .filter(|scheduled| {
                scheduled.previous() == previous && scheduled.backoff_wake_id() == retired
            })
            .cloned()?;
        self.clear_pending_retry_exact(node, &scheduled)
    }

    fn take_pending_retry_record(
        &mut self,
        node: ResourceNodeId,
        scheduled: ScheduledResourceRetry,
    ) -> ScheduledResourceRetry {
        if self.pending_retry_by_node.get(&node) == Some(&scheduled) {
            self.pending_retry_by_node.remove(&node);
        }
        self.pending_retry_by_request
            .remove(&scheduled.previous().request_id());
        self.pending_retry_by_wake
            .remove(&scheduled.backoff_wake_id());
        self.retain_retry_lineage(node, scheduled.clone());
        scheduled
    }
}
