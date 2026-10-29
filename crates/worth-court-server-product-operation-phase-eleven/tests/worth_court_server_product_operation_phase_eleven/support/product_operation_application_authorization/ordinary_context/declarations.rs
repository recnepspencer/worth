#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

pub(crate) use {std::sync::Arc, std::sync::Mutex};

pub(crate) use {worth_server::WorthServerProductOperationAuthorization, worth_server::WorthServerProductOperationAuthorizationDenial, worth_server::WorthServerProductOperationAuthorizationRequest, worth_server::WorthServerProductOperationAuthorizer, worth_server::WorthServerProductOperationExecutionBoundary, worth_server::WorthServerProductOperationInput, worth_server::WorthServerProductOperationSurfaceDenialCode, worth_server::WorthServerTransportCallerAdmissionRequest, worth_server::WorthServerTransportCallerVerification, worth_server::WorthServerTransportCallerVerifier, worth_server::WorthServerVerifiedTransportCaller};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture as fixture};
#[allow(unused_imports)]
pub(crate) use {};

pub(crate) use {worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::build_server_with_operation_authority, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::direct_session, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::direct_session_with_proof, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::render_payload, worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::StatefulEditorLikeBackend};
pub(crate) use {worth_testkit_server::shared_support_route_assembly_phase_twelve_request_driver::WorthServerRouteHttpTestDriver};

pub(crate) const AUTHORITY_PROOF: &str = "application-decision-1";
pub(crate) const CAPABILITY_PLAN: &str = "capability-plan-1";

#[derive(Debug)]
pub(crate) struct TestTransportVerifier;

impl WorthServerTransportCallerVerifier for TestTransportVerifier {
    fn verify(
        &self,
        _request: &WorthServerTransportCallerAdmissionRequest,
    ) -> WorthServerTransportCallerVerification {
        worth_server::WorthServerTransportCallerVerification::Verified(
            worth_server::WorthServerVerifiedTransportCaller::new(
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
pub(crate) struct RecordingAuthorizer {
    pub(crate) surfaces: Mutex<Vec<String>>,
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
                worth_server::WorthServerProductOperationAuthorizationDenial::new(
                    "application_proof_missing",
                    "application authority proof is required",
                )
            })?;
        if proof != AUTHORITY_PROOF || request.operation_name() != "product_editor.render" {
            return Err(worth_server::WorthServerProductOperationAuthorizationDenial::new(
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
        worth_server::WorthServerProductOperationAuthorization::new(
            proof,
            CAPABILITY_PLAN,
            "authority-snapshot-1",
        )
        .map_err(|detail| {
            worth_server::WorthServerProductOperationAuthorizationDenial::new("application_proof_invalid", detail)
        })
    }
}