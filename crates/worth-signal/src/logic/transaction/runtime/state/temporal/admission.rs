use crate::data::error::SignalError;
use crate::data::node::EvaluationCondition;
use crate::data::temporal::{
    ClockTick, ScheduledTemporalWake, TemporalCondition, TemporalWakeAdmissionSummary,
    TemporalWakeOwner,
};
use crate::logic::planner::{EvaluationPlan, TemporalLoweringContext};

use super::super::runtime_state::SignalRuntime;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn schedule_temporal_wake(
        &mut self,
        condition: TemporalCondition,
        due_tick: ClockTick,
    ) -> Result<ScheduledTemporalWake, SignalError> {
        let telemetry = self
            .graph
            .captures_observation_surface(
                crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
            )
            .then_some(&mut self.telemetry.temporal);
        self.temporal.schedule_wake(condition, due_tick, telemetry)
    }

    pub fn schedule_owned_temporal_wake(
        &mut self,
        owner: TemporalWakeOwner,
        condition: TemporalCondition,
        due_tick: ClockTick,
    ) -> Result<ScheduledTemporalWake, SignalError> {
        self.validate_temporal_wake_owner(owner)?;
        self.temporal.schedule_owned_wake(
            owner,
            condition,
            due_tick,
            self.graph
                .captures_observation_surface(
                    crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
                )
                .then_some(&mut self.telemetry.temporal),
        )
    }

    pub(super) fn validate_temporal_wake_owner(
        &self,
        owner: TemporalWakeOwner,
    ) -> Result<(), SignalError> {
        match owner {
            TemporalWakeOwner::Manual => Ok(()),
            TemporalWakeOwner::Node(node) | TemporalWakeOwner::ResourceNode(node) => {
                if self.graph.is_alive(node) {
                    Ok(())
                } else {
                    Err(SignalError::invalid_input(format!(
                        "cannot admit temporal wake for non-live node owner {node}"
                    )))
                }
            }
        }
    }

    fn due_tick_for_node_temporal_condition(
        &self,
        condition: &TemporalCondition,
    ) -> Result<Option<ClockTick>, SignalError> {
        let current = self.clock_basis().current_tick();
        let due = match condition {
            TemporalCondition::AtOrAfter(condition) => {
                if current >= condition.tick() {
                    return Ok(None);
                }
                condition.tick()
            }
            TemporalCondition::After(condition) => {
                ClockTick::new(current.get().saturating_add(condition.delay_ms()))
            }
            TemporalCondition::Debounce(condition) => {
                ClockTick::new(current.get().saturating_add(condition.quiet_period_ms()))
            }
            TemporalCondition::Throttle(condition) => {
                ClockTick::new(current.get().saturating_add(condition.window_ms()))
            }
            TemporalCondition::StaleAfter(condition) => {
                ClockTick::new(current.get().saturating_add(condition.stale_after_ms()))
            }
            TemporalCondition::Interval(interval) => match interval.anchor() {
                crate::data::temporal::IntervalAnchor::ExplicitTick(tick) if *tick > current => {
                    *tick
                }
                crate::data::temporal::IntervalAnchor::ExplicitTick(_)
                | crate::data::temporal::IntervalAnchor::Registration
                | crate::data::temporal::IntervalAnchor::FirstEvaluation => {
                    ClockTick::new(current.get().saturating_add(interval.period_ms()))
                }
            },
        };
        Ok(Some(due))
    }

    pub fn admit_node_temporal_wake(
        &mut self,
        node: crate::data::handle::NodeId,
    ) -> Result<Option<ScheduledTemporalWake>, SignalError> {
        let summary = self.admit_node_temporal_wake_with_summary(node)?;
        Ok(summary.scheduled().last().cloned())
    }

    pub fn admit_node_temporal_wake_with_summary(
        &mut self,
        node: crate::data::handle::NodeId,
    ) -> Result<TemporalWakeAdmissionSummary, SignalError> {
        let mut summary = TemporalWakeAdmissionSummary::default();
        if !self.graph.is_alive(node) {
            return Err(SignalError::invalid_input(format!(
                "cannot admit temporal wake for non-live node owner {node}"
            )));
        }
        let EvaluationCondition::Temporal(condition) =
            self.graph.node_eval_config(node)?.condition.clone()
        else {
            return Ok(summary);
        };
        let owner = TemporalWakeOwner::Node(node);
        let Some(due_tick) = self.due_tick_for_node_temporal_condition(&condition)? else {
            return Ok(summary);
        };
        if let Some(active_wake_id) = self.temporal.active_wake_for_owner(owner) {
            return self.admit_existing_node_temporal_wake(
                owner,
                condition,
                due_tick,
                active_wake_id,
            );
        }
        let wake = self.schedule_owned_temporal_wake(owner, condition, due_tick)?;
        summary.record_scheduled(wake);
        Ok(summary)
    }

    fn admit_existing_node_temporal_wake(
        &mut self,
        owner: TemporalWakeOwner,
        condition: TemporalCondition,
        due_tick: ClockTick,
        active_wake_id: crate::data::temporal::TemporalWakeId,
    ) -> Result<TemporalWakeAdmissionSummary, SignalError> {
        let mut summary = TemporalWakeAdmissionSummary::default();
        let Some(active) = self.temporal.scheduled_wake(active_wake_id) else {
            if let Some(ready) = self.temporal.ready_wake_for_owner(owner)
                && ready.condition() != &condition
            {
                let supersession = self.temporal.supersede_wake_with_condition(
                    active_wake_id,
                    condition,
                    due_tick,
                    self.graph
                        .captures_observation_surface(
                            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
                        )
                        .then_some(&mut self.telemetry.temporal),
                )?;
                summary.record_policy_supersession(supersession);
            }
            return Ok(summary);
        };
        if active.condition() != &condition {
            let supersession = self.temporal.supersede_wake_with_condition(
                active_wake_id,
                condition,
                due_tick,
                self.graph
                    .captures_observation_surface(
                        crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
                    )
                    .then_some(&mut self.telemetry.temporal),
            )?;
            summary.record_policy_supersession(supersession);
            return Ok(summary);
        }
        if matches!(condition, TemporalCondition::Debounce(_))
            && active.due_tick() > self.clock_basis().current_tick()
            && due_tick > active.due_tick()
        {
            let reschedule = self.reschedule_temporal_wake(active_wake_id, due_tick)?;
            summary.record_reschedule(reschedule);
        } else {
            let reuse = crate::data::temporal::TemporalWakeReuse::from_scheduled(
                active,
                due_tick,
                self.clock_basis().current_tick(),
            );
            summary.record_reused(reuse);
            self.with_telemetry(|telemetry| telemetry.temporal.wake_reuse_count += 1);
        }
        Ok(summary)
    }

    pub fn admit_temporal_wakes_for_plan(
        &mut self,
        plan: &EvaluationPlan,
    ) -> Result<Vec<ScheduledTemporalWake>, SignalError> {
        let summary = self.admit_temporal_wakes_for_plan_with_summary(plan)?;
        Ok(summary.scheduled().to_vec())
    }

    pub fn admit_temporal_wakes_for_plan_with_summary(
        &mut self,
        plan: &EvaluationPlan,
    ) -> Result<TemporalWakeAdmissionSummary, SignalError> {
        let mut summary = TemporalWakeAdmissionSummary::default();
        for stage in &plan.stages {
            for task in &stage.tasks {
                let node_summary = self.admit_node_temporal_wake_with_summary(task.node)?;
                summary.extend(node_summary);
            }
        }
        Ok(summary)
    }

    pub fn admit_temporal_wakes_for_nodes(
        &mut self,
        nodes: &[crate::data::handle::NodeId],
    ) -> Result<Vec<ScheduledTemporalWake>, SignalError> {
        let summary = self.admit_temporal_wakes_for_nodes_with_summary(nodes)?;
        Ok(summary.scheduled().to_vec())
    }

    pub fn admit_temporal_wakes_for_nodes_with_summary(
        &mut self,
        nodes: &[crate::data::handle::NodeId],
    ) -> Result<TemporalWakeAdmissionSummary, SignalError> {
        let mut summary = TemporalWakeAdmissionSummary::default();
        for node in nodes {
            let node_summary = self.admit_node_temporal_wake_with_summary(*node)?;
            summary.extend(node_summary);
        }
        Ok(summary)
    }

    pub(in crate::logic::transaction::runtime) fn temporal_lowering_context_for_plan(
        &self,
        plan: &EvaluationPlan,
    ) -> TemporalLoweringContext {
        let mut context = TemporalLoweringContext::runtime_clock_basis(self.clock_basis());
        for stage in &plan.stages {
            for task in &stage.tasks {
                if let Some(wake) = self
                    .temporal
                    .ready_wake_for_owner(TemporalWakeOwner::Node(task.node))
                    .filter(|wake| {
                        self.graph
                            .node_eval_config(task.node)
                            .is_ok_and(|config| {
                                matches!(&config.condition, EvaluationCondition::Temporal(condition) if wake.condition() == condition)
                            })
                    })
                {
                    context = context.with_ready_wake(wake);
                }
            }
        }
        context
    }

    pub(in crate::logic::transaction::runtime) fn temporal_lowering_context_for_nodes(
        &self,
        nodes: &[crate::data::handle::NodeId],
    ) -> TemporalLoweringContext {
        let mut context = TemporalLoweringContext::runtime_clock_basis(self.clock_basis());
        for node in nodes {
            if let Some(wake) = self
                .temporal
                .ready_wake_for_owner(TemporalWakeOwner::Node(*node))
                .filter(|wake| {
                    self.graph.node_eval_config(*node).is_ok_and(|config| {
                        matches!(&config.condition, EvaluationCondition::Temporal(condition) if wake.condition() == condition)
                    })
                })
            {
                context = context.with_ready_wake(wake);
            }
        }
        context
    }
}
