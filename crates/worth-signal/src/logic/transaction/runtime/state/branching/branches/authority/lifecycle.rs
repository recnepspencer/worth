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

mod support;

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
                true,
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
        let in_flight = self.require_in_flight(report.admitted_request().handle())?;
        Ok(SignalOwnedAsyncRequestAdmission::new(report, in_flight))
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
        self.admit_owned_async_source(node)?;
        if intent.node() != node {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        let capture = self.captures_telemetry();
        let digest = self
            .derived
            .resource
            .descriptor_for_node(node)
            .expect("admitted source retains its descriptor")
            .revalidation_decision_plan()
            .decision_digest()
            .clone();
        let prepared = match self
            .derived
            .resource
            .prepare_explicit_resource_revalidation(
                intent,
                digest,
                capture.then_some(&mut self.derived.telemetry.resource),
            ) {
            Ok(prepared) => prepared,
            Err(report) => return Ok(SignalOwnedAsyncRevalidationAdmission::new(report, None)),
        };
        let prior_timeout = self.derived.resource.active_timeout_wake_for_node(node);
        let tick = self.derived.temporal.clock_basis().current_tick();
        let timeout = self.schedule_owned_async_timeout(node, tick)?;
        let timeout_wake = timeout.as_ref().map(|(wake, _)| wake.id());
        let report = self.derived.resource.admit_prepared_resource_revalidation(
            prepared,
            self.branch_id(),
            tick,
            timeout.map(|(_, admission)| admission),
            capture.then_some(&mut self.derived.telemetry.resource),
        );
        if report.admitted_revalidation().is_some() {
            self.retire_optional_wake(prior_timeout, TemporalWakeRetirementReason::Superseded)?;
        } else {
            self.retire_optional_wake(timeout_wake, TemporalWakeRetirementReason::Disposed)?;
        }
        let in_flight = report
            .admitted_revalidation()
            .map(|admitted| self.require_in_flight(admitted.admitted_request().handle()))
            .transpose()?;
        Ok(SignalOwnedAsyncRevalidationAdmission::new(
            report, in_flight,
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
        self.require_owned_async_handle(node, handle)?;
        let wake = self
            .derived
            .resource
            .active_timeout_wake_for_handle(handle)
            .ok_or(SignalConditionalServiceExecutionDenial::DefinitionMismatch)?;
        let ready = self.advance_owned_async_wake(wake, coordinate)?;
        self.retire_optional_wake(Some(wake), TemporalWakeRetirementReason::Consumed)?;
        let capture = self.captures_telemetry();
        let report = self.derived.resource.admit_resource_timeout(
            handle,
            ready,
            capture.then_some(&mut self.derived.telemetry.resource),
        );
        Ok(SignalOwnedAsyncTimeoutAdmission::new(report))
    }

    pub(crate) fn schedule_owned_async_retry(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
    ) -> Result<SignalOwnedAsyncRetrySchedule, SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_source(node)?;
        let capture = self.captures_telemetry();
        let tick = self.derived.temporal.clock_basis().current_tick();
        let (delay, next_attempt, digest, budget) =
            match self.derived.resource.retry_backoff_delay_for_handle(
                handle,
                tick,
                capture.then_some(&mut self.derived.telemetry.resource),
            ) {
                Ok(decision) => decision,
                Err(class) => {
                    let report = self.derived.resource.deny_resource_retry_schedule(
                        handle,
                        class,
                        capture.then_some(&mut self.derived.telemetry.resource),
                    );
                    return Ok(SignalOwnedAsyncRetrySchedule::new(report));
                }
            };
        let due = ClockTick::new(tick.get().saturating_add(delay.get()));
        let wake = self
            .derived
            .temporal
            .schedule_owned_wake(
                TemporalWakeOwner::ResourceNode(node.node()),
                TemporalCondition::after(delay.get())
                    .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?,
                due,
                capture.then_some(&mut self.derived.telemetry.temporal),
            )
            .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?;
        let report = self.derived.resource.schedule_resource_retry(
            handle,
            ResourceRetryReason::TimedOut,
            wake.id(),
            next_attempt,
            delay,
            digest,
            budget,
            capture.then_some(&mut self.derived.telemetry.resource),
        );
        if report.denied_retry().is_some() {
            self.retire_optional_wake(Some(wake.id()), TemporalWakeRetirementReason::Disposed)?;
        }
        Ok(SignalOwnedAsyncRetrySchedule::new(report))
    }

    pub(crate) fn advance_owned_async_retry(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        schedule: &SignalOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncRetryAdmission, SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_source(node)?;
        let scheduled = schedule
            .report()
            .scheduled_retry()
            .filter(|scheduled| scheduled.previous() == handle)
            .ok_or(SignalConditionalServiceExecutionDenial::DefinitionMismatch)?;
        let wake = scheduled.backoff_wake_id();
        if self.derived.resource.pending_retry_wake_for_handle(handle) != Some(wake) {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        let ready = self.advance_owned_async_wake(wake, coordinate)?;
        let capture = self.captures_telemetry();
        let prepared = match self.derived.resource.prepare_scheduled_resource_retry(
            handle,
            &ready,
            capture.then_some(&mut self.derived.telemetry.resource),
        ) {
            Ok(prepared) => prepared,
            Err(report) => return Ok(SignalOwnedAsyncRetryAdmission::new(report, None)),
        };
        self.retire_optional_wake(Some(wake), TemporalWakeRetirementReason::Consumed)?;
        let generation_started = prepared.previous().generation_started_tick();
        let tick = self.derived.temporal.clock_basis().current_tick();
        let timeout = self.schedule_owned_async_timeout(node, generation_started)?;
        let timeout_wake = timeout.as_ref().map(|(wake, _)| wake.id());
        let report = self
            .derived
            .resource
            .admit_prepared_scheduled_resource_retry(
                prepared,
                ready,
                self.branch_id(),
                tick,
                timeout.map(|(_, admission)| admission),
                capture.then_some(&mut self.derived.telemetry.resource),
            );
        if report.denied_retry().is_some() {
            self.retire_optional_wake(timeout_wake, TemporalWakeRetirementReason::Disposed)?;
        }
        let in_flight = report
            .admitted_retry()
            .map(|admitted| self.require_in_flight(admitted.admitted_request().handle()))
            .transpose()?;
        Ok(SignalOwnedAsyncRetryAdmission::new(report, in_flight))
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
}
