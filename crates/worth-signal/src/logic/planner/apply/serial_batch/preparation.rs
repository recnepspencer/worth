use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::proof::{ClassifiedSnapshotBatchCommit, SnapshotBatchCommit};
use crate::data::trace::RuntimeArtifactFinalizeImage;
use crate::logic::evaluation::{EffectDependencyInputs, PendingDependencySnapshot};
use crate::logic::explain::RewiringSummary;
use crate::logic::planner::semantic::StageSemanticIdentity;
use crate::logic::planner::types::{EligibleTask, ExecutionRecordId, StageExecutionRecord};
use crate::logic::prepared::PreparedEvaluation;

use super::lowered_stage::LoweredSerialStage;
use super::witness::{ExactStageWidth, StageTaskOrderProof};

#[derive(Debug, Clone)]
pub(super) struct SerialApplyInput {
    pub(super) node: NodeId,
    pub(super) record_id: ExecutionRecordId,
    pub(super) prepared: PreparedEvaluation,
    pub(super) dependency_updates: u32,
    pub(super) dependency_inputs: EffectDependencyInputs,
}

impl SerialApplyInput {
    pub(super) fn new(
        node: NodeId,
        record_id: ExecutionRecordId,
        prepared: PreparedEvaluation,
        dependency_updates: u32,
        dependency_inputs: EffectDependencyInputs,
    ) -> Self {
        Self {
            node,
            record_id,
            prepared,
            dependency_updates,
            dependency_inputs,
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::logic::planner) struct SerialFinalizeSeed {
    pub(in crate::logic::planner) task_index: usize,
    pub(in crate::logic::planner) node: NodeId,
    pub(in crate::logic::planner) identity: StageSemanticIdentity,
    pub(in crate::logic::planner) before_state: NodeState,
    pub(in crate::logic::planner) before_artifact_state: Option<RuntimeArtifactFinalizeImage>,
    pub(in crate::logic::planner) dependency_updates: u32,
    pub(in crate::logic::planner) recomputed: bool,
    pub(in crate::logic::planner) partition_aware: bool,
    pub(in crate::logic::planner) rewiring: Option<RewiringSummary>,
}

impl SerialFinalizeSeed {
    pub(super) fn from_execution_parts(
        task_index: usize,
        node: NodeId,
        identity: StageSemanticIdentity,
        before_state: NodeState,
        before_artifact_state: Option<RuntimeArtifactFinalizeImage>,
        dependency_updates: u32,
        recomputed: bool,
        partition_aware: bool,
        rewiring: Option<RewiringSummary>,
    ) -> Self {
        Self {
            task_index,
            node,
            identity,
            before_state,
            before_artifact_state,
            dependency_updates,
            recomputed,
            partition_aware,
            rewiring,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(in crate::logic::planner) struct DeferredSnapshotBatch {
    pub(super) pending_snapshots: Vec<PendingDependencySnapshot>,
}

impl DeferredSnapshotBatch {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            pending_snapshots: Vec::with_capacity(capacity),
        }
    }

    pub(in crate::logic::planner) fn push(&mut self, snapshot: PendingDependencySnapshot) {
        self.pending_snapshots.push(snapshot);
    }

    pub(in crate::logic::planner) fn len(&self) -> usize {
        self.pending_snapshots.len()
    }

    pub(in crate::logic::planner) fn classify(self) -> ClassifiedSnapshotBatchCommit {
        SnapshotBatchCommit::from_unique_pending_snapshots_in_stage_order(self.pending_snapshots)
            .classify()
    }
}

#[derive(Debug, Clone)]
pub(in crate::logic::planner) struct PreparedSerialStageBatch {
    pub(super) stage_index: u32,
    pub(super) exact_width: ExactStageWidth,
    pub(super) stage_tasks: Vec<EligibleTask>,
    pub(super) finalize_seeds: Vec<SerialFinalizeSeed>,
    pub(super) apply_inputs: Vec<SerialApplyInput>,
    pub(super) pending_snapshots: DeferredSnapshotBatch,
    pub(super) stage_order: StageTaskOrderProof,
}

impl PreparedSerialStageBatch {
    pub(in crate::logic::planner) fn prepare(
        graph: &mut SignalGraph,
        lowered: LoweredSerialStage,
        stage_record: &mut StageExecutionRecord,
    ) -> Result<Self, SignalError> {
        stage_record.apply_mode = Some(crate::logic::planner::ParallelApplyMode::SerialApply);
        stage_record.apply_group_count = 1;
        stage_record.serial_apply_rejection_reason = lowered.serial_rejection_reason();
        stage_record.serial_fallback_group_count =
            u32::from(lowered.serial_rejection_reason().is_some());
        stage_record.serial_apply_task_count = lowered.stage_width() as u32;

        let mut reconcile_batch = Vec::with_capacity(lowered.exact_width.get());
        for task in &lowered.lowered_tasks {
            reconcile_batch.push((task.node, task.desired_dependencies.as_slice()));
        }
        let reconcile_start = crate::clock::RuntimeInstant::now();
        graph.reconcile_dependencies_batch_borrowed(&reconcile_batch)?;
        let reconcile_nanos = reconcile_start.elapsed().as_nanos();
        graph.with_telemetry(|telemetry| {
            telemetry.execution.dependency_reconcile_nanos += reconcile_nanos;
        });

        let dependency_input_start = crate::clock::RuntimeInstant::now();
        let dependency_inputs = crate::logic::evaluation::collect_effect_dependency_inputs_iter(
            graph,
            lowered.lowered_tasks.iter().map(|task| task.node),
        )?;
        let dependency_input_nanos = dependency_input_start.elapsed().as_nanos();
        graph.with_telemetry(|telemetry| {
            telemetry.execution.dependency_input_build_nanos += dependency_input_nanos;
        });

        let apply_inputs = lowered
            .lowered_tasks
            .into_iter()
            .zip(dependency_inputs)
            .map(|(task, dependency_inputs)| {
                SerialApplyInput::new(
                    task.node,
                    task.record_id,
                    task.prepared,
                    task.dependency_updates,
                    dependency_inputs,
                )
            })
            .collect();

        Ok(Self {
            stage_index: lowered.stage_index,
            exact_width: lowered.exact_width,
            stage_tasks: lowered.stage_tasks,
            finalize_seeds: lowered.finalize_seeds,
            apply_inputs,
            pending_snapshots: DeferredSnapshotBatch::with_capacity(lowered.exact_width.get()),
            stage_order: lowered.stage_order,
        })
    }
}
