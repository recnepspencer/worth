#[path = "support/test_execution_policy.rs"]
mod test_execution_policy;
use worth_query::facade::runtime::{
    WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupport, WorthQueryRuntimeSupportProfile,
};
use worth_server::{
    WorthServerDirectDeliveryClass, WorthServerDirectFreshnessMode, WorthServerMiddlewareConfig,
    WorthServerQueryHandoffDenialCode, WorthServerQueryHandoffInput,
    WorthServerQueryHandoffOperation, WorthServerQueryRequestedResume, WorthServerSurfaceFamily,
    WorthServerTransportClass,
};

#[path = "support/query_handoff/fixture.rs"]
mod query_handoff_fixture;
#[path = "support/query_handoff/runtime.rs"]
mod query_handoff_runtime;

use query_handoff_fixture::{
    admit_mutation_posture, admit_read_posture, denied, request_input, resolve_request_context,
    test_server, test_server_with_middleware,
};
use query_handoff_runtime::{ProfiledTestWorkspaceProvider, TestWorkspaceProvider};

#[test]
fn prepare_denies_durable_resume_but_admits_runtime_backed_resume() {
    let server = test_server(TestWorkspaceProvider, false);
    let admission = admit_read_posture(
        &server,
        resolve_request_context(
            &server,
            request_input(
                WorthServerSurfaceFamily::WorthNative,
                WorthServerTransportClass::WorthNativeInProcess,
            ),
        ),
    );

    let durable_denial = denied(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admission.clone(),
                WorthServerQueryHandoffOperation::downstream_delivery(
                    "users.profile",
                    WorthServerDirectFreshnessMode::LiveStrict,
                    WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                    WorthServerQueryRequestedResume::durable(),
                ),
            )),
    );
    let runtime_backed = query_handoff_fixture::success(server.query_handoff().prepare(
        WorthServerQueryHandoffInput::new(
            admission,
            WorthServerQueryHandoffOperation::downstream_delivery(
                "users.profile",
                WorthServerDirectFreshnessMode::LiveStrict,
                WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                WorthServerQueryRequestedResume::runtime_backed(None::<String>),
            ),
        ),
    ));

    assert_eq!(
        durable_denial.code(),
        WorthServerQueryHandoffDenialCode::DurableResumeDeferred
    );
    assert!(durable_denial
        .detail()
        .contains("durable resume remains deferred"));
    assert!(matches!(
        runtime_backed.support_posture(),
        worth_server::WorthServerQuerySupportPosture::RuntimeBackedResumeSupported { .. }
    ));
}

#[test]
fn prepare_denies_mismatched_read_intent_before_query_binding() {
    let server = test_server(TestWorkspaceProvider, false);
    let denial = denied(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admit_read_posture(
                    &server,
                    resolve_request_context(
                        &server,
                        request_input(
                            WorthServerSurfaceFamily::WorthNative,
                            WorthServerTransportClass::WorthNativeInProcess,
                        ),
                    ),
                ),
                WorthServerQueryHandoffOperation::query_mutation("users.rename"),
            )),
    );

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::PreparedIntentMismatch
    );
}

#[test]
fn prepare_denies_read_handoff_when_query_workspace_does_not_admit_read_family() {
    let server = test_server(
        ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                WorthQueryRuntimeFamilySupport::unsupported(
                    WorthQueryRuntimeFacadeFamily::Read,
                    "read is intentionally denied in this hostile test profile",
                ),
            ),
        ),
        false,
    );

    let denial = denied(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admit_read_posture(
                    &server,
                    resolve_request_context(
                        &server,
                        request_input(
                            WorthServerSurfaceFamily::WorthNative,
                            WorthServerTransportClass::WorthNativeInProcess,
                        ),
                    ),
                ),
                WorthServerQueryHandoffOperation::query_read("users.profile"),
            )),
    );

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert!(denial
        .detail()
        .contains("query workspace does not admit `read` facade family"));
}

#[test]
fn prepare_denies_downstream_delivery_when_query_workspace_does_not_admit_live_family() {
    let server = test_server(
        ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                WorthQueryRuntimeFamilySupport::unsupported(
                    WorthQueryRuntimeFacadeFamily::Live,
                    "live delivery is intentionally denied in this hostile test profile",
                ),
            ),
        ),
        false,
    );

    let denial = denied(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admit_read_posture(
                    &server,
                    resolve_request_context(
                        &server,
                        request_input(
                            WorthServerSurfaceFamily::WorthNative,
                            WorthServerTransportClass::WorthNativeInProcess,
                        ),
                    ),
                ),
                WorthServerQueryHandoffOperation::downstream_delivery(
                    "users.profile",
                    WorthServerDirectFreshnessMode::LiveStrict,
                    WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                    WorthServerQueryRequestedResume::none(),
                ),
            )),
    );

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert!(denial
        .detail()
        .contains("query workspace does not admit `live` facade family"));
}

#[test]
fn prepare_denies_downstream_delivery_when_middleware_only_admitted_mutation_intent() {
    let server = test_server_with_middleware(
        TestWorkspaceProvider,
        false,
        WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );
    let mutation_admission = admit_mutation_posture(
        &server,
        resolve_request_context(
            &server,
            request_input(
                WorthServerSurfaceFamily::WorthNative,
                WorthServerTransportClass::WorthNativeInProcess,
            ),
        ),
    );

    let denial = denied(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                mutation_admission,
                WorthServerQueryHandoffOperation::downstream_delivery(
                    "users.rename",
                    WorthServerDirectFreshnessMode::LiveStrict,
                    WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                    WorthServerQueryRequestedResume::none(),
                ),
            )),
    );

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::DownstreamDeliveryRequiresReadIntent
    );
}
