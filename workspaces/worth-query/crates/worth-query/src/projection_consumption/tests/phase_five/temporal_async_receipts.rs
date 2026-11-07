use crate::authorized_projection::{
    AuthorizedProjectionArtifact, AuthorizedProjectionCounters, MaskedProjectionArtifact,
    PolicyFieldInfluenceSet,
};
use crate::projection_consumption::{
    ProjectMaterializedFacts, ProjectionFactConsumptionAttempt, ProjectionMaterializedFactPosture,
    ProjectionMaterializedFactPostureKind,
};
use crate::runtime::{WorthQueryReadExecutionEngine, WorthQueryReadReceipt, WorthQueryReadResult};

use super::super::phase_four::support::{
    authorized_projection, entity_from_projection, int_value, query_context_execution_preview,
    read_result_shape, text_value,
};

fn temporal_async_posture(
    kind: ProjectionMaterializedFactPostureKind,
    lower_declaration_digest: &str,
    basis_digest: &str,
    support_digest: &str,
) -> ProjectionMaterializedFactPosture {
    ProjectionMaterializedFactPosture::new(
        kind,
        lower_declaration_digest,
        basis_digest,
        support_digest,
        Some(format!("runtime-origin:{lower_declaration_digest}")),
    )
    .expect("fixture digest must fit")
}

fn read_result_with_posture(
    kind: ProjectionMaterializedFactPostureKind,
    lower_declaration_digest: &str,
    basis_digest: &str,
    support_digest: &str,
) -> WorthQueryReadResult {
    WorthQueryReadResult::test_only(
        vec![entity_from_projection(
            crate::memory_workspace::admit_authored_entity_label("task-1")
                .expect("fixture digest must fit"),
            [
                ("profile.display_name", text_value("Task One")),
                ("metrics.priority", int_value(1)),
            ],
        )],
        WorthQueryReadReceipt::test_only(
            "read-graph:test",
            "query:test",
            basis_digest,
            "result:test",
            WorthQueryReadExecutionEngine::QueryRuntimeCurrent,
        )
        .expect("fixture digest must fit")
        .test_only_with_materialized_fact_posture(temporal_async_posture(
            kind,
            lower_declaration_digest,
            basis_digest,
            support_digest,
        )),
    )
}

fn authorized_projection_with_policy(
    result_shape_digest: &str,
    policy_digest: &str,
) -> AuthorizedProjectionArtifact {
    AuthorizedProjectionArtifact::new(
        "query:test",
        result_shape_digest,
        policy_digest,
        "tenant-schema:test",
        crate::projection_consumption::test_authorized_field_paths(&["profile.display_name"]),
        MaskedProjectionArtifact::new(Vec::new(), Vec::new()),
        "narrowed-result-shape:test".to_string(),
        PolicyFieldInfluenceSet::new(&["influence:test".to_string()], 1),
        AuthorizedProjectionCounters::default(),
    )
}

#[test]
fn time_only_materialized_read_receipt_retains_projection_consumption_posture() {
    let result_shape = read_result_shape();
    let authorized_projection = authorized_projection(
        "query:test",
        result_shape.digest().as_str(),
        &["profile.display_name"],
    );
    let result = read_result_with_posture(
        ProjectionMaterializedFactPostureKind::TimeOnly,
        "declaration:time-only",
        "basis:time-only",
        "support:time-only",
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
        .expect("time-only read consumption should stay typed");

    let completed = match attempt {
        ProjectionFactConsumptionAttempt::Admitted(completed) => completed,
        other => panic!("expected admitted time-only read consumption, got {other:?}"),
    };
    let posture = completed
        .materialized_fact_posture()
        .expect("time-only posture should survive into completed consumption");

    assert_eq!(
        posture.kind(),
        ProjectionMaterializedFactPostureKind::TimeOnly
    );
    assert_eq!(posture.lower_declaration_digest(), "declaration:time-only");
    assert_eq!(posture.basis_digest(), "basis:time-only");
    assert_eq!(completed.facts().materialized_fact_posture(), Some(posture));
    assert_eq!(
        completed.receipt().materialized_fact_posture(),
        Some(posture)
    );
}

#[test]
fn async_backed_query_context_consumption_receipt_retains_materialized_posture() {
    let execution = query_context_execution_preview().test_only_with_materialized_fact_posture(
        temporal_async_posture(
            ProjectionMaterializedFactPostureKind::AsyncBacked,
            "declaration:async",
            "basis:async",
            "support:async",
        ),
    );
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
        .expect("async-backed query-context consumption should stay typed");

    let completed = match attempt {
        ProjectionFactConsumptionAttempt::AdmittedWithWarnings(completed, _) => completed,
        other => {
            panic!("expected warning-bearing async-backed query-context consumption, got {other:?}")
        }
    };
    let posture = completed
        .materialized_fact_posture()
        .expect("async-backed posture should survive into completed consumption");

    assert_eq!(
        posture.kind(),
        ProjectionMaterializedFactPostureKind::AsyncBacked
    );
    assert_eq!(posture.lower_declaration_digest(), "declaration:async");
    assert_eq!(
        completed.receipt().materialized_fact_posture(),
        Some(posture)
    );
}

#[test]
fn temporal_async_consumed_facts_remain_basis_policy_and_support_bound() {
    let result_shape = read_result_shape();
    let authorized_projection_left =
        authorized_projection_with_policy(result_shape.digest().as_str(), "policy:left");
    let authorized_projection_right =
        authorized_projection_with_policy(result_shape.digest().as_str(), "policy:right");
    let left = read_result_with_posture(
        ProjectionMaterializedFactPostureKind::MixedCause,
        "declaration:mixed",
        "basis:left",
        "support:left",
    );
    let right = read_result_with_posture(
        ProjectionMaterializedFactPostureKind::MixedCause,
        "declaration:mixed",
        "basis:right",
        "support:right",
    );

    let left = left
        .consume_projection_facts(
            &result_shape,
            &authorized_projection_left,
            ProjectMaterializedFacts::declare().display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
        )
        .expect("left mixed-cause consumption should stay typed")
        .completed()
        .expect("left mixed-cause consumption should admit")
        .clone();
    let right = right
        .consume_projection_facts(
            &result_shape,
            &authorized_projection_right,
            ProjectMaterializedFacts::declare().display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
        )
        .expect("right mixed-cause consumption should stay typed")
        .completed()
        .expect("right mixed-cause consumption should admit")
        .clone();

    assert_ne!(
        left.materialized_fact_posture()
            .expect("left posture")
            .posture_digest(),
        right
            .materialized_fact_posture()
            .expect("right posture")
            .posture_digest()
    );
    assert_ne!(
        left.facts().fact_set_digest(),
        right.facts().fact_set_digest()
    );
    assert_ne!(
        left.receipt().contract_digest(),
        right.receipt().contract_digest()
    );
    assert_ne!(
        left.receipt().receipt_digest(),
        right.receipt().receipt_digest()
    );
}
