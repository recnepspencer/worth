use crate::branch::owner_services::conditional_execution::{
    SignalConditionalServiceExecutionDenial, SignalOwnedAsyncRequestAdmission,
    SignalOwnedAsyncRetryAdmission, SignalOwnedAsyncRetrySchedule,
    SignalOwnedAsyncRevalidationAdmission, SignalOwnedAsyncTimeoutAdmission,
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
    pub(super) fn admit_owned_async_request_with_lifecycle(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<SignalOwnedAsyncRequestAdmission, SignalConditionalServiceExecutionDenial> {
        use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncManagedAdmissionState;
        match self.admit_owned_async_request_handoff(node, true, false)? {
            SignalOwnedAsyncManagedAdmissionState::Ready { request, .. } => Ok(request),
            SignalOwnedAsyncManagedAdmissionState::CleanupRequired { cause, .. } => {
                // The legacy Native host signature has no caller-owned cleanup
                // result. Application orchestration uses the complete custody
                // entrance; the Native runtime still owns its bounded state.
                replacement_handoff::legacy_replacement_refusal(Some(cause))?;
                unreachable!("a cleanup refusal carries a Native service cause")
            }
        }
    }

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

    pub(super) fn revalidate_owned_async_request_with_lifecycle(
        &mut self,
        node: ResourceNodeId,
        intent: ResourceRevalidationIntent,
    ) -> Result<SignalOwnedAsyncRevalidationAdmission, SignalConditionalServiceExecutionDenial>
    {
        self.require_unmanaged_owned_async_revalidation(node, &intent)?;
        let handoff = self.revalidate_owned_async_request_handoff(node, intent)?;
        replacement_handoff::legacy_replacement_refusal(handoff.refusal)?;
        let crate::branch::owner_services::conditional_execution::SignalOwnedAsyncManagedReport::Revalidation(report) = handoff.report else {
            unreachable!("revalidation retains its actual native report")
        };
        Ok(SignalOwnedAsyncRevalidationAdmission::new(
            report,
            handoff.in_flight,
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

    pub(crate) fn advance_owned_async_request_to_timeout(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncTimeoutAdmission, SignalConditionalServiceExecutionDenial> {
        self.require_unmanaged_owned_async_progression(node, handle)?;
        let handoff = self.advance_owned_async_timeout_handoff(node, handle, coordinate)?;
        replacement_handoff::legacy_replacement_refusal(handoff.refusal)?;
        let crate::branch::owner_services::conditional_execution::SignalOwnedAsyncManagedReport::Timeout(report) = handoff.report else {
            unreachable!("a temporal deferred handoff carries its actual refusal")
        };
        Ok(SignalOwnedAsyncTimeoutAdmission::new(report))
    }

    pub(crate) fn schedule_owned_async_retry(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
    ) -> Result<
        crate::data::resource::ResourceRetryScheduleReport,
        SignalConditionalServiceExecutionDenial,
    > {
        self.require_unmanaged_owned_async_progression(node, handle)?;
        let handoff = self.schedule_owned_async_retry_handoff(node, handle)?;
        replacement_handoff::legacy_replacement_refusal(handoff.refusal)?;
        let crate::branch::owner_services::conditional_execution::SignalOwnedAsyncManagedReport::RetrySchedule(report) = handoff.report else {
            unreachable!("retry scheduling retains its actual native report")
        };
        Ok(report)
    }

    pub(crate) fn advance_owned_async_retry(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        schedule: &SignalOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncRetryAdmission, SignalConditionalServiceExecutionDenial> {
        self.require_unmanaged_owned_async_progression(node, handle)?;
        let handoff = self.advance_owned_async_retry_handoff(node, handle, schedule, coordinate)?;
        replacement_handoff::legacy_replacement_refusal(handoff.refusal)?;
        let crate::branch::owner_services::conditional_execution::SignalOwnedAsyncManagedReport::Retry(report) = handoff.report else {
            unreachable!("a temporal deferred handoff carries its actual refusal")
        };
        Ok(SignalOwnedAsyncRetryAdmission::new(
            report,
            handoff.in_flight,
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
