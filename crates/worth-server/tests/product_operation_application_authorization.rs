#[path = "support/test_bootstrap_policy.rs"]
mod test_bootstrap_policy;

use std::sync::{Arc, Mutex};

use worth_server::{
    WorthServerProductOperationAuthorization, WorthServerProductOperationAuthorizationDenial,
    WorthServerProductOperationAuthorizationRequest, WorthServerProductOperationAuthorizer,
    WorthServerProductOperationExecutionBoundary, WorthServerProductOperationInput,
    WorthServerProductOperationSurfaceDenialCode, WorthServerTransportCallerAdmissionRequest,
    WorthServerTransportCallerVerification, WorthServerTransportCallerVerifier,
    WorthServerVerifiedTransportCaller,
};

#[path = "support/product_operation_phase_thirteen/fixture.rs"]
mod fixture;
#[path = "support/product_operation_phase_thirteen/parity_driver.rs"]
mod parity_driver;

use fixture::{
    build_server_with_operation_authority, direct_session, direct_session_with_proof,
    render_payload, StatefulEditorLikeBackend,
};
use parity_driver::WorthServerRouteHttpTestDriver;

const AUTHORITY_PROOF: &str = "application-decision-1";
const CAPABILITY_PLAN: &str = "capability-plan-1";

#[derive(Debug)]
struct TestTransportVerifier;

impl WorthServerTransportCallerVerifier for TestTransportVerifier {
    fn verify(
        &self,
        _request: &WorthServerTransportCallerAdmissionRequest,
    ) -> WorthServerTransportCallerVerification {
        WorthServerTransportCallerVerification::Verified(
            WorthServerVerifiedTransportCaller::new(
                "principal-7",
                AUTHORITY_PROOF,
                "browser-session-1",
                "test-application-session-v1",
                1,
            )
            .unwrap(),
        )
    }
}

#[derive(Debug, Default)]
struct RecordingAuthorizer {
    surfaces: Mutex<Vec<String>>,
}

impl WorthServerProductOperationAuthorizer for RecordingAuthorizer {
    fn authorize(
        &self,
        request: &WorthServerProductOperationAuthorizationRequest<'_>,
    ) -> Result<
        WorthServerProductOperationAuthorization,
        WorthServerProductOperationAuthorizationDenial,
    > {
        let proof = request
            .application_authority_proof_identity()
            .ok_or_else(|| {
                WorthServerProductOperationAuthorizationDenial::new(
                    "application_proof_missing",
                    "application authority proof is required",
                )
            })?;
        if proof != AUTHORITY_PROOF || request.operation_name() != "product_editor.render" {
            return Err(WorthServerProductOperationAuthorizationDenial::new(
                "application_proof_denied",
                "application authority proof was not admitted",
            ));
        }
        self.surfaces.lock().unwrap().push(format!(
            "{:?}",
            request
                .operation_request()
                .resolved_request_context()
                .surface_family()
        ));
        WorthServerProductOperationAuthorization::new(
            proof,
            CAPABILITY_PLAN,
            "authority-snapshot-1",
        )
        .map_err(|detail| {
            WorthServerProductOperationAuthorizationDenial::new("application_proof_invalid", detail)
        })
    }
}

#[tokio::test]
async fn native_and_projected_http_share_one_application_authorization_plan() {
    let backend = StatefulEditorLikeBackend::new();
    let authorizer = Arc::new(RecordingAuthorizer::default());
    let server = build_server_with_operation_authority(
        &backend,
        Arc::new(TestTransportVerifier),
        authorizer.clone(),
    );
    let basis = backend.basis_digest();

    let missing = direct_session(&server)
        .product_operations()
        .execute(
            WorthServerProductOperationInput::new("product_editor.render", render_payload())
                .with_basis_digest(&basis),
        )
        .expect_err("native execution without an application proof must deny");
    assert_eq!(
        missing.code(),
        WorthServerProductOperationSurfaceDenialCode::AdmissionDenied
    );
    assert_eq!(
        missing.facts().and_then(|facts| facts.execution_boundary()),
        Some(&WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution)
    );

    let native = direct_session_with_proof(&server, AUTHORITY_PROOF)
        .product_operations()
        .execute(
            WorthServerProductOperationInput::new("product_editor.render", render_payload())
                .with_basis_digest(&basis),
        )
        .expect("native execution with an admitted proof should succeed");
    let native_authorization = native
        .plan()
        .and_then(|plan| plan.application_authorization())
        .expect("native lowered plan should retain application authorization");
    assert_eq!(native_authorization.plan_digest(), CAPABILITY_PLAN);

    let projected = WorthServerRouteHttpTestDriver::new(&server)
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
