use serde::{Deserialize, Serialize};

use crate::data::aspect::AspectMask;
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::dependency::CanonicalDependencies;
use crate::data::handle::NodeId;
use crate::data::node::{AuthorityPolicy, NodeState, PathClass};
use crate::data::performance::{ResolvedExecutionStrategy, ResolvedMaintenanceStrategy};
use crate::data::proof::{
    DedupedNodeBatch, LoweredForm, PartitionScopeSet, SortedSourceBatch, StructuralDelta,
};
use crate::data::trace::RuntimeArtifactFinalizeImage;
use crate::logic::explain::RewiringSummary;
use crate::logic::prepared::PreparedEvaluation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyFootprint {
    pub partitions: PartitionScopeSet,
    pub touched_nodes: DedupedNodeBatch,
    pub touched_sources: SortedSourceBatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisjointApplyGroup {
    pub task_indices: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReductionOrderingContract {
    StageTaskIndexOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReductionWorkClass {
    DeterministicPublicationOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcurrentApplyReductionPlan {
    pub ordering_contract: ReductionOrderingContract,
    pub allowed_work: ReductionWorkClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApplyPlanSerialFallbackReason {
    BelowFullParallelThreshold,
    FullParallelUnsupportedByMutableEngine,
}

impl ApplyPlanSerialFallbackReason {
    pub fn code(self) -> &'static str {
        match self {
            Self::BelowFullParallelThreshold => "below-full-parallel-threshold",
            Self::FullParallelUnsupportedByMutableEngine => {
                "full-parallel-unsupported-by-mutable-engine"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerialApplyPlan {
    pub groups: Vec<DisjointApplyGroup>,
    pub rejection_reason: Option<ApplyPlanSerialFallbackReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcurrentApplyPlan {
    pub groups: Vec<DisjointApplyGroup>,
    pub reduction: ConcurrentApplyReductionPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoweredApplyPlan {
    Serial(SerialApplyPlan),
    GroupedConcurrent(ConcurrentApplyPlan),
}

#[derive(Debug, Clone)]
pub(crate) struct LoweredTaskExecution {
    prepared: PreparedEvaluation,
    before_state: NodeState,
    before_artifact_state: Option<RuntimeArtifactFinalizeImage>,
    dependency_updates: u32,
    recomputed: bool,
    partition_aware: bool,
    rewiring: Option<RewiringSummary>,
}

impl LoweredTaskExecution {
    pub(crate) fn new(
        prepared: PreparedEvaluation,
        before_state: NodeState,
        before_artifact_state: Option<RuntimeArtifactFinalizeImage>,
        dependency_updates: u32,
        recomputed: bool,
        partition_aware: bool,
        rewiring: Option<RewiringSummary>,
    ) -> Self {
        Self {
            prepared,
            before_state,
            before_artifact_state,
            dependency_updates,
            recomputed,
            partition_aware,
            rewiring,
        }
    }

    pub(crate) fn prepared(&self) -> &PreparedEvaluation {
        &self.prepared
    }

    pub(crate) fn dependency_updates(&self) -> u32 {
        self.dependency_updates
    }

    pub(crate) fn recomputed(&self) -> bool {
        self.recomputed
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        PreparedEvaluation,
        NodeState,
        Option<RuntimeArtifactFinalizeImage>,
        u32,
        bool,
        bool,
        Option<RewiringSummary>,
    ) {
        (
            self.prepared,
            self.before_state,
            self.before_artifact_state,
            self.dependency_updates,
            self.recomputed,
            self.partition_aware,
            self.rewiring,
        )
    }
}

#[derive(Debug, Clone)]
pub struct LoweredTask {
    task_index: usize,
    node: NodeId,
    produced_aspects: AspectMask,
    dependency_inputs: CanonicalDependencies,
    comparator_policy: VersionComparatorPolicy,
    path_class: PathClass,
    authority_policy: AuthorityPolicy,
    footprint: ApplyFootprint,
    execution: LoweredTaskExecution,
}

#[derive(Debug, Clone)]
pub struct LoweredStagePlan {
    stage_index: u32,
    tasks: Vec<LoweredTask>,
    lowered_apply_plan: LoweredApplyPlan,
    dirty_delta: StructuralDelta,
    execution_strategy: ResolvedExecutionStrategy,
    maintenance_strategy: ResolvedMaintenanceStrategy,
    authority_policy: AuthorityPolicy,
}

impl LoweredStagePlan {
    pub(crate) fn new(
        stage_index: u32,
        tasks: Vec<LoweredTask>,
        lowered_apply_plan: LoweredApplyPlan,
        dirty_delta: StructuralDelta,
        execution_strategy: ResolvedExecutionStrategy,
        maintenance_strategy: ResolvedMaintenanceStrategy,
        authority_policy: AuthorityPolicy,
    ) -> Self {
        Self {
            stage_index,
            tasks,
            lowered_apply_plan,
            dirty_delta,
            execution_strategy,
            maintenance_strategy,
            authority_policy,
        }
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub(crate) fn tasks(&self) -> &[LoweredTask] {
        self.tasks.as_slice()
    }

    pub(crate) fn dirty_delta(&self) -> &StructuralDelta {
        &self.dirty_delta
    }

    pub(crate) fn execution_strategy(&self) -> ResolvedExecutionStrategy {
        self.execution_strategy
    }

    pub(crate) fn maintenance_strategy(&self) -> ResolvedMaintenanceStrategy {
        self.maintenance_strategy
    }

    pub(crate) fn authority_policy(&self) -> AuthorityPolicy {
        self.authority_policy
    }

    pub fn apply_groups(&self) -> &[DisjointApplyGroup] {
        match &self.lowered_apply_plan {
            LoweredApplyPlan::Serial(plan) => plan.groups.as_slice(),
            LoweredApplyPlan::GroupedConcurrent(plan) => plan.groups.as_slice(),
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        u32,
        Vec<LoweredTask>,
        LoweredApplyPlan,
        StructuralDelta,
        ResolvedExecutionStrategy,
        ResolvedMaintenanceStrategy,
        AuthorityPolicy,
    ) {
        (
            self.stage_index,
            self.tasks,
            self.lowered_apply_plan,
            self.dirty_delta,
            self.execution_strategy,
            self.maintenance_strategy,
            self.authority_policy,
        )
    }
}

impl LoweredTask {
    pub(crate) fn new(
        task_index: usize,
        node: NodeId,
        produced_aspects: AspectMask,
        dependency_inputs: CanonicalDependencies,
        comparator_policy: VersionComparatorPolicy,
        path_class: PathClass,
        authority_policy: AuthorityPolicy,
        footprint: ApplyFootprint,
        execution: LoweredTaskExecution,
    ) -> Self {
        Self {
            task_index,
            node,
            produced_aspects,
            dependency_inputs,
            comparator_policy,
            path_class,
            authority_policy,
            footprint,
            execution,
        }
    }

    pub(crate) fn node(&self) -> NodeId {
        self.node
    }

    pub(crate) fn produced_aspects(&self) -> AspectMask {
        self.produced_aspects
    }

    pub(crate) fn dependency_inputs(&self) -> &CanonicalDependencies {
        &self.dependency_inputs
    }

    pub(crate) fn path_class(&self) -> PathClass {
        self.path_class
    }

    pub(crate) fn authority_policy(&self) -> AuthorityPolicy {
        self.authority_policy
    }

    pub(crate) fn footprint(&self) -> &ApplyFootprint {
        &self.footprint
    }

    pub(crate) fn execution(&self) -> &LoweredTaskExecution {
        &self.execution
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        usize,
        NodeId,
        AspectMask,
        CanonicalDependencies,
        VersionComparatorPolicy,
        PathClass,
        AuthorityPolicy,
        ApplyFootprint,
        LoweredTaskExecution,
    ) {
        (
            self.task_index,
            self.node,
            self.produced_aspects,
            self.dependency_inputs,
            self.comparator_policy,
            self.path_class,
            self.authority_policy,
            self.footprint,
            self.execution,
        )
    }
}

impl LoweredForm for ApplyFootprint {}
impl LoweredForm for DisjointApplyGroup {}
impl LoweredForm for LoweredTask {}
impl LoweredForm for LoweredStagePlan {}
