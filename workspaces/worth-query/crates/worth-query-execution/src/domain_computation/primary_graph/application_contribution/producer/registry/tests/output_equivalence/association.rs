//! Pure association contrasts use admitted metadata, never pretend Native evaluation evidence.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::{
    output_equivalence_contract::InstalledProducerOutputEquivalence,
    output_evaluation::dependency_matches_output,
};
use std::sync::Arc;
use worth_foundational::facade::{
    AspectBinding, AspectMask, AuthoritativeAspectChangeKind, FieldKey, ProjectionMask,
};
use worth_runtime_bridge::facade::{
    BridgeSemanticDependencyCandidate, BridgeSemanticDependencyCandidateParts,
    BridgeSemanticLocality, RelationalBridgeRecordIdentityParts as Record,
};

fn dependency(
    field: &InstalledProducerOutputEquivalence,
    record: Option<Record>,
    binding: AspectBinding,
) -> BridgeSemanticDependencyCandidate {
    BridgeSemanticDependencyCandidate::admit(BridgeSemanticDependencyCandidateParts {
        source_installation_identity: Arc::from("association-only-installation"),
        source_basis: Arc::from("association-only-basis"),
        source_runtime_authority: 1,
        source_installation_generation: 1,
        source_authority_binding_identity: Arc::from("association-only-binding"),
        source_stage_identity: None,
        source_node_identity: Arc::from("association-only-node"),
        dependency_ordinal: 0,
        declared_graph_role: Arc::from("model"),
        graph_participation_identity: Arc::from("association-only-participation"),
        graph_adapter_identity: Arc::from("association-only-adapter"),
        source_record_identity: record,
        observation_record_identity: record,
        contract: field.field.aspect.clone(),
        projection_mask: AspectMask::<ProjectionMask>::whole_aspect(),
        binding,
        locality: if record.is_some() {
            BridgeSemanticLocality::SourceRecord
        } else {
            BridgeSemanticLocality::ManagedSourceRecord
        },
        relevant_changes: vec![AuthoritativeAspectChangeKind::FieldSet],
    })
    .unwrap()
}
#[test]
fn readiness_dependency_requires_actual_output_generation_and_physical_binding() {
    let declaration = declaration::<Struct>();
    let field = declaration.installed_output_equivalence.unwrap();
    let output = Record::entity(0, 7, 3);
    for record in [None, Some(output)] {
        assert!(dependency_matches_output(
            &dependency(&field, record, field.field.native_binding.clone()),
            &field,
            output
        ));
    }
    for wrong in [
        Record::entity(0, 8, 3),
        Record::entity(0, 7, 4),
        Record::entity(1, 7, 3),
    ] {
        assert!(
            !dependency_matches_output(
                &dependency(&field, Some(wrong), field.field.native_binding.clone()),
                &field,
                output
            ),
            "same contract cannot substitute a sibling, replacement generation, or partition"
        );
    }
    let wrong_binding = AspectBinding::EntityField {
        field: FieldKey::new("another-physical-field").unwrap(),
    };
    assert!(!dependency_matches_output(
        &dependency(&field, Some(output), wrong_binding),
        &field,
        output
    ));
}
