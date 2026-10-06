//! Preserve the flat warm-row wire while decoding full-width nested artifacts directly.
use std::sync::Arc;

use serde::{Deserialize, Deserializer};
use smallvec::SmallVec;

use super::{NodeDefinitionData, NodeWarmData};
use crate::data::aspect::{Aspect, PartitionVersionOverrides};
use crate::data::core_profile::HOT_VEC_INLINE_CAPACITY;
use crate::data::node::NodeEvaluationConfig;
use crate::data::output::PartitionSubscription;
use crate::data::proof::invalidation::binding::PendingDependencyRevalidation;
use crate::data::proof::invalidation::source_seed::DirectInvalidationBasis;
use crate::data::trace::RuntimeArtifactState;

pub(super) struct WarmWire {
    pub(super) definition: NodeDefinitionData,
    pub(super) evaluation: NodeWarmData,
}

#[derive(Deserialize)]
struct WarmFields {
    #[serde(default)]
    tombstoned: bool,
    #[serde(default)]
    conditional_contract_generation: u64,
    #[serde(default)]
    conditional_contract_occurrence: u64,
    #[serde(default)]
    eval_config: NodeEvaluationConfig,
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
}

impl<'de> Deserialize<'de> for WarmWire {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = WarmFields::deserialize(deserializer)?;
        Ok(Self {
            definition: NodeDefinitionData {
                tombstoned: fields.tombstoned,
                conditional_contract_generation: fields.conditional_contract_generation,
                conditional_contract_occurrence: fields.conditional_contract_occurrence,
                eval_config: fields.eval_config,
            },
            evaluation: NodeWarmData {
                pending_dependency_revalidation: fields.pending_dependency_revalidation,
                direct_invalidation_basis: fields.direct_invalidation_basis,
                direct_invalidation_generation: fields.direct_invalidation_generation,
                aspect_version_overrides: fields.aspect_version_overrides,
                dirty_partition_scope_payload: fields.dirty_partition_scope_payload,
                runtime_artifact_state: fields.runtime_artifact_state,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::WarmWireRef;
    use super::*;
    use crate::data::core_profile::StableHashValue;
    use crate::data::trace::RuntimeArtifactHot;

    #[test]
    fn flat_warm_row_keeps_full_width_artifact_identity() {
        let definition = NodeDefinitionData {
            conditional_contract_generation: 7,
            ..Default::default()
        };
        let evaluation = NodeWarmData {
            runtime_artifact_state: Some(Arc::new(RuntimeArtifactState::new(
                RuntimeArtifactHot {
                    output_hash: StableHashValue::MAX,
                    ..Default::default()
                },
                Default::default(),
            ))),
            ..Default::default()
        };
        let wire = serde_json::to_string(&WarmWireRef {
            definition: &definition,
            evaluation: &evaluation,
        })
        .unwrap();
        let restored: WarmWire = serde_json::from_str(&wire).unwrap();
        assert_eq!(restored.definition, definition);
        assert_eq!(restored.evaluation, evaluation);
    }
}
