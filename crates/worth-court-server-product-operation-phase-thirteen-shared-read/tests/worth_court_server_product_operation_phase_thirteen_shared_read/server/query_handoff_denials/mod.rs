#[allow(unused_imports)]
use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

#[allow(unused_imports)]
use worth_testkit_server::shared_support_query_handoff_fixture as query_handoff_fixture;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime;

use worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider;
use {
    worth_testkit_server::shared_support_query_handoff_fixture::admit_mutation_posture,
    worth_testkit_server::shared_support_query_handoff_fixture::admit_read_posture,
    worth_testkit_server::shared_support_query_handoff_fixture::denied,
    worth_testkit_server::shared_support_query_handoff_fixture::request_input,
    worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context,
    worth_testkit_server::shared_support_query_handoff_fixture::test_server,
    worth_testkit_server::shared_support_query_handoff_fixture::test_server_with_middleware,
};

#[test]
fn prepare_denies_durable_resume_but_admits_runtime_backed_resume() {
    let server = test_server(TestWorkspaceProvider, false);
    let admission = admit_read_posture(
        &server,
        resolve_request_context(
            &server,
            request_input(
                worth_server::WorthServerSurfaceFamily::WorthNative,
                worth_server::WorthServerTransportClass::WorthNativeInProcess,
            ),
        ),
    );

    let durable_denial = denied(server.query_handoff().prepare(
        worth_server::WorthServerQueryHandoffInput::new(
            admission.clone(),
            worth_server::WorthServerQueryHandoffOperation::downstream_delivery(
                "users.profile",
                worth_server::WorthServerDirectFreshnessMode::LiveStrict,
                worth_server::WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                worth_server::WorthServerQueryRequestedResume::durable(),
            ),
        ),
    ));
    let runtime_backed = worth_testkit_server::shared_support_query_handoff_fixture::success(
        server
            .query_handoff()
            .prepare(worth_server::WorthServerQueryHandoffInput::new(
                admission,
                worth_server::WorthServerQueryHandoffOperation::downstream_delivery(
                    "users.profile",
                    worth_server::WorthServerDirectFreshnessMode::LiveStrict,
                    worth_server::WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                    worth_server::WorthServerQueryRequestedResume::runtime_backed(None::<String>),
                ),
            )),
    );

    assert_eq!(
        durable_denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::DurableResumeDeferred
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
    let denial = denied(server.query_handoff().prepare(
        worth_server::WorthServerQueryHandoffInput::new(
            admit_read_posture(
                &server,
                resolve_request_context(
                    &server,
                    request_input(
                        worth_server::WorthServerSurfaceFamily::WorthNative,
                        worth_server::WorthServerTransportClass::WorthNativeInProcess,
                    ),
                ),
            ),
            worth_server::WorthServerQueryHandoffOperation::query_mutation("users.rename"),
        ),
    ));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::PreparedIntentMismatch
    );
}

#[test]
fn prepare_denies_read_handoff_when_query_workspace_does_not_admit_read_family() {
    let server = test_server(
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                worth_query::facade::runtime::WorthQueryRuntimeFamilySupport::unsupported(
                    worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::Read,
                    "read is intentionally denied in this hostile test profile",
                ),
            ),
        ),
        false,
    );

    let denial = denied(server.query_handoff().prepare(
        worth_server::WorthServerQueryHandoffInput::new(
            admit_read_posture(
                &server,
                resolve_request_context(
                    &server,
                    request_input(
                        worth_server::WorthServerSurfaceFamily::WorthNative,
                        worth_server::WorthServerTransportClass::WorthNativeInProcess,
                    ),
                ),
            ),
            worth_server::WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
    ));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert!(denial
        .detail()
        .contains("query workspace does not admit `read` facade family"));
}

#[test]
fn prepare_denies_downstream_delivery_when_query_workspace_does_not_admit_live_family() {
    let server = test_server(
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                worth_query::facade::runtime::WorthQueryRuntimeFamilySupport::unsupported(
                    worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::Live,
                    "live delivery is intentionally denied in this hostile test profile",
                ),
            ),
        ),
        false,
    );

    let denial = denied(server.query_handoff().prepare(
        worth_server::WorthServerQueryHandoffInput::new(
            admit_read_posture(
                &server,
                resolve_request_context(
                    &server,
                    request_input(
                        worth_server::WorthServerSurfaceFamily::WorthNative,
                        worth_server::WorthServerTransportClass::WorthNativeInProcess,
                    ),
                ),
            ),
            worth_server::WorthServerQueryHandoffOperation::downstream_delivery(
                "users.profile",
                worth_server::WorthServerDirectFreshnessMode::LiveStrict,
                worth_server::WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                worth_server::WorthServerQueryRequestedResume::none(),
            ),
        ),
    ));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
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
        worth_server::WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );
    let mutation_admission = admit_mutation_posture(
        &server,
        resolve_request_context(
            &server,
            request_input(
                worth_server::WorthServerSurfaceFamily::WorthNative,
                worth_server::WorthServerTransportClass::WorthNativeInProcess,
            ),
        ),
    );

    let denial = denied(server.query_handoff().prepare(
        worth_server::WorthServerQueryHandoffInput::new(
            mutation_admission,
            worth_server::WorthServerQueryHandoffOperation::downstream_delivery(
                "users.rename",
                worth_server::WorthServerDirectFreshnessMode::LiveStrict,
                worth_server::WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                worth_server::WorthServerQueryRequestedResume::none(),
            ),
        ),
    ));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::DownstreamDeliveryRequiresReadIntent
    );
}
