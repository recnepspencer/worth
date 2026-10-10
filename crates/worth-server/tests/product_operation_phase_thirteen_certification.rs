#[path = "support/product_operation_phase_thirteen/fixture.rs"]
mod fixture;
#[path = "support/product_operation_phase_thirteen/proof_builders.rs"]
mod proof_builders;
#[path = "support/test_execution_policy.rs"]
mod test_execution_policy;

use fixture::{build_server, StatefulEditorLikeBackend};
use proof_builders::{blocked_editor_like_fixture, complete_editor_like_fixture};

#[tokio::test]
async fn worth_server_contains_no_product_semantic_branches() {
    let backend = StatefulEditorLikeBackend::new();
    let server = build_server(&backend);
    let certification = server
        .operation_runtime_certification()
        .certify_product_editor_readiness(complete_editor_like_fixture().await);

    assert!(certification.no_product_semantics_proof().is_ready());
    assert!(certification
        .no_product_semantics_proof()
        .product_families_are_generic());
    assert!(certification
        .no_product_semantics_proof()
        .product_routes_are_generic());
    assert!(certification
        .no_product_semantics_proof()
        .support_rows_remain_passthrough());
    assert!(certification
        .no_product_semantics_proof()
        .semantic_route_metadata_remains_generic());
}

#[tokio::test]
async fn product_operation_runtime_support_row_closes_only_with_all_phase_artifacts() {
    let backend = StatefulEditorLikeBackend::new();
    let server = build_server(&backend);

    let ready = server
        .operation_runtime_certification()
        .certify_product_editor_readiness(complete_editor_like_fixture().await);
    let blocked = server
        .operation_runtime_certification()
        .certify_product_editor_readiness(blocked_editor_like_fixture().await);

    assert!(ready.is_ready());
    assert_eq!(
        ready.support_row().readiness_label(),
        "product-operation-runtime-ready"
    );
    assert!(!blocked.is_ready());
    assert_eq!(
        blocked.support_row().readiness_label(),
        "product-operation-runtime-blocked"
    );
    assert!(blocked
        .support_row()
        .blocking_artifact_names()
        .contains(&"operation-planner"));
    assert!(blocked
        .support_row()
        .blocking_artifact_names()
        .contains(&"product-editor-readiness"));
    assert!(blocked
        .support_row()
        .blocking_artifact_names()
        .contains(&"precondition-posture"));
    assert!(ready
        .support_row()
        .requirements()
        .rows()
        .iter()
        .any(|row| row.artifact_name() == "authority-footprint"));
    assert!(ready
        .support_row()
        .requirements()
        .rows()
        .iter()
        .any(|row| row.artifact_name() == "authorization-posture"));
    assert!(ready
        .support_row()
        .requirements()
        .rows()
        .iter()
        .any(|row| row.artifact_name() == "support-posture"));
    assert!(ready
        .support_row()
        .requirements()
        .rows()
        .iter()
        .any(|row| row.artifact_name() == "precondition-posture"));
}
