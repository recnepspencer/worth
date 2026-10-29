#[allow(unused_imports)] use crate::product_operation_application_authorization::ordinary_context::*;
#[allow(unused_imports)]
use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

use {std::sync::Arc, std::sync::Mutex};

use {worth_server::WorthServerProductOperationAuthorization, worth_server::WorthServerProductOperationAuthorizationDenial, worth_server::WorthServerProductOperationAuthorizationRequest, worth_server::WorthServerProductOperationAuthorizer, worth_server::WorthServerProductOperationExecutionBoundary, worth_server::WorthServerProductOperationInput, worth_server::WorthServerProductOperationSurfaceDenialCode, worth_server::WorthServerTransportCallerAdmissionRequest, worth_server::WorthServerTransportCallerVerification, worth_server::WorthServerTransportCallerVerifier, worth_server::WorthServerVerifiedTransportCaller};

#[allow(unused_imports)]
use {worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture as fixture};
#[allow(unused_imports)]
use {};

use {worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::build_server_with_operation_authority, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::direct_session, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::direct_session_with_proof, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::render_payload, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::StatefulEditorLikeBackend};
use {worth_testkit_server::shared_support_route_assembly_phase_twelve_request_driver::WorthServerRouteHttpTestDriver};

#[tokio::test]
async fn native_and_projected_http_share_one_application_authorization_plan() {
    let backend = worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::StatefulEditorLikeBackend::new();
    let authorizer = std::sync::Arc::new(crate::product_operation_application_authorization::ordinary_context::RecordingAuthorizer::default());
    let server = build_server_with_operation_authority(
        &backend,
        std::sync::Arc::new(TestTransportVerifier),
        authorizer.clone(),
    );
    let basis = backend.basis_digest();

    let missing = direct_session(&server)
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.render", render_payload())
                .with_basis_digest(&basis),
        )
        .expect_err("native execution without an application proof must deny");
    assert_eq!(
        missing.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(
        missing.facts().and_then(|facts| facts.execution_boundary()),
        Some(&worth_server::WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution)
    );

    let native = direct_session_with_proof(&server, AUTHORITY_PROOF)
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new("product_editor.render", render_payload())
                .with_basis_digest(&basis),
        )
        .expect("native execution with an admitted proof should succeed");
    let native_authorization = native
        .plan()
        .and_then(|plan| plan.application_authorization())
        .expect("native lowered plan should retain application authorization");
    assert_eq!(native_authorization.plan_digest(), CAPABILITY_PLAN);

    let projected = worth_testkit_server::shared_support_route_assembly_phase_twelve_request_driver::WorthServerRouteHttpTestDriver::new(&server)
        .get(
            &format!("/compat/reads/product_editor.render?basis={basis}"),
            &[
                ("x-principal-id", "forged-principal"),
                ("x-tenant-id", "tenant-a"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
            ],
        )
        .await;
    assert_eq!(projected.status(), axum::http::StatusCode::OK);
    assert_eq!(
        authorizer.surfaces.lock().unwrap().as_slice(),
        ["WorthNative", "CompatHttp"]
    );
}
