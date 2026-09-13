use crate::data::resource::{
    InFlightResourceRequest, ResourceNodeId, ResourceRevalidationReport,
    ResourceTimeoutDeadlineAuthority, ResourceTimeoutDecisionPlan,
};

use crate::data::temporal::{
    ScheduledTemporalWake, TemporalCondition, TemporalDuration, TemporalWakeOwner,
    TemporalWakeRetirementReason,
};

use super::super::super::resource::ResolvedResourceTimeoutPlan;

use super::super::SignalRuntime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::logic::transaction::runtime::state::runtime_state) enum RetryTimeoutAdmissionResolution
{
    Disabled,
    InheritedDeadlineExhausted,
}

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::logic::transaction::runtime::state::runtime_state) fn resolve_timeout_admission(
        &mut self,
        resource_node: ResourceNodeId,
        timeout_plan: &ResourceTimeoutDecisionPlan,
        generation_started_tick: crate::data::temporal::ClockTick,
        transaction_deadline: Option<TemporalDuration>,
    ) -> Result<Option<ResolvedResourceTimeoutPlan>, crate::data::error::SignalError> {
        self.with_resource_telemetry(|telemetry| {
            telemetry.resource_timeout_policy_decision_count += 1;
        });
        let current_tick = self.clock_basis().current_tick();
        let (timeout_duration, deadline_authority) = match timeout_plan.class() {
            crate::data::resource::ResourceTimeoutDecisionClass::Disabled => return Ok(None),
            crate::data::resource::ResourceTimeoutDecisionClass::TransactionInheritedDeadline => {
                let Some(deadline) = transaction_deadline else {
                    return Err(crate::data::error::SignalError::invalid_input(format!(
                        "resource node {} requires a transaction-inherited deadline",
                        resource_node.node()
                    )));
                };
                (
                    deadline,
                    ResourceTimeoutDeadlineAuthority::TransactionIntent,
                )
            }
            crate::data::resource::ResourceTimeoutDecisionClass::RuntimeInheritedDeadline => {
                let Some(deadline) = self.config.resource_runtime_deadline() else {
                    return Err(crate::data::error::SignalError::invalid_input(format!(
                        "resource node {} requires a runtime-inherited deadline",
                        resource_node.node()
                    )));
                };
                (deadline, ResourceTimeoutDeadlineAuthority::RuntimeConfig)
            }
            _ => {
                return Ok(
                    super::super::super::resource::resolve_descriptor_timeout_plan(
                        timeout_plan,
                        current_tick,
                        generation_started_tick,
                    ),
                );
            }
        };
        if !matches!(
            deadline_authority,
            ResourceTimeoutDeadlineAuthority::Descriptor
        ) {
            self.with_resource_telemetry(|telemetry| {
                telemetry.resource_deadline_inherited_count += 1
            });
        }
        let due_tick = crate::data::temporal::ClockTick::new(
            current_tick.get().saturating_add(timeout_duration.get()),
        );
        let decision_digest = crate::data::resource::ResourcePolicyDigest::new(format!(
            "resolved-timeout-decision:{}:{}:{}:{}",
            timeout_plan.decision_digest().as_str(),
            timeout_duration.get(),
            timeout_plan.outcome_class().as_str(),
            deadline_authority.as_str()
        ));
        Ok(Some(ResolvedResourceTimeoutPlan::new(
            timeout_duration,
            due_tick,
            timeout_plan.outcome_class(),
            deadline_authority,
            decision_digest,
        )))
    }

    pub(in crate::logic::transaction::runtime::state::runtime_state) fn schedule_resource_timeout_wake(
        &mut self,
        resource_node: ResourceNodeId,
        resolved_timeout: &ResolvedResourceTimeoutPlan,
    ) -> Result<ScheduledTemporalWake, crate::data::error::SignalError> {
        let timeout = resolved_timeout.timeout_duration();
        let condition = TemporalCondition::after(timeout.get())?;
        self.schedule_owned_temporal_wake(
            TemporalWakeOwner::ResourceNode(resource_node.node()),
            condition,
            resolved_timeout.due_tick(),
        )
    }

    pub(in crate::logic::transaction::runtime::state::runtime_state) fn schedule_resource_stale_after_wake(
        &mut self,
        resource_node: ResourceNodeId,
        stale_after: TemporalDuration,
    ) -> Result<Option<ScheduledTemporalWake>, crate::data::error::SignalError> {
        let due_tick = crate::data::temporal::ClockTick::new(
            self.clock_basis()
                .current_tick()
                .get()
                .saturating_add(stale_after.get()),
        );
        self.schedule_owned_temporal_wake(
            TemporalWakeOwner::ResourceNode(resource_node.node()),
            TemporalCondition::after(stale_after.get())?,
            due_tick,
        )
        .map(Some)
    }

    pub(in crate::logic::transaction::runtime::state::runtime_state) fn resolve_retry_timeout_admission(
        &mut self,
        in_flight: InFlightResourceRequest,
        timeout_plan: &ResourceTimeoutDecisionPlan,
    ) -> Result<
        Result<ResolvedResourceTimeoutPlan, RetryTimeoutAdmissionResolution>,
        crate::data::error::SignalError,
    > {
        match in_flight.timeout_deadline_authority() {
            ResourceTimeoutDeadlineAuthority::Descriptor => Ok(self
                .resolve_timeout_admission(
                    in_flight.node(),
                    timeout_plan,
                    in_flight.generation_started_tick(),
                    None,
                )?
                .ok_or(RetryTimeoutAdmissionResolution::Disabled)),
            ResourceTimeoutDeadlineAuthority::TransactionIntent
            | ResourceTimeoutDeadlineAuthority::RuntimeConfig => {
                let Some(due_tick) = in_flight.timeout_due_tick() else {
                    return Err(crate::data::error::SignalError::invalid_input(format!(
                        "resource request {} lost inherited deadline due tick",
                        in_flight.handle().request_id().get()
                    )));
                };
                let current_tick = self.clock_basis().current_tick();
                let remaining = due_tick.get().saturating_sub(current_tick.get());
                if remaining == 0 {
                    return Ok(Err(
                        RetryTimeoutAdmissionResolution::InheritedDeadlineExhausted,
                    ));
                }
                let timeout_duration = TemporalDuration::temporal_duration(remaining)
                    .expect("positive inherited deadline remainder must stay valid");
                let decision_digest = crate::data::resource::ResourcePolicyDigest::new(format!(
                    "resolved-timeout-decision:{}:{}:{}:{}",
                    timeout_plan.decision_digest().as_str(),
                    timeout_duration.get(),
                    timeout_plan.outcome_class().as_str(),
                    in_flight.timeout_deadline_authority().as_str()
                ));
                Ok(Ok(ResolvedResourceTimeoutPlan::new(
                    timeout_duration,
                    due_tick,
                    timeout_plan.outcome_class(),
                    in_flight.timeout_deadline_authority(),
                    decision_digest,
                )))
            }
        }
    }

    pub(in crate::logic::transaction::runtime::state::runtime_state) fn dispose_resource_timeout_wake(
        &mut self,
        scheduled_timeout_wake: &ScheduledTemporalWake,
    ) {
        let _ = self.retire_temporal_wake(
            scheduled_timeout_wake.id(),
            TemporalWakeRetirementReason::Disposed,
        );
    }

    pub(in crate::logic::transaction::runtime::state::runtime_state) fn dispose_resource_stale_after_wake(
        &mut self,
        scheduled_stale_after_wake: &ScheduledTemporalWake,
    ) {
        let _ = self.retire_temporal_wake(
            scheduled_stale_after_wake.id(),
            TemporalWakeRetirementReason::Disposed,
        );
    }

    pub(in crate::logic::transaction::runtime::state::runtime_state) fn retire_superseded_resource_timeout_wake(
        &mut self,
        prior_timeout_wake: Option<crate::data::temporal::TemporalWakeId>,
        scheduled_timeout_wake: Option<&ScheduledTemporalWake>,
    ) -> Result<(), crate::data::error::SignalError> {
        if let Some(wake_id) = prior_timeout_wake {
            if let Err(err) =
                self.retire_temporal_wake(wake_id, TemporalWakeRetirementReason::Superseded)
            {
                if let Some(wake) = scheduled_timeout_wake {
                    self.dispose_resource_timeout_wake(wake);
                }
                return Err(err);
            }
        }
        Ok(())
    }

    pub(in crate::logic::transaction::runtime::state::runtime_state) fn reconcile_resource_revalidation_wakes(
        &mut self,
        report: &ResourceRevalidationReport,
        resource_node: ResourceNodeId,
        prior_timeout_wake: Option<crate::data::temporal::TemporalWakeId>,
        prior_stale_after_wake: Option<crate::data::temporal::TemporalWakeId>,
        scheduled_timeout_wake: Option<ScheduledTemporalWake>,
    ) -> Result<(), crate::data::error::SignalError> {
        if report.admitted_revalidation().is_none() {
            if let Some(wake) = scheduled_timeout_wake.as_ref() {
                self.dispose_resource_timeout_wake(wake);
            }
            return Ok(());
        }

        self.retire_superseded_resource_timeout_wake(prior_timeout_wake, None)?;
        self.retire_superseded_resource_stale_after_wake(prior_stale_after_wake, None)?;
        if prior_stale_after_wake.is_some() {
            self.resource.clear_stale_after_wake_for_node(resource_node);
        }
        Ok(())
    }

    pub(in crate::logic::transaction::runtime::state::runtime_state) fn retire_superseded_resource_stale_after_wake(
        &mut self,
        prior_stale_after_wake: Option<crate::data::temporal::TemporalWakeId>,
        scheduled_stale_after_wake: Option<&ScheduledTemporalWake>,
    ) -> Result<(), crate::data::error::SignalError> {
        if let Some(wake_id) = prior_stale_after_wake {
            if let Err(err) =
                self.retire_temporal_wake(wake_id, TemporalWakeRetirementReason::Superseded)
            {
                if let Some(wake) = scheduled_stale_after_wake {
                    self.dispose_resource_stale_after_wake(wake);
                }
                return Err(err);
            }
        }
        Ok(())
    }

    pub(in crate::logic::transaction::runtime::state::runtime_state) fn retire_superseded_resource_retry_wake(
        &mut self,
        prior_retry_wake: Option<crate::data::temporal::TemporalWakeId>,
    ) -> Result<(), crate::data::error::SignalError> {
        if let Some(wake_id) = prior_retry_wake {
            self.retire_temporal_wake(wake_id, TemporalWakeRetirementReason::Superseded)?;
        }
        Ok(())
    }
}
