use crate::branch::owner_services::conditional_execution::{
    SignalConditionalServiceExecutionDenial, SignalOwnedAsyncRequestAdmission,
    SignalOwnedAsyncRetrySchedule,
};
use crate::data::resource::{
    RawCompletionEnvelope, ResourceCancellationReason, ResourceCancellationReport,
    ResourceCompletionAdmissionReport, ResourceNodeId, ResourceRequestHandle,
    ResourceRequestIntent, ResourceRetryReason, ResourceRevalidationIntent,
};
use crate::data::temporal::{
    ClockTick, TemporalCondition, TemporalWakeOwner, TemporalWakeRetirementReason,
};

use super::BranchState;

mod completion_commit;
mod initial_request;
mod managed_queue;
mod managed_replacement;
mod process_local_release;
mod process_local_retry;
mod replacement_handoff;
mod request_custody;
mod retry;
mod retry_schedule;
mod revalidation;
mod temporal_progress;
mod timeout;
mod wake_cleanup;
mod wake_observation_for_test;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn admit_owned_async_completion_with_lifecycle(
        &mut self,
        node: ResourceNodeId,
        raw: RawCompletionEnvelope,
    ) -> Result<ResourceCompletionAdmissionReport, SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_source(node)?;
        let capture = self.captures_telemetry();
        Ok(self.derived.resource.admit_resource_completion(
            raw,
            capture.then_some(&mut self.derived.telemetry.resource),
        ))
    }

    pub(super) fn cancel_owned_async_request_with_lifecycle(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        reason: ResourceCancellationReason,
    ) -> Result<ResourceCancellationReport, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_handle(node, handle)?;
        for wake in self
            .derived
            .resource
            .active_timeout_wakes_for_cancellation_footprint(handle)
        {
            self.retire_optional_wake(Some(wake), TemporalWakeRetirementReason::Cancelled)?;
        }
        let capture = self.captures_telemetry();
        Ok(self.derived.resource.cancel_resource_request(
            handle,
            reason,
            capture.then_some(&mut self.derived.telemetry.resource),
        ))
    }

    pub(crate) fn owned_async_active_request_count(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<usize, SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_source(node)?;
        Ok(usize::from(
            self.derived
                .resource
                .active_request_handle_for_node(node)
                .is_some(),
        ))
    }

    fn captures_telemetry(&self) -> bool {
        self.graph().captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        )
    }
}
