use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::node::EvaluationCondition;
use crate::data::temporal::{
    ClockTick, LoweredTemporalEligibility, ScheduledTemporalWake, TemporalCondition,
    TemporalPreviousValueAccess, TemporalPreviousValueReference, TemporalWakeId, TemporalWakeOwner,
    TemporalWakeRetirementReason,
};
use crate::logic::planner::{EvaluationPlan, ExecutionReport, TemporalLoweringContext};
use std::collections::BTreeSet;

use super::super::super::transaction::SignalTransaction;

impl<'a, D, I, E, Ctx, T> SignalTransaction<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    Ctx: Sync,
    T: Copy + Ord,
{
    pub fn grant_temporal_previous_value_access(
        &mut self,
        wake_id: TemporalWakeId,
    ) -> Result<TemporalPreviousValueAccess, SignalError> {
        let Some(owner) = self.temporal.active_wake_owner(wake_id) else {
            return Err(SignalError::invalid_input(format!(
                "cannot grant previous-value access from inactive temporal wake {}",
                wake_id.get()
            )));
        };
        if let TemporalWakeOwner::Node(node) = owner
            && !self.graph.is_alive(node)
        {
            return Err(SignalError::invalid_input(format!(
                "cannot grant previous-value access from wake {} owned by non-live node {}",
                wake_id.get(),
                node
            )));
        }
        self.temporal
            .grant_previous_value_access(self.graph.current_branch().id, wake_id)
    }

    pub fn previous_temporal_value(
        &mut self,
        access: &TemporalPreviousValueAccess,
        node: NodeId,
    ) -> Result<TemporalPreviousValueReference, SignalError> {
        let current_branch = self.graph.current_branch();
        if access.branch_id() != current_branch.id {
            return Err(SignalError::invalid_input(format!(
                "temporal previous-value access belongs to branch {} but current branch is {}",
                access.branch_id().0,
                current_branch.id.0
            )));
        }
        let aspect_version = self.graph.node_aspect_version(node)?;
        let output_identity = self
            .graph
            .observe()
            .runtime_artifact_warm(node)?
            .and_then(|warm| warm.output_identity.clone());
        let reference = self.temporal.capture_previous_value_reference(
            access,
            node,
            aspect_version,
            output_identity,
            self.telemetry
                .as_deref_mut()
                .map(|telemetry| &mut telemetry.temporal),
        )?;
        self.scratch
            .temporal
            .record_previous_value_reference(reference.clone());
        Ok(reference)
    }

    fn due_tick_for_node_temporal_condition(
        &self,
        condition: &TemporalCondition,
    ) -> Result<Option<ClockTick>, SignalError> {
        let current = self.temporal.clock_basis().current_tick();
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

    fn admit_node_temporal_wake(
        &mut self,
        node: NodeId,
    ) -> Result<Option<ScheduledTemporalWake>, SignalError> {
        if !self.graph.is_alive(node) {
            return Err(SignalError::invalid_input(format!(
                "cannot admit temporal wake for non-live node owner {node}"
            )));
        }
        let EvaluationCondition::Temporal(condition) =
            self.graph.node_eval_config(node)?.condition.clone()
        else {
            return Ok(None);
        };
        let owner = TemporalWakeOwner::Node(node);
        let Some(due_tick) = self.due_tick_for_node_temporal_condition(&condition)? else {
            return Ok(None);
        };
        if let Some(active_wake_id) = self.temporal.active_wake_for_owner(owner) {
            let Some(active) = self.temporal.scheduled_wake(active_wake_id) else {
                if let Some(ready) = self.temporal.ready_wake_for_owner(owner)
                    && ready.condition() != &condition
                {
                    let supersession = self.temporal.supersede_wake_with_condition(
                        active_wake_id,
                        condition,
                        due_tick,
                        self.telemetry
                            .as_deref_mut()
                            .map(|telemetry| &mut telemetry.temporal),
                    )?;
                    self.scratch
                        .temporal
                        .record_rescheduled_wake(supersession.clone());
                    self.scratch
                        .temporal
                        .record_retired_wake(supersession.retired().clone());
                    self.scratch
                        .temporal
                        .record_scheduled_wake(supersession.scheduled().clone());
                    return Ok(Some(supersession.scheduled().clone()));
                }
                return Ok(None);
            };
            if active.condition() != &condition {
                let supersession = self.temporal.supersede_wake_with_condition(
                    active_wake_id,
                    condition,
                    due_tick,
                    self.telemetry
                        .as_deref_mut()
                        .map(|telemetry| &mut telemetry.temporal),
                )?;
                self.scratch
                    .temporal
                    .record_rescheduled_wake(supersession.clone());
                self.scratch
                    .temporal
                    .record_retired_wake(supersession.retired().clone());
                self.scratch
                    .temporal
                    .record_scheduled_wake(supersession.scheduled().clone());
                return Ok(Some(supersession.scheduled().clone()));
            }
            if matches!(condition, TemporalCondition::Debounce(_))
                && active.due_tick() > self.temporal.clock_basis().current_tick()
                && due_tick > active.due_tick()
            {
                let reschedule = self.temporal.reschedule_wake(
                    active_wake_id,
                    due_tick,
                    self.telemetry
                        .as_deref_mut()
                        .map(|telemetry| &mut telemetry.temporal),
                )?;
                self.scratch
                    .temporal
                    .record_rescheduled_wake(reschedule.clone());
                self.scratch
                    .temporal
                    .record_retired_wake(reschedule.retired().clone());
                self.scratch
                    .temporal
                    .record_scheduled_wake(reschedule.scheduled().clone());
            } else {
                let reuse = crate::data::temporal::TemporalWakeReuse::from_scheduled(
                    active,
                    due_tick,
                    self.temporal.clock_basis().current_tick(),
                );
                self.scratch.temporal.record_reused_wake(reuse);
                self.with_telemetry(|telemetry| telemetry.temporal.wake_reuse_count += 1);
            }
            return Ok(None);
        }
        let wake = self.temporal.schedule_owned_wake(
            owner,
            condition,
            due_tick,
            self.telemetry
                .as_deref_mut()
                .map(|telemetry| &mut telemetry.temporal),
        )?;
        self.scratch.temporal.record_scheduled_wake(wake.clone());
        Ok(Some(wake))
    }

    pub(super) fn admit_temporal_wakes_for_plan(
        &mut self,
        plan: &EvaluationPlan,
    ) -> Result<Vec<ScheduledTemporalWake>, SignalError> {
        let mut scheduled = Vec::new();
        for stage in &plan.stages {
            for task in &stage.tasks {
                if let Some(wake) = self.admit_node_temporal_wake(task.node)? {
                    scheduled.push(wake);
                }
            }
        }
        Ok(scheduled)
    }

    pub(in crate::logic::transaction::runtime::execution) fn admit_temporal_wakes_for_nodes(
        &mut self,
        nodes: &[NodeId],
    ) -> Result<Vec<ScheduledTemporalWake>, SignalError> {
        let mut scheduled = Vec::new();
        for node in nodes {
            if let Some(wake) = self.admit_node_temporal_wake(*node)? {
                scheduled.push(wake);
            }
        }
        Ok(scheduled)
    }

    pub(in crate::logic::transaction::runtime::execution) fn promote_due_temporal_wakes_ready(
        &mut self,
    ) -> Result<(), SignalError> {
        self.with_telemetry(|telemetry| telemetry.temporal.temporal_broad_scan_denial_count += 1);
        loop {
            let frontier = self.temporal.frontier_snapshot();
            let Some(next_due_tick) = frontier.next_due_tick() else {
                break;
            };
            if next_due_tick > self.temporal.clock_basis().current_tick() {
                break;
            }
            let Some(wake_id) = frontier.next_due_wake_id() else {
                return Err(SignalError::internal(
                    "temporal frontier reported due tick without a due wake id",
                ));
            };
            let ready = self.temporal.promote_wake_ready(
                wake_id,
                self.telemetry
                    .as_deref_mut()
                    .map(|telemetry| &mut telemetry.temporal),
            )?;
            self.scratch.temporal.record_ready_wake(ready);
        }
        Ok(())
    }

    pub(super) fn temporal_lowering_context_for_plan(
        &self,
        plan: &EvaluationPlan,
    ) -> TemporalLoweringContext {
        let mut context = TemporalLoweringContext::runtime_clock_basis(self.temporal.clock_basis());
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

    pub(in crate::logic::transaction::runtime::execution) fn temporal_lowering_context_for_nodes(
        &self,
        nodes: &[NodeId],
    ) -> TemporalLoweringContext {
        let mut context = TemporalLoweringContext::runtime_clock_basis(self.temporal.clock_basis());
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

    pub(in crate::logic::transaction::runtime::execution) fn retire_consumed_temporal_wakes_from_report(
        &mut self,
        report: &ExecutionReport,
    ) -> Result<(), SignalError> {
        let mut consumed = BTreeSet::new();
        for stage in &report.stages {
            for task in &stage.task_records {
                let Some(LoweredTemporalEligibility::Ready(ready)) =
                    task.temporal_eligibility.as_ref()
                else {
                    continue;
                };
                if let Some(wake_id) = ready.wake_id() {
                    consumed.insert(wake_id);
                }
            }
        }
        for wake_id in consumed {
            let Some(owner) = self.temporal.active_wake_owner(wake_id) else {
                continue;
            };
            match self.temporal.ready_wake_for_owner(owner) {
                Some(ready) if matches!(ready.condition(), TemporalCondition::Interval(_)) => {
                    let regeneration = self.temporal.regenerate_interval_wake(
                        wake_id,
                        self.telemetry
                            .as_deref_mut()
                            .map(|telemetry| &mut telemetry.temporal),
                    )?;
                    self.scratch
                        .temporal
                        .record_interval_regeneration(regeneration.clone());
                    self.scratch
                        .temporal
                        .record_retired_wake(regeneration.retired().clone());
                    self.scratch
                        .temporal
                        .record_scheduled_wake(regeneration.scheduled().clone());
                }
                Some(_) => {
                    let retired = self.temporal.retire_wake(
                        wake_id,
                        TemporalWakeRetirementReason::Consumed,
                        self.telemetry
                            .as_deref_mut()
                            .map(|telemetry| &mut telemetry.temporal),
                    )?;
                    self.scratch.temporal.record_retired_wake(retired);
                }
                None => {}
            }
        }
        Ok(())
    }
}
