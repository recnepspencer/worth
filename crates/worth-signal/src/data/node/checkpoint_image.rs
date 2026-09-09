use serde::{Deserialize, Serialize};

use crate::data::aspect::{Aspect, AspectMask, PartitionVersionMap};
use crate::data::dependency::DependencySnapshotId;
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::data::graph::{DependencySetId, SubscriberSetId};
use crate::data::output::PartitionSubscription;
use crate::data::proof::invalidation::binding::{
    DependencyRevision, PendingDependencyRevalidation,
};
use crate::data::trace::{
    CausalityMetadata, ExecutionTraceStamp, RetainedDiagnosticArtifact, RuntimeArtifactState,
};

use super::{NodeEvaluationConfig, NodeState};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CheckpointNodeImageParts {
    pub(crate) state: NodeState,
    pub(crate) dirty_aspects: AspectMask,
    pub(crate) dirty_partition_scopes: Vec<(Aspect, PartitionSubscription)>,
    pub(crate) aspect_versions: PartitionVersionMap,
    pub(crate) dependencies_id: DependencySetId,
    pub(crate) subscribers_id: SubscriberSetId,
    pub(crate) dep_snapshot_id: DependencySnapshotId,
    pub(crate) pending_cause_set_id: PendingCauseSetId,
    pub(crate) dependency_revision: DependencyRevision,
    pub(crate) pending_dependency_revalidation: Option<PendingDependencyRevalidation>,
    pub(crate) direct_invalidation_basis:
        Option<crate::data::proof::invalidation::source_seed::DirectInvalidationBasis>,
    pub(crate) direct_invalidation_generation: u64,
    pub(crate) tombstoned: bool,
    pub(crate) conditional_contract_generation: u64,
    pub(crate) conditional_contract_occurrence: u64,
    pub(crate) runtime_artifact_state: Option<RuntimeArtifactState>,
    pub(crate) retained_artifact: Option<RetainedDiagnosticArtifact>,
    pub(crate) causality: Option<CausalityMetadata>,
    pub(crate) execution_trace: Option<ExecutionTraceStamp>,
    pub(crate) eval_config: NodeEvaluationConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointNodeImage {
    state: NodeState,
    dirty_aspects: AspectMask,
    #[serde(default)]
    dirty_partition_scopes: Vec<(Aspect, PartitionSubscription)>,
    aspect_versions: PartitionVersionMap,
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
    direct_invalidation_basis:
        Option<crate::data::proof::invalidation::source_seed::DirectInvalidationBasis>,
    #[serde(default)]
    direct_invalidation_generation: u64,
    tombstoned: bool,
    #[serde(default)]
    conditional_contract_generation: u64,
    #[serde(default)]
    conditional_contract_occurrence: u64,
    #[serde(default)]
    runtime_artifact_state: Option<RuntimeArtifactState>,
    #[serde(default)]
    retained_artifact: Option<RetainedDiagnosticArtifact>,
    #[serde(default)]
    causality: Option<CausalityMetadata>,
    #[serde(default)]
    execution_trace: Option<ExecutionTraceStamp>,
    #[serde(default)]
    eval_config: NodeEvaluationConfig,
}

impl CheckpointNodeImage {
    pub(crate) fn from_parts(parts: CheckpointNodeImageParts) -> Self {
        Self {
            state: parts.state,
            dirty_aspects: parts.dirty_aspects,
            dirty_partition_scopes: parts.dirty_partition_scopes,
            aspect_versions: parts.aspect_versions,
            dependencies_id: parts.dependencies_id,
            subscribers_id: parts.subscribers_id,
            dep_snapshot_id: parts.dep_snapshot_id,
            pending_cause_set_id: parts.pending_cause_set_id,
            dependency_revision: parts.dependency_revision,
            pending_dependency_revalidation: parts.pending_dependency_revalidation,
            direct_invalidation_basis: parts.direct_invalidation_basis,
            direct_invalidation_generation: parts.direct_invalidation_generation,
            tombstoned: parts.tombstoned,
            conditional_contract_generation: parts.conditional_contract_generation,
            conditional_contract_occurrence: parts.conditional_contract_occurrence,
            runtime_artifact_state: parts.runtime_artifact_state,
            retained_artifact: parts.retained_artifact,
            causality: parts.causality,
            execution_trace: parts.execution_trace,
            eval_config: parts.eval_config,
        }
    }

    pub(crate) fn into_parts(self) -> CheckpointNodeImageParts {
        CheckpointNodeImageParts {
            state: self.state,
            dirty_aspects: self.dirty_aspects,
            dirty_partition_scopes: self.dirty_partition_scopes,
            aspect_versions: self.aspect_versions,
            dependencies_id: self.dependencies_id,
            subscribers_id: self.subscribers_id,
            dep_snapshot_id: self.dep_snapshot_id,
            pending_cause_set_id: self.pending_cause_set_id,
            dependency_revision: self.dependency_revision,
            pending_dependency_revalidation: self.pending_dependency_revalidation,
            direct_invalidation_basis: self.direct_invalidation_basis,
            direct_invalidation_generation: self.direct_invalidation_generation,
            tombstoned: self.tombstoned,
            conditional_contract_generation: self.conditional_contract_generation,
            conditional_contract_occurrence: self.conditional_contract_occurrence,
            runtime_artifact_state: self.runtime_artifact_state,
            retained_artifact: self.retained_artifact,
            causality: self.causality,
            execution_trace: self.execution_trace,
            eval_config: self.eval_config,
        }
    }

    pub(crate) fn runtime_artifact_state(&self) -> Option<&RuntimeArtifactState> {
        self.runtime_artifact_state.as_ref()
    }

    pub(crate) fn runtime_artifact_state_mut(&mut self) -> Option<&mut RuntimeArtifactState> {
        self.runtime_artifact_state.as_mut()
    }

    pub(crate) fn retained_artifact(&self) -> Option<&RetainedDiagnosticArtifact> {
        self.retained_artifact.as_ref()
    }

    pub(crate) fn causality(&self) -> Option<&CausalityMetadata> {
        self.causality.as_ref()
    }

    pub(crate) fn set_dependencies_id(&mut self, dependencies_id: DependencySetId) {
        self.dependencies_id = dependencies_id;
    }

    pub(crate) fn set_subscribers_id(&mut self, subscribers_id: SubscriberSetId) {
        self.subscribers_id = subscribers_id;
    }

    pub(crate) fn set_dep_snapshot_id(&mut self, dep_snapshot_id: DependencySnapshotId) {
        self.dep_snapshot_id = dep_snapshot_id;
    }

    pub(crate) fn clear_dependency_handles_for_adoption(&mut self) {
        self.dependencies_id = DependencySetId::EMPTY;
        self.subscribers_id = SubscriberSetId::EMPTY;
        self.dep_snapshot_id = DependencySnapshotId::EMPTY;
    }

    pub(crate) fn set_eval_config(&mut self, eval_config: NodeEvaluationConfig) {
        self.eval_config = eval_config;
    }

    pub(crate) fn set_runtime_artifact_state(
        &mut self,
        runtime_artifact_state: Option<RuntimeArtifactState>,
    ) {
        self.runtime_artifact_state = runtime_artifact_state;
    }

    pub(crate) fn set_retained_artifact(
        &mut self,
        retained_artifact: Option<RetainedDiagnosticArtifact>,
    ) {
        self.retained_artifact = retained_artifact;
    }

    pub(crate) fn set_causality(&mut self, causality: Option<CausalityMetadata>) {
        self.causality = causality;
    }
}
