use super::super::super::{
    ProjectMaterializedFacts, ProjectionConsumptionSource, ProjectionFactExtractionError,
};
use super::support::{
    admitted, binding_for_result_shape, query_context_execution_current,
    query_context_execution_historical, query_context_execution_preview, text_value,
};
use crate::query_context::{QueryContextExecutionArtifact, QueryContextExecutionFamily};

#[test]
fn current_query_context_extracts_identity_and_row_bound_field_facts() {
    let execution = query_context_execution_current();
    let contract = admitted(
        ProjectionConsumptionSource::from_query_context_execution(&execution),
        binding_for_result_shape(
            "result-shape:test",
            &["profile.display_name", "metrics.priority"],
        ),
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .view_local_identities()
            .display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            )
            .derived_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("metrics")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("priority")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
    )
    .bind_contract()
    .expect("fixture digest must fit");

    let consumed = contract
        .extract_from_query_context_execution(&execution)
        .unwrap();

    assert_eq!(
        consumed.entity_identities()[0].entity_identity(),
        &crate::memory_workspace::admit_authored_entity_label(
            consumed.entity_identities()[0].source_row_identity(),
        )
    );
    assert_eq!(
        consumed.view_local_identities()[1].view_local_identity(),
        consumed.view_local_identities()[1].source_row_identity(),
    );
    assert_eq!(
        consumed.display_fields()[0]
            .native_value()
            .scalar()
            .unwrap(),
        &text_value("payload-row-0")
    );
    assert_eq!(
        consumed.derived_fields()[1]
            .native_value()
            .scalar()
            .unwrap(),
        &text_value("payload-row-1")
    );
    assert_eq!(consumed.counters().source_row_width_consumed(), 4);
    assert_eq!(consumed.counters().source_evidence_lookup_width(), 0);
    assert_eq!(consumed.counters().authority_reopen_count(), 0);
}

#[test]
fn query_context_extracts_bound_source_reference_metadata() {
    let execution = query_context_execution_historical();
    let contract = admitted(
        ProjectionConsumptionSource::from_query_context_execution(&execution),
        binding_for_result_shape("result-shape:test", &["profile.display_name"]),
        ProjectMaterializedFacts::declare()
            .display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            )
            .source_references(),
    )
    .bind_contract()
    .expect("fixture digest must fit");

    let consumed = contract
        .extract_from_query_context_execution(&execution)
        .unwrap();

    assert_eq!(consumed.source_references().len(), 1);
    assert_eq!(
        consumed.source_references()[0].label(),
        "query_context_materialization_path"
    );
    assert_eq!(
        consumed.source_references()[0].identity(),
        "materialization-path:test"
    );
    assert_eq!(
        consumed.display_fields()[0]
            .native_value()
            .scalar()
            .unwrap(),
        &text_value("historical-row-0")
    );
    assert_eq!(consumed.counters().source_row_width_consumed(), 1);
    assert_eq!(consumed.counters().source_evidence_lookup_width(), 1);
}

#[test]
fn query_context_extraction_rejects_result_shape_drift() {
    let execution = query_context_execution_current();
    let contract = admitted(
        ProjectionConsumptionSource::from_query_context_execution(&execution),
        binding_for_result_shape("result-shape:test", &["profile.display_name"]),
        ProjectMaterializedFacts::declare().display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("profile")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("display_name")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
    )
    .bind_contract()
    .expect("fixture digest must fit");
    let mismatched = QueryContextExecutionArtifact::test_only(
        QueryContextExecutionFamily::RuntimeCurrent,
        "query:test",
        "basis:test",
        "result:test",
        "result-shape:other",
        vec!["payload-row-0".to_string()],
        None,
        None,
    );

    let error = contract
        .extract_from_query_context_execution(&mismatched)
        .unwrap_err();
    assert!(matches!(
        error,
        ProjectionFactExtractionError::SourceArtifactMetadataMismatch {
            metadata_label: "result_shape_digest",
            ..
        }
    ));
}

#[test]
fn query_context_extraction_rejects_missing_bound_source_reference() {
    let execution = query_context_execution_preview();
    let contract = admitted(
        ProjectionConsumptionSource::from_query_context_execution(&execution),
        binding_for_result_shape("result-shape:test", &[]),
        ProjectMaterializedFacts::declare().source_references(),
    )
    .bind_contract()
    .expect("fixture digest must fit");
    let missing_reference = QueryContextExecutionArtifact::test_only(
        QueryContextExecutionFamily::PreviewDerivedHistorical,
        "query:test",
        "basis:test",
        "result:test",
        "result-shape:test",
        vec!["preview-row-0".to_string()],
        None,
        None,
    );

    let error = contract
        .extract_from_query_context_execution(&missing_reference)
        .unwrap_err();
    assert!(matches!(
        error,
        ProjectionFactExtractionError::SourceReferenceEvidenceMismatch {
            expected_count: 1,
            actual_count: 0,
        }
    ));
}
