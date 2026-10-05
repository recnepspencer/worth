use crate::data::comparator::VersionComparatorPolicy;
use crate::data::graph::PreparedParallelApplyCommitPacket;
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::proof::ClassifiedSnapshotBatchCommit;
use crate::data::proof::SingleConsumer;
use crate::data::trace::RuntimeArtifactFinalizeImage;
use crate::logic::evaluation::EffectDependencyInputs;
use crate::logic::explain::RewiringSummary;
use crate::logic::planner::semantic::{StageSemanticBatch, StageSemanticIdentity};
use crate::logic::planner::ExecutionRecordId;
use crate::logic::prepared::PreparedEvaluation;

use super::serial_batch::AppliedSerialStageBatch;

mod capacity;
pub(in crate::logic::planner) use capacity::ApplyMemberBasis;

#[derive(Debug, Clone)]
pub(crate) struct ConcurrentWorkerInput {
    task_index: usize,
    node: NodeId,
    identity: StageSemanticIdentity,
    before_state: NodeState,
    before_artifact_state: Option<RuntimeArtifactFinalizeImage>,
    dependency_updates: u32,
    recomputed: bool,
    partition_aware: bool,
    rewiring: Option<RewiringSummary>,
    comparator_policy: VersionComparatorPolicy,
    prepared: PreparedEvaluation,
    dependency_inputs: EffectDependencyInputs,
}

#[derive(Debug, Clone)]
pub(crate) struct ConcurrentApplyGroupInput {
    group_index: usize,
    worker_inputs: Vec<ConcurrentWorkerInput>,
}

#[derive(Debug)]
pub(crate) struct GroupLocalTaskCommit {
    task_index: usize,
    node: NodeId,
    identity: StageSemanticIdentity,
    before_state: NodeState,
    before_artifact_state: Option<RuntimeArtifactFinalizeImage>,
    dependency_updates: u32,
    recomputed: bool,
    partition_aware: bool,
    rewiring: Option<RewiringSummary>,
    commit_packet: PreparedParallelApplyCommitPacket,
}

#[derive(Debug)]
pub(crate) struct GroupedApplyFailure {
    pub(in crate::logic::planner) node: NodeId,
    pub(in crate::logic::planner) record_id: ExecutionRecordId,
    pub(in crate::logic::planner) error: crate::data::error::SignalError,
    pub(in crate::logic::planner) reuse_failure: Option<crate::data::reuse::ReuseBoundaryFailure>,
}

#[derive(Debug)]
pub(crate) struct GroupLocalApplyPacket {
    group_index: usize,
    task_count: usize,
    task_commits: Vec<GroupLocalTaskCommit>,
}

impl GroupLocalApplyPacket {
    pub(in crate::logic::planner) fn new(
        group_index: usize,
        task_commits: Vec<GroupLocalTaskCommit>,
    ) -> Self {
        let task_count = task_commits.len();
        Self {
            group_index,
            task_count,
            task_commits,
        }
    }

    pub(in crate::logic::planner) fn packet_breadth(&self) -> usize {
        self.task_count
    }

    pub(in crate::logic::planner) fn group_index(&self) -> usize {
        self.group_index
    }

    pub(in crate::logic::planner) fn into_task_commits(self) -> Vec<GroupLocalTaskCommit> {
        self.task_commits
    }
}

/// Stage-lifetime workspace for lowered apply, snapshot deferral, and semantic finalize.
#[derive(Debug)]
pub(in crate::logic::planner) struct StageScratch {
    finalize_work: StageFinalizeWork,
    pending_snapshots: ClassifiedSnapshotBatchCommit,
}

#[derive(Debug)]
pub(in crate::logic::planner) enum StageFinalizeWork {
    Serial(AppliedSerialStageBatch),
    Parallel(SingleConsumer<StageSemanticBatch>),
}

impl ConcurrentWorkerInput {
    pub(in crate::logic::planner) fn new(
        task_index: usize,
        node: NodeId,
        identity: StageSemanticIdentity,
        before_state: NodeState,
        before_artifact_state: Option<RuntimeArtifactFinalizeImage>,
        dependency_updates: u32,
        recomputed: bool,
        partition_aware: bool,
        rewiring: Option<RewiringSummary>,
        comparator_policy: VersionComparatorPolicy,
        prepared: PreparedEvaluation,
        dependency_inputs: EffectDependencyInputs,
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
            comparator_policy,
            prepared,
            dependency_inputs,
        }
    }

    pub(in crate::logic::planner) fn into_parts(
        self,
    ) -> (
        usize,
        NodeId,
        StageSemanticIdentity,
        NodeState,
        Option<RuntimeArtifactFinalizeImage>,
        u32,
        bool,
        bool,
        Option<RewiringSummary>,
        VersionComparatorPolicy,
        PreparedEvaluation,
        EffectDependencyInputs,
    ) {
        (
            self.task_index,
            self.node,
            self.identity,
            self.before_state,
            self.before_artifact_state,
            self.dependency_updates,
            self.recomputed,
            self.partition_aware,
            self.rewiring,
            self.comparator_policy,
            self.prepared,
            self.dependency_inputs,
        )
    }
}

impl ConcurrentApplyGroupInput {
    pub(in crate::logic::planner) fn task_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.worker_inputs.iter().map(|input| input.task_index)
    }

    pub(in crate::logic::planner) fn new(
        group_index: usize,
        worker_inputs: Vec<ConcurrentWorkerInput>,
    ) -> Self {
        Self {
            group_index,
            worker_inputs,
        }
    }

    pub(in crate::logic::planner) fn into_parts(self) -> (usize, Vec<ConcurrentWorkerInput>) {
        (self.group_index, self.worker_inputs)
    }
}

impl GroupLocalTaskCommit {
    pub(in crate::logic::planner) fn task_index(&self) -> usize {
        self.task_index
    }

    pub(in crate::logic::planner) fn new(
        task_index: usize,
        node: NodeId,
        identity: StageSemanticIdentity,
        before_state: NodeState,
        before_artifact_state: Option<RuntimeArtifactFinalizeImage>,
        dependency_updates: u32,
        recomputed: bool,
        partition_aware: bool,
        rewiring: Option<RewiringSummary>,
        commit_packet: PreparedParallelApplyCommitPacket,
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
            commit_packet,
        }
    }

    pub(in crate::logic::planner) fn into_parts(
        self,
    ) -> (
        usize,
        NodeId,
        StageSemanticIdentity,
        NodeState,
        Option<RuntimeArtifactFinalizeImage>,
        u32,
        bool,
        bool,
        Option<RewiringSummary>,
        PreparedParallelApplyCommitPacket,
    ) {
        (
            self.task_index,
            self.node,
            self.identity,
            self.before_state,
            self.before_artifact_state,
            self.dependency_updates,
            self.recomputed,
            self.partition_aware,
            self.rewiring,
            self.commit_packet,
        )
    }
}

impl StageScratch {
    pub(in crate::logic::planner) fn new(
        finalize_work: StageFinalizeWork,
        pending_snapshots: ClassifiedSnapshotBatchCommit,
    ) -> Self {
        Self {
            finalize_work,
            pending_snapshots,
        }
    }

    pub(in crate::logic::planner) fn into_parts(
        self,
    ) -> (StageFinalizeWork, ClassifiedSnapshotBatchCommit) {
        (self.finalize_work, self.pending_snapshots)
    }
}

impl worth_execution::ChargedBytes for GroupLocalApplyPacket {
    fn additional_charged_bytes(&self) -> u64 {
        worth_execution::ChargedBytes::additional_charged_bytes(&self.task_commits)
    }
}

impl worth_execution::ChargedBytes for GroupLocalTaskCommit {
    fn additional_charged_bytes(&self) -> u64 {
        use crate::data::retained_storage::{
            RetainedStorageMeasurement, RetainedStoragePreparation,
        };
        let mut work = RetainedStoragePreparation::new(usize::MAX);
        let metadata = self
            .before_artifact_state
            .retained_heap_charge(&mut work)
            .and_then(|charge| charge.checked_add(self.rewiring.retained_heap_charge(&mut work)?))
            .map_or(u64::MAX, |charge| charge.bytes());
        metadata.saturating_add(worth_execution::ChargedBytes::additional_charged_bytes(
            &self.commit_packet,
        ))
    }
}

impl worth_execution::ChargedBytes for ConcurrentApplyGroupInput {
    fn additional_charged_bytes(&self) -> u64 {
        worth_execution::ChargedBytes::additional_charged_bytes(&self.worker_inputs)
    }
}

impl worth_execution::ChargedBytes for ConcurrentWorkerInput {
    fn additional_charged_bytes(&self) -> u64 {
        use crate::data::retained_storage::{
            RetainedStorageCharge as Charge, RetainedStorageMeasurement,
            RetainedStoragePreparation as Work,
        };
        let mut work = Work::new(usize::MAX);
        self.before_artifact_state
            .retained_heap_charge(&mut work)
            .and_then(|charge| charge.checked_add(self.rewiring.retained_heap_charge(&mut work)?))
            .and_then(|charge| {
                charge.checked_add(self.comparator_policy.retained_heap_charge(&mut work)?)
            })
            .and_then(|charge| charge.checked_add(self.prepared.retained_heap_charge(&mut work)?))
            .and_then(|charge| {
                charge.checked_add(
                    self.dependency_inputs
                        .dependency_snapshot_update
                        .retained_heap_charge(&mut work)?,
                )
            })
            .map_or(u64::MAX, Charge::bytes)
    }
}
