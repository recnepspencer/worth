use crate::branch::owner_services::conditional_execution::SignalConditionalServiceExecutionDenial;
use crate::data::resource::{InFlightResourceRequest, ResourceNodeId, ResourceRequestHandle};
use crate::data::temporal::{
    ClockAdvanceRequest, ClockDomain, ClockTick, ReadyTemporalWake, TemporalCondition,
    TemporalWakeOwner, TemporalWakeRetirementReason,
};

use super::super::BranchState;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
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

    pub(super) fn advance_owned_async_wake(
        &mut self,
        wake: crate::data::temporal::TemporalWakeId,
        coordinate: u64,
    ) -> Result<ReadyTemporalWake, SignalConditionalServiceExecutionDenial> {
        let request =
            ClockAdvanceRequest::new(ClockDomain::MonotonicExecution, ClockTick::new(coordinate));
        let validated = self
            .derived
            .temporal
            .validate_clock_advance(request)
            .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?;
        self.derived.temporal.apply_clock_advance(validated);
        let capture = self.captures_telemetry();
        self.derived
            .temporal
            .promote_wake_ready(
                wake,
                capture.then_some(&mut self.derived.telemetry.temporal),
            )
            .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)
    }

    pub(super) fn require_owned_async_handle(
        &self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        let actual = self
            .derived
            .resource
            .in_flight_request_optional(handle, None)
            .map(InFlightResourceRequest::node);
        if actual == Some(node) {
            Ok(())
        } else {
            Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch)
        }
    }

    pub(super) fn require_in_flight(
        &mut self,
        handle: ResourceRequestHandle,
    ) -> Result<InFlightResourceRequest, SignalConditionalServiceExecutionDenial> {
        let capture = self.captures_telemetry();
        self.derived
            .resource
            .in_flight_request_optional(
                handle,
                capture.then_some(&mut self.derived.telemetry.resource),
            )
            .cloned()
            .ok_or_else(|| {
                SignalConditionalServiceExecutionDenial::SlotAdmission(
                    crate::data::error::SignalError::invalid_input(
                        "owned async lifecycle admission lost its in-flight state",
                    ),
                )
            })
    }

    pub(super) fn retire_optional_wake(
        &mut self,
        wake: Option<crate::data::temporal::TemporalWakeId>,
        reason: TemporalWakeRetirementReason,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        let Some(wake) = wake else {
            return Ok(());
        };
        let capture = self.captures_telemetry();
        self.derived
            .temporal
            .retire_wake(
                wake,
                reason,
                capture.then_some(&mut self.derived.telemetry.temporal),
            )
            .map(|_| ())
            .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)
    }

    pub(super) fn captures_telemetry(&self) -> bool {
        self.graph().captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        )
    }
}
