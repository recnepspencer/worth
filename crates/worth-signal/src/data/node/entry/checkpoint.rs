use crate::data::aspect::PartitionVersionMap;
use crate::data::node::CheckpointNodeImage;

use super::layout::{NodeColdData, NodeHotData, NodeWarmData};
use super::{NodeDefinitionData, NodeEntry};

impl NodeEntry {
    pub(crate) fn to_checkpoint_image(&self) -> CheckpointNodeImage {
        CheckpointNodeImage::from_parts(super::super::checkpoint_image::CheckpointNodeImageParts {
            state: self.hot.state,
            dirty_aspects: self.hot.dirty_aspects,
            dirty_partition_scopes: self
                .warm
                .dirty_partition_scope_payload
                .iter()
                .cloned()
                .collect(),
            aspect_versions: PartitionVersionMap::from_storage_parts(
                self.hot.aspect_version_header,
                self.warm.aspect_version_overrides.clone(),
            ),
            dependencies_id: self.hot.dependencies_id,
            subscribers_id: self.hot.subscribers_id,
            dep_snapshot_id: self.hot.dep_snapshot_id,
            pending_cause_set_id: self.hot.pending_cause_set_id,
            dependency_revision: self.hot.dependency_revision,
            pending_dependency_revalidation: self.warm.pending_dependency_revalidation.clone(),
            direct_invalidation_basis: self.warm.direct_invalidation_basis.clone(),
            direct_invalidation_generation: self.warm.direct_invalidation_generation,
            tombstoned: self.definition.tombstoned,
            conditional_contract_generation: self.definition.conditional_contract_generation,
            conditional_contract_occurrence: self.definition.conditional_contract_occurrence,
            runtime_artifact_state: self.warm.runtime_artifact_state.clone(),
            retained_artifact: self.cold_artifact_record().cloned(),
            causality: self.get_causality().cloned(),
            execution_trace: self.execution_trace_stamp(),
            eval_config: self.definition.eval_config.clone(),
        })
    }

    pub(crate) fn from_checkpoint_image(image: CheckpointNodeImage) -> Self {
        let mut image = image.into_parts();
        image.eval_config = image.eval_config.upgrade_legacy_output_equivalence();
        let (aspect_version_header, aspect_version_overrides) =
            image.aspect_versions.into_storage_parts();
        let mut entry = Self {
            definition: NodeDefinitionData {
                tombstoned: image.tombstoned,
                conditional_contract_generation: image.conditional_contract_generation,
                conditional_contract_occurrence: image.conditional_contract_occurrence,
                eval_config: image.eval_config,
            },
            hot: NodeHotData {
                state: image.state,
                dirty_aspects: image.dirty_aspects,
                dirty_partition_scope_aspects: crate::data::aspect::AspectMask::EMPTY,
                aspect_version_header,
                dependencies_id: image.dependencies_id,
                subscribers_id: image.subscribers_id,
                dep_snapshot_id: image.dep_snapshot_id,
                pending_cause_set_id: image.pending_cause_set_id,
                dependency_revision: image.dependency_revision,
            },
            warm: NodeWarmData {
                pending_dependency_revalidation: image.pending_dependency_revalidation,
                direct_invalidation_basis: image.direct_invalidation_basis,
                direct_invalidation_generation: image.direct_invalidation_generation,
                aspect_version_overrides,
                dirty_partition_scope_payload: image.dirty_partition_scopes.into_iter().collect(),
                runtime_artifact_state: image.runtime_artifact_state,
            },
            cold: None,
        };
        entry.sync_all_dirty_partition_scope_flags();
        entry.set_retained_diagnostic_artifact(image.retained_artifact);
        entry.set_causality(image.causality);
        entry.set_execution_trace_stamp(image.execution_trace);
        entry
    }

    pub(crate) fn from_storage_parts(
        definition: NodeDefinitionData,
        hot: NodeHotData,
        warm: NodeWarmData,
        cold: Option<Box<NodeColdData>>,
    ) -> Self {
        Self {
            definition,
            hot,
            warm,
            cold,
        }
    }

    pub(crate) fn into_storage_parts(
        self,
    ) -> (
        NodeDefinitionData,
        NodeHotData,
        NodeWarmData,
        Option<Box<NodeColdData>>,
    ) {
        (self.definition, self.hot, self.warm, self.cold)
    }
}
