use crate::data::resource::{
    LoweredResourceDescriptor, ResourceRequestHandle, ResourceRetryAdmissionReport,
    ResourceRetryDenialClass, ResourceRetryReason, ResourceRetryScheduleReport,
};

use crate::data::temporal::{
    ReadyTemporalWake, TemporalCondition, TemporalWakeOwner, TemporalWakeRetirementReason,
};

use super::super::SignalRuntime;
use super::timeout_wakes::RetryTimeoutAdmissionResolution;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn schedule_resource_retry(
        &mut self,
        handle: ResourceRequestHandle,
        reason: ResourceRetryReason,
    ) -> Result<ResourceRetryScheduleReport, crate::data::error::SignalError> {
        let capture_telemetry = self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        );
        let (delay, next_attempt, retry_decision_digest, retry_budget_charge) =
            match self.resource.retry_backoff_delay_for_handle(
                handle,
                self.clock_basis().current_tick(),
                capture_telemetry.then_some(&mut self.telemetry.resource),
            ) {
                Ok(delay) => delay,
                Err(class) => {
                    return Ok(self.resource.deny_resource_retry_schedule(
                        handle,
                        class,
                        capture_telemetry.then_some(&mut self.telemetry.resource),
                    ));
                }
            };
        let condition = TemporalCondition::after(delay.get())?;
        let current_tick = self.clock_basis().current_tick();
        let due_tick =
            crate::data::temporal::ClockTick::new(current_tick.get().saturating_add(delay.get()));
        let node = self
            .resource
            .in_flight_request_optional(
                handle,
                capture_telemetry.then_some(&mut self.telemetry.resource),
            )
            .map(|in_flight| in_flight.node())
            .ok_or_else(|| {
                crate::data::error::SignalError::invalid_input(format!(
                    "cannot schedule retry for unknown resource request {}",
                    handle.request_id().get()
                ))
            })?;
        let wake = self.schedule_owned_temporal_wake(
            TemporalWakeOwner::ResourceNode(node.node()),
            condition,
            due_tick,
        )?;
        let report = self.resource.schedule_resource_retry(
            handle,
            reason,
            wake.id(),
            next_attempt,
            delay,
            retry_decision_digest,
            retry_budget_charge,
            capture_telemetry.then_some(&mut self.telemetry.resource),
        );
        if report.denied_retry().is_some() {
            let _ = self.retire_temporal_wake(wake.id(), TemporalWakeRetirementReason::Disposed);
        }
        Ok(report)
    }

    pub fn admit_scheduled_resource_retry(
        &mut self,
        handle: ResourceRequestHandle,
        ready_wake: ReadyTemporalWake,
    ) -> Result<ResourceRetryAdmissionReport, crate::data::error::SignalError> {
        let capture_telemetry = self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        );
        let prepared_retry = match self.resource.prepare_scheduled_resource_retry(
            handle,
            &ready_wake,
            capture_telemetry.then_some(&mut self.telemetry.resource),
        ) {
            Ok(prepared) => prepared,
            Err(report) => return Ok(report),
        };
        let retry_lineage = Some(prepared_retry.previous().clone());
        let current_tick = self.clock_basis().current_tick();
        let resource_node = retry_lineage.as_ref().map(|in_flight| in_flight.node());
        let timeout_plan = resource_node
            .and_then(|node| self.resource.descriptor_for_node(node))
            .map(|descriptor| descriptor.timeout_decision_plan().clone())
            .unwrap_or_else(LoweredResourceDescriptor::default_timeout_decision_plan);
        let prior_timeout_wake =
            resource_node.and_then(|node| self.resource.active_timeout_wake_for_node(node));
        let prior_stale_after_wake =
            resource_node.and_then(|node| self.resource.active_stale_after_wake_for_node(node));
        let timeout_resolution = match retry_lineage {
            Some(in_flight) => {
                Some(self.resolve_retry_timeout_admission(in_flight, &timeout_plan)?)
            }
            None => None,
        };
        let scheduled_timeout_wake = match (resource_node, timeout_resolution.as_ref()) {
            (Some(node), Some(Ok(resolved))) => {
                Some(self.schedule_resource_timeout_wake(node, resolved)?)
            }
            _ => None,
        };
        if !matches!(
            &timeout_resolution,
            Some(Err(
                RetryTimeoutAdmissionResolution::InheritedDeadlineExhausted
            ))
        ) {
            self.retire_superseded_resource_timeout_wake(
                prior_timeout_wake,
                scheduled_timeout_wake.as_ref(),
            )?;
            self.retire_superseded_resource_stale_after_wake(prior_stale_after_wake, None)?;
        }

        let wake_id = ready_wake.id();
        if self
            .resource
            .pending_retry_wake_for_handle(handle)
            .is_some_and(|pending| pending == wake_id)
        {
            self.retire_temporal_wake(wake_id, TemporalWakeRetirementReason::Consumed)?;
        }

        if matches!(
            &timeout_resolution,
            Some(Err(
                RetryTimeoutAdmissionResolution::InheritedDeadlineExhausted
            ))
        ) {
            return Ok(self.resource.deny_resource_retry_admission_for_report(
                handle,
                ResourceRetryDenialClass::RetryTimeoutWindowExhausted,
                capture_telemetry.then_some(&mut self.telemetry.resource),
            ));
        }

        let resolved_timeout = match timeout_resolution {
            Some(Ok(resolved)) => Some(resolved),
            Some(Err(RetryTimeoutAdmissionResolution::Disabled)) | None => None,
            Some(Err(RetryTimeoutAdmissionResolution::InheritedDeadlineExhausted)) => None,
        };
        let scheduled_timeout_admission = resolved_timeout.map(|resolved| {
            resolved.bind_scheduled_wake(
                scheduled_timeout_wake
                    .as_ref()
                    .expect("resolved retry timeout must schedule one temporal wake")
                    .id(),
            )
        });
        let report = self.resource.admit_prepared_scheduled_resource_retry(
            prepared_retry,
            ready_wake,
            self.graph.current_branch().id,
            current_tick,
            scheduled_timeout_admission,
            capture_telemetry.then_some(&mut self.telemetry.resource),
        );

        if report.denied_retry().is_some()
            && let Some(wake) = scheduled_timeout_wake
        {
            self.dispose_resource_timeout_wake(&wake);
        }

        Ok(report)
    }
}
