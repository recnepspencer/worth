//! Storage projections retain source independently of an executable graph binding.
use super::NodeEntry;
use crate::data::aspect::AspectMask;
use crate::data::conditional_execution::*;
use crate::data::node::{EvaluationCondition, NodeEvaluationConfig};
use std::sync::Arc;

fn source() -> SignalConditionalContractDefinition {
    SignalConditionalContractDefinition {
        condition: SignalConditionalCondition::DeltaThreshold(SignalDeltaThresholdContract::new(
            worth_foundational::facade::AspectValue::String(
                worth_foundational::facade::InternedString::Raw("authored literal".into()),
            ),
            "authored units",
            SignalThresholdValueFamily::Integer,
            SignalThresholdComparisonDomain::AbsoluteDifference,
            SignalThresholdBoundary::Inclusive,
        )),
        dependency_aspects: AspectMask::ALL,
        trigger_aspects: AspectMask::ALL,
        dependency_comparator: SignalConditionalVersionComparator::Exact,
        output_comparator: SignalConditionalVersionComparator::Exact,
        artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
    }
}

#[test]
fn node_memory_and_checkpoint_projections_retain_the_full_conditional_source() {
    let definition = source();
    let mut entry = NodeEntry::new();
    entry
        .install_conditional_contract_occurrence(31, Arc::new(definition.clone()))
        .unwrap();
    let restored = NodeEntry::from_checkpoint_image(entry.to_checkpoint_image());
    let decoded: NodeEntry = serde_json::from_slice(&serde_json::to_vec(&entry).unwrap()).unwrap();
    for projected in [restored, decoded] {
        assert_eq!(
            projected.definition.conditional_definition.as_deref(),
            Some(&definition)
        );
    }
}

#[test]
fn unrelated_node_metadata_preserves_conditional_source_and_a_changed_condition_retires_it() {
    let definition = source();
    let mut entry = NodeEntry::new();
    entry
        .install_conditional_contract_occurrence(31, Arc::new(definition.clone()))
        .unwrap();
    let mut metadata = entry.get_eval_config().clone();
    metadata.partitioned_output = true;
    entry.set_eval_config(metadata);
    assert_eq!(
        entry.definition.conditional_definition.as_deref(),
        Some(&definition)
    );
    entry.set_eval_config(NodeEvaluationConfig {
        condition: EvaluationCondition::OnDemand,
        ..entry.get_eval_config().clone()
    });
    assert!(entry.definition.conditional_definition.is_none());
}
