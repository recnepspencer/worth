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
mod managed_queue;
mod managed_replacement;
mod replacement_handoff;
mod request_custody;
mod retry;
mod retry_schedule;
mod revalidation;
mod temporal_progress;
mod timeout;
mod wake_cleanup;

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
        let report = self.admit_owned_async_request_report(node, true)?;
        let in_flight = self.require_in_flight(report.admitted_request().handle())?;
        Ok(SignalOwnedAsyncRequestAdmission::new(report, in_flight))
    }

    fn admit_owned_async_request_report(
        &mut self,
        node: ResourceNodeId,
        allow_coalescing: bool,
    ) -> Result<
        crate::data::resource::ResourceRequestAdmissionReport,
        SignalConditionalServiceExecutionDenial,
    > {
        self.admit_owned_async_source(node)?;
        let tick = self.derived.temporal.clock_basis().current_tick();
        let timeout = self.schedule_owned_async_timeout(node, tick)?;
        let timeout_wake = timeout.as_ref().map(|(wake, _)| wake.id());
        let timeout_admission = timeout.map(|(_, admission)| admission);
        let capture = self.captures_telemetry();
        let report = self
            .derived
            .resource
            .admit_resource_request(
                ResourceRequestIntent::new(node),
                self.branch_id(),
                tick,
                allow_coalescing,
                timeout_admission,
                capture.then_some(&mut self.derived.telemetry.resource),
            )
            .map_err(|error| {
                if let Some(wake) = timeout_wake {
                    let _ = self.derived.temporal.retire_wake(
                        wake,
                        TemporalWakeRetirementReason::Disposed,
                        capture.then_some(&mut self.derived.telemetry.temporal),
                    );
                }
                SignalConditionalServiceExecutionDenial::SlotAdmission(error)
            })?;
        Ok(report)
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
