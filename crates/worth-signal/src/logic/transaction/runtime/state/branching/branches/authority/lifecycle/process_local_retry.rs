use super::*;
use crate::data::resource::{ResourceInFlightStatus, ScheduledResourceRetry};
use crate::data::temporal::RetiredTemporalWake;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn abort_process_local_owned_async_retry(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        scheduled: &ScheduledResourceRetry,
    ) -> Result<RetiredTemporalWake, SignalConditionalServiceExecutionDenial> {
        self.require_process_local_owned_async(node)?;
        let request = self.require_owned_async_request_instance(node, handle)?;
        if scheduled.previous() != handle
            || request.status() != ResourceInFlightStatus::TimedOut
            || !self.derived.resource.pending_retry_matches(node, scheduled)
        {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        let capture = self.captures_telemetry();
        let retired = self
            .derived
            .temporal
            .retire_resource_retry_wake(
                node,
                scheduled.backoff_wake_id(),
                capture.then_some(&mut self.derived.telemetry.temporal),
            )
            .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?;
        // The exact native map comparison was made under this same cell lock.
        // No fallible work follows the actual wake retirement.
        self.derived
            .resource
            .clear_pending_retry_exact(node, scheduled)
            .expect("the actual retired retry retains its exact pending record");
        self.derived
            .resource
            .release_process_local_retry_lineage(node, scheduled);
        self.derived
            .temporal
            .release_resource_wake(scheduled.backoff_wake_id());
        Ok(retired)
    }
}
