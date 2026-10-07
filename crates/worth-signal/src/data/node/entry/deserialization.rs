//! Read the historical flat node fields directly, preserving nested full-width hashes.
use std::sync::Arc;

use serde::{Deserialize, Deserializer};
use smallvec::SmallVec;

use super::{NodeColdData, NodeDefinitionData, NodeEntry, NodeHotData, NodeState, NodeWarmData};
use crate::data::aspect::{Aspect, AspectMask, AspectVersionHeader, PartitionVersionOverrides};
use crate::data::core_profile::HOT_VEC_INLINE_CAPACITY;
use crate::data::dependency::DependencySnapshotId;
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::data::graph::{DependencySetId, SubscriberSetId};
use crate::data::node::NodeEvaluationConfig;
use crate::data::output::PartitionSubscription;
use crate::data::proof::invalidation::binding::{
    DependencyRevision, PendingDependencyRevalidation,
};
use crate::data::proof::invalidation::source_seed::DirectInvalidationBasis;
use crate::data::trace::RuntimeArtifactState;

#[derive(Deserialize)]
struct FlatNodeFields {
    #[serde(default)]
    tombstoned: bool,
    #[serde(default)]
    conditional_contract_generation: u64,
    #[serde(default)]
    conditional_contract_occurrence: u64,
    #[serde(default)]
    eval_config: NodeEvaluationConfig,
    state: NodeState,
    dirty_aspects: AspectMask,
    #[serde(default)]
    dirty_partition_scope_aspects: AspectMask,
    aspect_version_header: AspectVersionHeader,
    dependencies_id: DependencySetId,
    subscribers_id: SubscriberSetId,
    dep_snapshot_id: DependencySnapshotId,
    #[serde(default)]
    pending_cause_set_id: PendingCauseSetId,
    #[serde(default)]
    dependency_revision: DependencyRevision,
    #[serde(default)]
    pending_dependency_revalidation: Option<PendingDependencyRevalidation>,
    #[serde(default)]
    direct_invalidation_basis: Option<Arc<DirectInvalidationBasis>>,
    #[serde(default)]
    direct_invalidation_generation: u64,
    #[serde(default)]
    aspect_version_overrides: Arc<PartitionVersionOverrides>,
    #[serde(default)]
    dirty_partition_scope_payload:
        SmallVec<[(Aspect, PartitionSubscription); HOT_VEC_INLINE_CAPACITY]>,
    #[serde(default)]
    runtime_artifact_state: Option<Arc<RuntimeArtifactState>>,
    #[serde(default)]
    cold: Option<Box<NodeColdData>>,
}

impl<'de> Deserialize<'de> for NodeEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = FlatNodeFields::deserialize(deserializer)?;
        Ok(Self {
            definition: NodeDefinitionData {
                tombstoned: fields.tombstoned,
                conditional_contract_generation: fields.conditional_contract_generation,
                conditional_contract_occurrence: fields.conditional_contract_occurrence,
                eval_config: fields.eval_config,
            },
            hot: NodeHotData {
                state: fields.state,
                dirty_aspects: fields.dirty_aspects,
                dirty_partition_scope_aspects: fields.dirty_partition_scope_aspects,
                aspect_version_header: fields.aspect_version_header,
                dependencies_id: fields.dependencies_id,
                subscribers_id: fields.subscribers_id,
                dep_snapshot_id: fields.dep_snapshot_id,
                pending_cause_set_id: fields.pending_cause_set_id,
                dependency_revision: fields.dependency_revision,
            },
            warm: NodeWarmData {
                pending_dependency_revalidation: fields.pending_dependency_revalidation,
                direct_invalidation_basis: fields.direct_invalidation_basis,
                direct_invalidation_generation: fields.direct_invalidation_generation,
                aspect_version_overrides: fields.aspect_version_overrides,
                dirty_partition_scope_payload: fields.dirty_partition_scope_payload,
                runtime_artifact_state: fields.runtime_artifact_state,
            },
            cold: fields.cold,
        })
    }
}
