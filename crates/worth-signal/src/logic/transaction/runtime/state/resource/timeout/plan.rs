use crate::data::resource::*;
use crate::data::temporal::TemporalWakeId;

#[derive(Debug, Clone)]
pub(in crate::logic::transaction::runtime::state) struct ResolvedResourceTimeoutPlan {
    pub(in crate::logic::transaction::runtime::state::resource) timeout_duration:
        crate::data::temporal::TemporalDuration,
    pub(in crate::logic::transaction::runtime::state::resource) due_tick:
        crate::data::temporal::ClockTick,
    pub(in crate::logic::transaction::runtime::state::resource) outcome_class:
        ResourceTimeoutOutcomeClass,
    pub(in crate::logic::transaction::runtime::state::resource) deadline_authority:
        ResourceTimeoutDeadlineAuthority,
    pub(in crate::logic::transaction::runtime::state::resource) decision_digest:
        ResourcePolicyDigest,
}

impl ResolvedResourceTimeoutPlan {
    pub(in crate::logic::transaction::runtime::state) fn new(
        timeout_duration: crate::data::temporal::TemporalDuration,
        due_tick: crate::data::temporal::ClockTick,
        outcome_class: ResourceTimeoutOutcomeClass,
        deadline_authority: ResourceTimeoutDeadlineAuthority,
        decision_digest: ResourcePolicyDigest,
    ) -> Self {
        Self {
            timeout_duration,
            due_tick,
            outcome_class,
            deadline_authority,
            decision_digest,
        }
    }

    pub(in crate::logic::transaction::runtime::state) const fn timeout_duration(
        &self,
    ) -> crate::data::temporal::TemporalDuration {
        self.timeout_duration
    }

    pub(in crate::logic::transaction::runtime::state) const fn due_tick(
        &self,
    ) -> crate::data::temporal::ClockTick {
        self.due_tick
    }

    pub(in crate::logic::transaction::runtime::state) fn bind_scheduled_wake(
        self,
        wake_id: TemporalWakeId,
    ) -> ScheduledResourceTimeoutAdmission {
        ScheduledResourceTimeoutAdmission {
            timeout_duration: self.timeout_duration,
            due_tick: self.due_tick,
            outcome_class: self.outcome_class,
            deadline_authority: self.deadline_authority,
            decision_digest: self.decision_digest,
            wake_id,
        }
    }
}

pub(in crate::logic::transaction::runtime::state) fn resolve_descriptor_timeout_plan(
    plan: &ResourceTimeoutDecisionPlan,
    current_tick: crate::data::temporal::ClockTick,
    generation_started_tick: crate::data::temporal::ClockTick,
) -> Option<ResolvedResourceTimeoutPlan> {
    let timeout_duration = plan.timeout_for_lineage(current_tick, generation_started_tick)?;
    let due_tick = crate::data::temporal::ClockTick::new(
        current_tick.get().saturating_add(timeout_duration.get()),
    );
    let decision_digest = ResourcePolicyDigest::new(format!(
        "resolved-timeout-decision:{}:{}:{}:descriptor",
        plan.decision_digest().as_str(),
        timeout_duration.get(),
        plan.outcome_class().as_str(),
    ));
    Some(ResolvedResourceTimeoutPlan::new(
        timeout_duration,
        due_tick,
        plan.outcome_class(),
        ResourceTimeoutDeadlineAuthority::Descriptor,
        decision_digest,
    ))
}

#[derive(Debug)]
pub(in crate::logic::transaction::runtime::state) struct ScheduledResourceTimeoutAdmission {
    pub(in crate::logic::transaction::runtime::state::resource) timeout_duration:
        crate::data::temporal::TemporalDuration,
    pub(in crate::logic::transaction::runtime::state::resource) due_tick:
        crate::data::temporal::ClockTick,
    pub(in crate::logic::transaction::runtime::state::resource) outcome_class:
        ResourceTimeoutOutcomeClass,
    pub(in crate::logic::transaction::runtime::state::resource) deadline_authority:
        ResourceTimeoutDeadlineAuthority,
    pub(in crate::logic::transaction::runtime::state::resource) decision_digest:
        ResourcePolicyDigest,
    pub(in crate::logic::transaction::runtime::state::resource) wake_id: TemporalWakeId,
}
