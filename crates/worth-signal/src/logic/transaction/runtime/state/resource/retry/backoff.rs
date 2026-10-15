use super::super::ResourceRuntimeState;
use super::budget::ResourceRetryBudgetCharge;
use crate::data::resource::*;
use crate::data::telemetry::ResourceTelemetry;

impl ResourceRuntimeState {
    pub fn retry_backoff_delay_for_handle(
        &self,
        handle: ResourceRequestHandle,
        current_tick: crate::data::temporal::ClockTick,
        mut telemetry: Option<&mut ResourceTelemetry>,
    ) -> Result<
        (
            crate::data::temporal::TemporalDuration,
            ResourceAttemptId,
            crate::data::resource::ResourcePolicyDigest,
            Option<ResourceRetryBudgetCharge>,
        ),
        ResourceRetryDenialClass,
    > {
        if let Some(telemetry) = telemetry.as_deref_mut() {
            telemetry.resource_retry_policy_decision_count += 1;
        }
        let in_flight = self
            .in_flight_by_request
            .get(&handle.request_id())
            .cloned()
            .filter(|in_flight| in_flight.handle() == handle)
            .ok_or(ResourceRetryDenialClass::UnknownOrStaleRequest)?;

        if in_flight.status() != ResourceInFlightStatus::TimedOut
            || in_flight.lifecycle() != ResourceLifecycleClass::TimedOut
        {
            return Err(ResourceRetryDenialClass::NonRetryableRequest);
        }
        if self
            .pending_retry_by_request
            .contains_key(&handle.request_id())
        {
            return Err(ResourceRetryDenialClass::RetryAlreadyScheduled);
        }

        let descriptor = self
            .descriptors
            .get(&in_flight.descriptor_id())
            .ok_or(ResourceRetryDenialClass::UnknownOrStaleRequest)?;
        if descriptor
            .timeout_decision_plan()
            .retry_window_exhausted(current_tick, in_flight.generation_started_tick())
        {
            return Err(ResourceRetryDenialClass::RetryTimeoutWindowExhausted);
        }
        let retry_plan = descriptor.retry_decision_plan();
        let next_attempt = in_flight.attempt().next();
        if !retry_plan.admits_attempt(next_attempt) {
            return Err(ResourceRetryDenialClass::RetryAttemptLimitReached);
        }
        if retry_plan.max_jitter().is_some()
            && let Some(telemetry) = telemetry
        {
            telemetry.resource_retry_jitter_decision_count += 1;
        }
        let retry_budget_charge = self.retry_budget_ledger.charge_for(
            &in_flight,
            retry_plan.retry_budget_scope(),
            retry_plan.retry_budget_limit(),
        );
        if retry_budget_charge.is_some_and(|charge| charge.spent_before() >= charge.limit()) {
            return Err(ResourceRetryDenialClass::RetryBudgetExhausted);
        }
        let scheduled_delay = retry_plan
            .delay_for_attempt(in_flight.handle(), next_attempt)
            .ok_or(ResourceRetryDenialClass::RetryPolicyDisabled)?;
        Ok((
            scheduled_delay,
            next_attempt,
            retry_plan.decision_digest().clone(),
            retry_budget_charge,
        ))
    }
    pub(in crate::logic::transaction::runtime::state::resource) fn retry_policy_decision_digest_for_request(
        &self,
        request_id: ResourceRequestId,
    ) -> ResourcePolicyDigest {
        if let Some(scheduled) = self.pending_retry_by_request.get(&request_id) {
            return scheduled.policy_decision_digest().clone();
        }
        if let Some(in_flight) = self.in_flight_by_request.get(&request_id)
            && let Some(descriptor) = self.descriptors.get(&in_flight.descriptor_id())
        {
            return descriptor.retry_decision_plan().decision_digest().clone();
        }
        ResourcePolicyDigest::new("resource-retry-policy-unavailable")
    }
}
