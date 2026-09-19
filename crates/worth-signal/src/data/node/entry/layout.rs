mod clone_work;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

mod retained_charge;

use crate::data::aspect::{AspectMask, AspectVersionHeader, PartitionVersionOverrides};
use crate::data::core_profile::HOT_VEC_INLINE_CAPACITY;
use crate::data::dependency::DependencySnapshotId;
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::data::graph::{DependencySetId, SubscriberSetId};
use crate::data::proof::invalidation::binding::DependencyRevision;
use crate::data::trace::{
    CausalityMetadata, ExecutionTraceStamp, RetainedDiagnosticArtifact, RuntimeArtifactState,
};

use super::NodeState;

/// Cold node fields are boxed so diagnostic richness does not enter the node's
/// inline operational storage footprint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct NodeColdData {
    #[serde(default)]
    pub(crate) retained_artifact: Option<RetainedDiagnosticArtifact>,
    #[serde(default)]
    pub(crate) causality: Option<CausalityMetadata>,
    #[serde(default)]
    pub(crate) execution_trace: Option<ExecutionTraceStamp>,
}

/// Hot node fields are the fixed operational state consulted by invalidation,
/// evaluation, and graph routing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct NodeHotData {
    pub(crate) state: NodeState,
    pub(crate) dirty_aspects: AspectMask,
    #[serde(default)]
    pub(crate) dirty_partition_scope_aspects: AspectMask,
    pub(crate) aspect_version_header: AspectVersionHeader,
    pub(crate) dependencies_id: DependencySetId,
    pub(crate) subscribers_id: SubscriberSetId,
    pub(crate) dep_snapshot_id: DependencySnapshotId,
    #[serde(default)]
    pub(crate) pending_cause_set_id: PendingCauseSetId,
    #[serde(default)]
    pub(crate) dependency_revision: DependencyRevision,
}

/// Warm fields preserve node-local state that is bounded but not required by
/// the tightest graph traversal loops.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct NodeWarmData {
    #[serde(default)]
    pub(crate) pending_dependency_revalidation:
        Option<crate::data::proof::invalidation::binding::PendingDependencyRevalidation>,
    #[serde(default)]
    pub(crate) direct_invalidation_basis:
        Option<crate::data::proof::invalidation::source_seed::DirectInvalidationBasis>,
    #[serde(default)]
    pub(crate) direct_invalidation_generation: u64,
    #[serde(default)]
    pub(crate) aspect_version_overrides: PartitionVersionOverrides,
    #[serde(default)]
    pub(crate) dirty_partition_scope_payload: SmallVec<
        [(
            crate::data::aspect::Aspect,
            crate::data::output::PartitionSubscription,
        ); HOT_VEC_INLINE_CAPACITY],
    >,
    #[serde(default)]
    pub(crate) runtime_artifact_state: Option<RuntimeArtifactState>,
}

pub(crate) fn node_hot_inline_size_bytes() -> u64 {
    std::mem::size_of::<NodeHotData>() as u64
}

pub(crate) fn node_warm_inline_size_bytes() -> u64 {
    std::mem::size_of::<NodeWarmData>() as u64
}
