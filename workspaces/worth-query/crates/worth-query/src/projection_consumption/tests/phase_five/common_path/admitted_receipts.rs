use super::super::super::phase_four::support::{
    authorized_projection, query_context_execution_preview, read_result, read_result_shape,
    write_receipt,
};
use super::assert_common_path_completion;
use crate::projection_consumption::{
    ProjectMaterializedFacts, ProjectionConsumptionWarningKind, ProjectionFactConsumptionAttempt,
};

#[test]
fn read_result_common_path_consumes_projection_facts_through_receipt() {
    let result = read_result();
    let result_shape = read_result_shape();
    let authorized_projection = authorized_projection(
        "query:test",
        result_shape.digest().as_str(),
        &["profile.display_name", "metrics.priority"],
    );

    let attempt = result
        .consume_projection_facts(
            &result_shape,
            &authorized_projection,
            ProjectMaterializedFacts::declare()
                .entity_identities()
                .display_field_path(
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
        ProjectionFactConsumptionAttempt::Admitted(completed) => {
            assert_common_path_completion(&completed, 2, 4);
        }
        other => panic!("expected admitted common-path read consumption, got {other:?}"),
    }
}

#[test]
fn write_receipt_common_path_consumes_projection_facts_through_receipt() {
    let receipt = write_receipt();
    let authorized_projection =
        authorized_projection("query:test", "result-shape:test", &["identity.id"]);

    let attempt = receipt
        .consume_projection_facts(
            "result-shape:test",
            &authorized_projection,
            ProjectMaterializedFacts::declare()
                .target_identity()
                .source_references(),
        )
        .unwrap();

    match attempt {
        ProjectionFactConsumptionAttempt::Admitted(completed) => {
            assert_common_path_completion(&completed, 2, 3);
        }
        other => panic!("expected admitted common-path write consumption, got {other:?}"),
    }
}

#[test]
fn query_context_common_path_preserves_warning_bearing_admission() {
    let execution = query_context_execution_preview();
    let authorized_projection =
        authorized_projection("query:test", "result-shape:test", &["profile.display_name"]);

    let attempt = execution
        .consume_projection_facts(
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
        ProjectionFactConsumptionAttempt::AdmittedWithWarnings(completed, warnings) => {
            assert_eq!(
                warnings.warning_kinds(),
                [ProjectionConsumptionWarningKind::PreviewDerivedContext]
            );
            assert_common_path_completion(&completed, 1, 1);
            assert_eq!(
                completed.receipt().warning_kinds(),
                [ProjectionConsumptionWarningKind::PreviewDerivedContext]
            );
            assert_eq!(
                completed.projection_consumption_envelope().warning_kinds(),
                [ProjectionConsumptionWarningKind::PreviewDerivedContext]
            );
        }
        other => {
            panic!("expected warning-bearing common-path query-context consumption, got {other:?}")
        }
    }
}

#[test]
fn completed_common_path_exposes_receipt_first_inspection_helpers() {
    let result = read_result();
    let result_shape = read_result_shape();
    let authorized_projection = authorized_projection(
        "query:test",
        result_shape.digest().as_str(),
        &["profile.display_name", "metrics.priority"],
    );

    let attempt = result
        .consume_projection_facts(
            &result_shape,
            &authorized_projection,
            ProjectMaterializedFacts::declare()
                .entity_identities()
                .display_field_path(
                    crate::projection_consumption::projection_fact_field_path_from_segments([
                        worth_foundational::facade::FieldKey::new("profile")
                            .expect("projection fact field segment should admit"),
                        worth_foundational::facade::FieldKey::new("display_name")
                            .expect("projection fact field segment should admit"),
                    ]),
                ),
        )
        .unwrap();

    let completed = attempt.completed().expect("read path should be admitted");
    assert_eq!(
        completed.source_family(),
        completed.receipt().source_family()
    );
    assert_eq!(
        completed.source_identity(),
        completed.receipt().source_identity()
    );
    assert_eq!(
        completed.support_posture(),
        completed.receipt().support_posture()
    );
    assert_eq!(
        completed.warning_kinds(),
        completed.receipt().warning_kinds()
    );
    assert_eq!(
        completed.admitted_fact_family_count(),
        completed.receipt().admitted_fact_family_count()
    );
    assert_eq!(
        completed.extracted_fact_count(),
        completed.receipt().extracted_fact_count()
    );
    assert_eq!(
        completed.authority_reopen_count(),
        completed.receipt().authority_reopen_count()
    );
    assert_eq!(
        completed.deferred_neighbors(),
        completed.receipt().deferred_neighbors()
    );
    assert_eq!(
        completed.transition_rules().rules_digest(),
        completed
            .receipt()
            .transition_rules()
            .expect("fixture digest must fit")
            .rules_digest()
    );
}
