mod retained_charge;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::diagnostics::policy::RetentionBudget;
use crate::diagnostics::profile::DiagnosticsTier;
use crate::logic::planner::{EvaluationPlan, SessionScratch, TaskReason};

pub type TaskReasonCounts = BTreeMap<TaskReason, u32>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationPlanSummary {
    pub profile: DiagnosticsTier,
    pub requested_target_count: u32,
    pub stage_count: u32,
    pub task_count: u32,
    pub max_stage_width: u32,
    pub contract_pruned_count: u32,
    pub stage_widths: Vec<u32>,
    pub direct_request_count: u32,
    pub transitive_task_count: u32,
    pub task_reason_counts: TaskReasonCounts,
}

impl EvaluationPlanSummary {
    pub fn from_plan(plan: &EvaluationPlan, profile: DiagnosticsTier) -> Self {
        Self::from_components(
            plan.summary.requested_target_count,
            plan.summary.stage_count,
            plan.summary.task_count,
            plan.summary.max_stage_width,
            plan.summary.contract_pruned_count,
            plan.stages.iter().map(|stage| stage.tasks.len() as u32),
            plan.stages.iter().flat_map(|stage| stage.tasks.iter()),
            profile,
        )
    }

    pub(crate) fn from_session(session: &SessionScratch<'_>, profile: DiagnosticsTier) -> Self {
        Self::from_components(
            session.summary.requested_target_count,
            session.summary.stage_count,
            session.summary.task_count,
            session.summary.max_stage_width,
            session.summary.contract_pruned_count,
            session
                .stages
                .iter()
                .map(|stage| (stage.end - stage.start) as u32),
            session.tasks.iter(),
            profile,
        )
    }

    fn from_components<'a>(
        requested_target_count: u32,
        stage_count: u32,
        task_count: u32,
        max_stage_width: u32,
        contract_pruned_count: u32,
        stage_widths_iter: impl Iterator<Item = u32>,
        tasks_iter: impl Iterator<Item = &'a crate::logic::planner::EligibleTask>,
        profile: DiagnosticsTier,
    ) -> Self {
        let mut task_reason_counts = TaskReasonCounts::new();
        let mut direct_request_count = 0_u32;
        let mut stage_widths = stage_widths_iter.collect::<Vec<_>>();
        for task in tasks_iter {
            *task_reason_counts.entry(task.reason).or_insert(0) += 1;
            if task.direct_request {
                direct_request_count += 1;
            }
        }
        let detail_limit = RetentionBudget::for_tier(profile).detail_limit.get();
        if stage_widths.len() > detail_limit {
            stage_widths.truncate(detail_limit);
        }
        Self {
            profile,
            requested_target_count,
            stage_count,
            task_count,
            max_stage_width,
            contract_pruned_count,
            stage_widths,
            direct_request_count,
            transitive_task_count: task_count.saturating_sub(direct_request_count),
            task_reason_counts,
        }
    }
}

impl EvaluationPlan {
    pub fn diagnostics_summary(&self, profile: DiagnosticsTier) -> EvaluationPlanSummary {
        EvaluationPlanSummary::from_plan(self, profile)
    }
}
