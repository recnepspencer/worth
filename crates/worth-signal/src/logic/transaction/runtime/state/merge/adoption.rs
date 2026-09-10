mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::dependency::{DependencyEdge, DependencySnapshot};
use crate::data::handle::NodeId;
use crate::data::node::NodeEvaluationConfig;
use crate::data::trace::ArtifactMergeAuthority;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetNodeIdentityIntent {
    ExistingMapping { mapped_target_node: NodeId },
    AllocateTargetNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdoptedNodeContract {
    pub eval_config: NodeEvaluationConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdoptionDependencyTopology {
    pub dependencies: Vec<DependencyEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdoptionDependencySnapshotRef {
    pub snapshot: DependencySnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RuntimeArtifactCarryPolicy {
    CarryMergeAdoptable,
    RebuildAfterAdoption,
    DoNotCarry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RetainedArtifactCarryPolicy {
    CarryIfPolicyAllows,
    ReconstructIfNeeded,
    Drop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CausalityCarryPolicy {
    CarryIfPolicyAllows,
    Drop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceNodeAdoptionPlanCore {
    pub source_node: NodeId,
    pub target_identity: TargetNodeIdentityIntent,
    pub authority: ArtifactMergeAuthority,
    pub entry_contract: AdoptedNodeContract,
    pub dependency_topology: AdoptionDependencyTopology,
    pub dependency_snapshot_ref: AdoptionDependencySnapshotRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceNodeAdoptionCarryPolicy {
    pub runtime_artifact: RuntimeArtifactCarryPolicy,
    pub retained_artifact: RetainedArtifactCarryPolicy,
    pub causality: CausalityCarryPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdoptedNodeMaterialization {
    pub target_node: NodeId,
    pub dependency_count: usize,
}
