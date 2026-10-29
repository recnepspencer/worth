#[allow(unused_imports)]
use crate::product_adapter_registration_contracts::ordinary_context::*;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

use std::sync::Arc;

use serde_json::json;
use tower::ServiceExt;
use {axum::body::to_bytes, axum::body::Body};
use {axum::http::Method, axum::http::Request, axum::http::StatusCode};
use {
    worth_query_host::facade::domain, worth_query_host::facade::primary_graph,
    worth_query_host::facade::runtime,
};
use {
    worth_query_host::facade::worth_query_application_schema,
    worth_query_host::facade::worth_query_aspect, worth_query_host::facade::worth_query_entity,
    worth_query_host::facade::worth_query_field,
    worth_query_host::facade::worth_query_principal_binding,
    worth_query_host::facade::worth_query_relation,
};
use {
    worth_server::CompatHttpSurface, worth_server::WorthNativeSurface, worth_server::WorthServer,
    worth_server::WorthServerBuildError, worth_server::WorthServerProductAdapterCertificationCode,
    worth_server::WorthServerProductApplicationAdapterRegistration,
    worth_server::WorthServerProductOperationBasisKind,
    worth_server::WorthServerProductOperationDeclaration,
    worth_server::WorthServerProductOperationErrorMaps,
    worth_server::WorthServerProductOperationSupportSnapshot,
};

#[allow(unused_imports)]
use worth_testkit_server::shared_support_product_adapter_phase_nine_fixture as fixture;

use {
    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::base_config,
    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::build_server,
    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::result_contract,
    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::EditorAdapter,
};

#[test]
fn product_adapter_registration_rejects_incomplete_authority_or_basis_contract() {
    let result = build_broken_registration_server(
        worth_server::WorthServerProductOperationDeclaration::product_mutation(
            "product_editor.apply",
            "product-editor.apply.v1",
            result_contract("product-editor.apply.result.v1"),
            worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
            worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(" "),
            "",
        )
        .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough()),
    );
    assert_registration_code(
        result,
        worth_server::WorthServerProductAdapterCertificationCode::BlankSupportSnapshotRow,
    );
}

#[test]
fn product_adapter_registration_rejects_missing_error_map_explicitly() {
    let result = build_broken_registration_server(
        worth_server::WorthServerProductOperationDeclaration::product_read(
            "product_editor.render",
            "product-editor.render.v1",
            result_contract("product-editor.render.result.v1"),
            worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
            worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                "render-supported",
            ),
        ),
    );
    assert_registration_code(
        result,
        worth_server::WorthServerProductAdapterCertificationCode::MissingErrorMap,
    );
}

#[test]
fn product_adapter_registration_rejects_blank_schema_identity_explicitly() {
    let result = build_broken_registration_server(
        worth_server::WorthServerProductOperationDeclaration::product_read(
            "product_editor.render",
            " ",
            result_contract("product-editor.render.result.v1"),
            worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
            worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                "render-supported",
            ),
        )
        .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough()),
    );
    assert_registration_code(
        result,
        worth_server::WorthServerProductAdapterCertificationCode::BlankPayloadSchemaIdentity,
    );
}

#[test]
fn primary_graph_application_registration_requires_its_query_owner() {
    let result = build_broken_registration_server(
        worth_server::WorthServerProductOperationDeclaration::product_read(
            "workflow_editor.render",
            "workflow-editor.render.v1",
            result_contract("workflow-editor.render.result.v1"),
            worth_server::WorthServerProductOperationBasisKind::PrimaryGraphApplication,
            worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                "workflow-editor-render-supported",
            ),
        )
        .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough()),
    );
    assert_registration_code(
        result,
        worth_server::WorthServerProductAdapterCertificationCode::MissingQueryApplicationReadinessProvider,
    );
}

#[tokio::test]
async fn mutation_session_response_carries_registered_primary_graph_basis() {
    let application = std::sync::Arc::new(primary_graph_application());
    let expected_basis = application
        .with_host_advancement(|host_phase| {
            application
                .on_branch(application.current_world())
                .select(&host_phase)
                .expect("current product branch must select")
                .inspect_application_readiness()
                .expect("published Query application must expose readiness")
                .basis_token()
                .to_string()
        })
        .expect("installed host call opens");
    let declaration = worth_server::WorthServerProductOperationDeclaration::product_mutation(
        "workflow_editor.apply",
        "workflow-editor.apply.v1",
        result_contract("workflow-editor.apply.result.v1"),
        worth_server::WorthServerProductOperationBasisKind::PrimaryGraphApplication,
        worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
            "workflow-editor-apply-supported",
        ),
        "draft",
    )
    .with_primary_graph_application(application)
    .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough());
    let server = build_server(vec![worth_server::WorthServerProductApplicationAdapterRegistration::new(
        "primary-graph-editor",
        std::sync::Arc::new(worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::EditorAdapter::default()),
    )
    .with_operation(declaration)]);
    let response = server
        .projected_router()
        .clone_axum_router()
        .oneshot(
            axum::http::Request::builder()
                .method(axum::http::Method::POST)
                .uri("/compat/mutations/product_session.open_mutation")
                .header("content-type", "application/json")
                .header("x-principal-id", "principal-7")
                .header("x-tenant-id", "tenant-a")
                .header("x-workspace-id", "workspace-42")
                .header("x-branch-id", "branch-9")
                .body(axum::body::Body::from(
                    serde_json::to_vec(&json!({
                        "operation_name": "workflow_editor.apply",
                        "expiry_seconds": 60
                    }))
                    .expect("session request must serialize"),
                ))
                .expect("session request must build"),
        )
        .await
        .expect("session route must respond");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("session response must read");
    let body: serde_json::Value =
        serde_json::from_slice(&body).expect("session response must be JSON");
    assert_eq!(body["basis"].as_str(), Some(expected_basis.as_str()));
}
