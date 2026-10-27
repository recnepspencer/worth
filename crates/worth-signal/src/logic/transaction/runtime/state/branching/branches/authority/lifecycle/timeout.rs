use super::*;
use crate::branch::owner_services::conditional_execution::{
    SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncManagedProgress, SignalOwnedAsyncManagedReport,
    SignalOwnedAsyncManagedWakeCleanup, SignalOwnedAsyncQueueDenial,
};

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn advance_owned_async_timeout_handoff(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncManagedHandoff, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_handle(node, handle)?;
        let wake = self
            .derived
            .resource
            .active_timeout_wake_for_handle(handle)
            .ok_or(SignalConditionalServiceExecutionDenial::DefinitionMismatch)?;
        let ready = match self.advance_owned_async_wake_handoff(wake, coordinate)? {
            Ok(ready) => ready,
            Err(deferred) => return Ok(deferred),
        };
        let capture = self.captures_telemetry();
        // The real timeout transition is retained before fallible wake cleanup.
        let report = self.derived.resource.admit_resource_timeout(
            handle,
            ready.clone(),
            capture.then_some(&mut self.derived.telemetry.resource),
        );
        let progress = if report.timed_out_request().is_some() {
            SignalOwnedAsyncManagedProgress::TimedOut
        } else {
            SignalOwnedAsyncManagedProgress::Denied
        };
        let (cleanup, refusal) =
            match self.retire_optional_wake(Some(wake), TemporalWakeRetirementReason::Consumed) {
                Ok(()) => (None, None),
                Err(cause) => (
                    Some(SignalOwnedAsyncManagedWakeCleanup::ReadyTimeout {
                        wake: ready,
                        reason: TemporalWakeRetirementReason::Consumed,
                    }),
                    Some(cause),
                ),
            };
        let mut handoff = self.replacement_handoff(
            progress,
            SignalOwnedAsyncManagedReport::Timeout(report),
            None,
            cleanup,
            refusal,
        );
        match self.require_in_flight(handle) {
            Ok(in_flight) => handoff.in_flight = Some(in_flight),
            Err(cause) if handoff.refusal.is_none() => {
                handoff.refusal = Some(SignalOwnedAsyncQueueDenial::Service(cause));
            }
            Err(_) => {}
        }
        Ok(handoff)
    }

    pub(super) fn schedule_owned_async_timeout(
        &mut self,
        node: ResourceNodeId,
        generation_started: ClockTick,
    ) -> Result<
        Option<(
            crate::data::temporal::ScheduledTemporalWake,
            crate::logic::transaction::runtime::state::resource::ScheduledResourceTimeoutAdmission,
        )>,
        SignalConditionalServiceExecutionDenial,
    > {
        let current = self.derived.temporal.clock_basis().current_tick();
        let plan = self
            .derived
            .resource
            .descriptor_for_node(node)
            .expect("admitted source retains its descriptor")
            .timeout_decision_plan()
            .clone();
        let Some(resolved) =
            crate::logic::transaction::runtime::state::resource::resolve_descriptor_timeout_plan(
                &plan,
                current,
                generation_started,
            )
        else {
            return Ok(None);
        };
        let capture = self.captures_telemetry();
        let wake = self
            .derived
            .temporal
            .schedule_owned_wake(
                TemporalWakeOwner::ResourceNode(node.node()),
                TemporalCondition::after(resolved.timeout_duration().get())
                    .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?,
                resolved.due_tick(),
                capture.then_some(&mut self.derived.telemetry.temporal),
            )
            .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?;
        let admission = resolved.bind_scheduled_wake(wake.id());
        Ok(Some((wake, admission)))
    }
}
