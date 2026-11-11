use super::super::super::phase_four::support::{
    authorized_projection, query_context_execution_preview, read_result, read_result_shape,
    write_receipt, write_receipt_without_source_references,
};
use crate::projection_consumption::{
    DeferredProjectionConsumptionReason, ProjectMaterializedFacts,
    ProjectionConsumptionDenialReason, ProjectionConsumptionSource,
    ProjectionFactConsumptionAttempt, ProjectionSourceFamily,
};

#[test]
fn attempt_inspection_helpers_preserve_typed_non_admitted_postures() {
    let result = read_result();
    let result_shape = read_result_shape();
    let read_authorized_projection = authorized_projection(
        "query:test",
        result_shape.digest().as_str(),
        &["metrics.priority"],
    );
    let denied = result
        .consume_projection_facts(
            &result_shape,
            &read_authorized_projection,
            ProjectMaterializedFacts::declare().display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
        )
        .unwrap();
    assert!(denied.completed().is_none());
    assert!(denied.warnings().is_none());
    assert!(denied.deferred().is_none());
    assert!(denied.source_mismatch().is_none());
    assert_eq!(
        denied.denied().expect("typed denial").reason(),
        &ProjectionConsumptionDenialReason::FactFamilyNotVisible {
            field_key: "profile.display_name".to_string(),
        }
    );

    let write_authorized_projection =
        authorized_projection("query:test", "result-shape:test", &["identity.id"]);
    let deferred = write_receipt_without_source_references()
        .consume_projection_facts(
            "result-shape:test",
            &write_authorized_projection,
            ProjectMaterializedFacts::declare().source_references(),
        )
        .unwrap();
    assert!(deferred.completed().is_none());
    assert_eq!(
        deferred.deferred().expect("typed deferred").reason(),
        &DeferredProjectionConsumptionReason::WriteReceiptContractBindingPending
    );

    let mismatch = write_receipt()
        .consume_projection_facts(
            "result-shape:test",
            &write_authorized_projection,
            ProjectMaterializedFacts::declare().display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
        )
        .unwrap();
    assert!(mismatch.completed().is_none());
    assert_eq!(
        mismatch
            .source_mismatch()
            .expect("typed source mismatch")
            .source_family(),
        ProjectionSourceFamily::QueryWriteReceipt
    );
}

#[test]
fn common_path_read_result_returns_typed_denial_without_constructing_later_phase_artifacts() {
    let result = read_result();
    let result_shape = read_result_shape();
    let authorized_projection = authorized_projection(
        "query:test",
        result_shape.digest().as_str(),
        &["metrics.priority"],
    );

    let attempt = result
        .consume_projection_facts(
            &result_shape,
            &authorized_projection,
            ProjectMaterializedFacts::declare().display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
        )
        .unwrap();

    match attempt {
        ProjectionFactConsumptionAttempt::Denied(denied) => {
            assert_eq!(
                denied.reason(),
                &ProjectionConsumptionDenialReason::FactFamilyNotVisible {
                    field_key: "profile.display_name".to_string(),
                }
            );
        }
        other => panic!("expected typed denial on the common path, got {other:?}"),
    }
}

#[test]
fn common_path_write_receipt_returns_typed_deferred_without_constructing_later_phase_artifacts() {
    let receipt = write_receipt();
    let authorized_projection =
        authorized_projection("query:test", "result-shape:test", &["identity.id"]);

    let attempt = receipt
        .consume_projection_facts(
            "result-shape:test",
            &authorized_projection,
            ProjectMaterializedFacts::declare().display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
        )
        .unwrap();

    match attempt {
        ProjectionFactConsumptionAttempt::SourceMismatch(mismatch) => {
            assert_eq!(
                mismatch.source_family(),
                ProjectionSourceFamily::QueryWriteReceipt
            );
        }
        other => panic!("expected typed source mismatch on the common path, got {other:?}"),
    }
}

#[test]
fn common_path_write_receipt_returns_typed_deferred_when_requested_evidence_is_not_carried() {
    let receipt = write_receipt_without_source_references();
    let authorized_projection =
        authorized_projection("query:test", "result-shape:test", &["identity.id"]);

    let attempt = receipt
        .consume_projection_facts(
            "result-shape:test",
            &authorized_projection,
            ProjectMaterializedFacts::declare().source_references(),
        )
        .unwrap();

    match attempt {
        ProjectionFactConsumptionAttempt::Deferred(deferred) => {
            assert_eq!(
                deferred.reason(),
                &DeferredProjectionConsumptionReason::WriteReceiptContractBindingPending
            );
        }
        other => panic!("expected typed deferred posture on the common path, got {other:?}"),
    }
}

#[test]
fn source_surfaces_expose_support_discovery_helpers() {
    let read_shape = read_result_shape();
    let read_support = read_result()
        .receipt()
        .discover_projection_fact_consumption_support(&read_shape)
        .expect("fixture digest must fit");
    assert_eq!(
        read_support.source_family(),
        ProjectionConsumptionSource::from_read_receipt(read_result().receipt(), &read_shape)
            .family()
    );

    let write_support = write_receipt()
        .discover_projection_fact_consumption_support()
        .expect("fixture digest must fit");
    assert_eq!(
        write_support.source_family(),
        ProjectionConsumptionSource::from_write_receipt(&write_receipt()).family()
    );

    let query_context_support = query_context_execution_preview()
        .discover_projection_fact_consumption_support()
        .expect("fixture digest must fit");
    assert_eq!(
        query_context_support.source_family(),
        ProjectionConsumptionSource::from_query_context_execution(
            &query_context_execution_preview()
        )
        .family()
    );
}
