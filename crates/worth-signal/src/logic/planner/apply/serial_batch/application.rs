use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::node::NodeState;
use crate::data::output::MemoizedResultOrigin;
use crate::data::reuse::ReuseBasis;
use crate::data::temporal::LoweredTemporalEligibility;
use crate::diagnostics::failure::{ExecutionFailureContext, ExecutionFailurePhase};
use crate::logic::evaluation::{
    apply_prepared_evaluation_after_dependencies_with_policy, EvaluationVerdict,
};
use crate::logic::planner::types::{EligibleTask, StageExecutor};

use super::preparation::{DeferredSnapshotBatch, PreparedSerialStageBatch, SerialApplyInput};
use super::witness::ExactStageWidth;
use crate::logic::planner::execution::task_reporting::record_execution_failure_if_enabled;
use crate::logic::planner::types::PlanSummary;

#[derive(Debug, Clone)]
pub(in crate::logic::planner) struct AppliedSerialTask {
    pub(in crate::logic::planner) node: crate::data::handle::NodeId,
    pub(in crate::logic::planner) verdict: EvaluationVerdict,
    pub(in crate::logic::planner) after_state: NodeState,
    pub(in crate::logic::planner) temporal_eligibility: Option<LoweredTemporalEligibility>,
    pub(in crate::logic::planner) memoized_origin: MemoizedResultOrigin,
    pub(in crate::logic::planner) reuse_basis: ReuseBasis,
}

impl AppliedSerialTask {
    pub(super) fn from_apply_result(
        graph: &SignalGraph,
        node: crate::data::handle::NodeId,
        verdict: EvaluationVerdict,
        temporal_eligibility: Option<LoweredTemporalEligibility>,
    ) -> Result<Self, SignalError> {
        let after_state = graph.get_state(node)?;
        let after_trace = graph.node_runtime_artifact_operational_summary(node)?;
        Ok(Self {
            node,
            verdict,
            after_state,
            temporal_eligibility,
            memoized_origin: after_trace
                .as_ref()
                .map(|trace| trace.memoized_origin)
                .unwrap_or(MemoizedResultOrigin::DirectCompute),
            reuse_basis: after_trace
                .map(|trace| trace.reuse_basis)
                .unwrap_or_else(ReuseBasis::fresh_compute),
        })
    }
}

impl PreparedSerialStageBatch {
    pub(in crate::logic::planner) fn apply(
        mut self,
        graph: &mut SignalGraph,
        summary: &PlanSummary,
        comparator_resolver: &mut impl crate::data::comparator::ComparatorPolicyResolver,
        executor: StageExecutor,
    ) -> Result<AppliedSerialStageBatch, SignalError> {
        let stage_index = self.stage_index;
        let mut applied_tasks = Vec::with_capacity(self.exact_width.get());
        for input in self.apply_inputs {
            let apply_result = apply_serial_input(
                graph,
                summary,
                stage_index,
                input,
                executor,
                comparator_resolver,
            )?;
            if let Some(snapshot) = apply_result.pending_snapshot {
                self.pending_snapshots.push(snapshot);
            }
            applied_tasks.push(AppliedSerialTask::from_apply_result(
                graph,
                apply_result.node,
                apply_result.verdict,
                apply_result.temporal_eligibility,
            )?);
        }

        let applied_tasks = StageOrderedAppliedTasks::new(self.exact_width, applied_tasks)?;
        let task_count = applied_tasks.len();
        let publication_breadth = (task_count + self.pending_snapshots.len()) as u64;
        graph.with_telemetry(|telemetry| {
            telemetry.execution.group_local_packet_breadth += task_count as u64;
            telemetry.execution.reduction_packet_breadth += 1;
            telemetry.execution.reduction_group_count += 1;
            telemetry.execution.shared_surface_publication_breadth += publication_breadth;
        });

        Ok(AppliedSerialStageBatch {
            stage_index: self.stage_index,
            exact_width: self.exact_width,
            stage_tasks: self.stage_tasks,
            finalize_seeds: self.finalize_seeds,
            applied_tasks,
            pending_snapshots: self.pending_snapshots,
            stage_order: self.stage_order,
        })
    }
}

struct SerialApplyResult {
    node: crate::data::handle::NodeId,
    verdict: EvaluationVerdict,
    temporal_eligibility: Option<LoweredTemporalEligibility>,
    pending_snapshot: Option<crate::logic::evaluation::PendingDependencySnapshot>,
}

fn apply_serial_input(
    graph: &mut SignalGraph,
    summary: &PlanSummary,
    stage_index: u32,
    input: SerialApplyInput,
    executor: StageExecutor,
    comparator_resolver: &mut impl crate::data::comparator::ComparatorPolicyResolver,
) -> Result<SerialApplyResult, SignalError> {
    let node = input.node;
    let record_id = input.record_id;
    let apply_result = apply_prepared_evaluation_after_dependencies_with_policy(
        graph,
        node,
        input.prepared,
        comparator_resolver,
        None,
        input.dependency_updates,
        Some(input.dependency_inputs),
        false,
        &mut crate::logic::evaluation::EvaluationWork::Ordinary,
    )
    .inspect_err(|err| {
        record_execution_failure_if_enabled(graph, || {
            ExecutionFailureContext::new(
                ExecutionFailurePhase::Apply,
                Some(stage_index),
                Some(node),
                Some(executor),
                Some(record_id),
                Some(*summary),
                err.to_string(),
            )
        });
    })?;
    Ok(SerialApplyResult {
        node,
        verdict: apply_result.report.verdict,
        temporal_eligibility: apply_result.temporal_eligibility,
        pending_snapshot: apply_result.pending_snapshot,
    })
}

#[derive(Debug, Clone)]
pub(super) struct StageOrderedAppliedTasks {
    pub(super) exact_width: ExactStageWidth,
    pub(super) tasks: Vec<AppliedSerialTask>,
}

impl StageOrderedAppliedTasks {
    pub(super) fn new(
        exact_width: ExactStageWidth,
        tasks: Vec<AppliedSerialTask>,
    ) -> Result<Self, SignalError> {
        if tasks.len() != exact_width.get() {
            return Err(SignalError::internal(
                "serial batch apply must produce exactly one ordered applied task per prepared input",
            ));
        }
        Ok(Self { exact_width, tasks })
    }

    pub(super) fn len(&self) -> usize {
        self.tasks.len()
    }

    pub(super) fn exact_width(&self) -> ExactStageWidth {
        self.exact_width
    }
}

#[derive(Debug, Clone)]
pub(in crate::logic::planner) struct AppliedSerialStageBatch {
    pub(super) stage_index: u32,
    pub(super) exact_width: ExactStageWidth,
    pub(super) stage_tasks: Vec<EligibleTask>,
    pub(super) finalize_seeds: Vec<super::preparation::SerialFinalizeSeed>,
    pub(super) applied_tasks: StageOrderedAppliedTasks,
    pub(super) pending_snapshots: DeferredSnapshotBatch,
    pub(super) stage_order: super::witness::StageTaskOrderProof,
}
