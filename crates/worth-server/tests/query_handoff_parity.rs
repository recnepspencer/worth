#[path = "support/test_bootstrap_policy.rs"]
mod test_bootstrap_policy;
use worth_server::{
    WorthServerMiddlewareConfig, WorthServerQueryHandoffInput, WorthServerQueryHandoffOperation,
    WorthServerSurfaceFamily, WorthServerTransportClass,
};

#[path = "support/query_handoff/fixture.rs"]
mod query_handoff_fixture;
#[path = "support/query_handoff/runtime.rs"]
mod query_handoff_runtime;

use query_handoff_fixture::{
    admit_mutation_posture, admit_read_posture, request_input, resolve_request_context, success,
    test_server, test_server_with_middleware,
};
use query_handoff_runtime::TestWorkspaceProvider;

#[test]
fn prepare_lowers_equivalent_cross_surface_reads_to_the_same_canonical_handoff_artifact() {
    let server = test_server(TestWorkspaceProvider, false);
    let worth_native = success(
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
    let compat_http = success(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admit_read_posture(
                    &server,
                    resolve_request_context(
                        &server,
                        request_input(
                            WorthServerSurfaceFamily::CompatHttp,
                            WorthServerTransportClass::CompatHttp,
                        ),
                    ),
                ),
                WorthServerQueryHandoffOperation::query_read("users.profile"),
            )),
    );

    assert_eq!(
        worth_native.canonical_digest(),
        compat_http.canonical_digest()
    );
    assert_eq!(
        worth_native.support_posture(),
        compat_http.support_posture()
    );
    assert_eq!(worth_native.operation(), compat_http.operation());
    assert_eq!(
        worth_native.workspace().name(),
        compat_http.workspace().name()
    );
}

#[test]
fn prepare_lowers_equivalent_cross_surface_mutations_to_the_same_canonical_handoff_artifact() {
    let server = test_server_with_middleware(
        TestWorkspaceProvider,
        false,
        WorthServerMiddlewareConfig::builder()
            .with_query_mutation_enabled(true)
            .build()
            .expect("middleware config should validate"),
    );
    let worth_native = success(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admit_mutation_posture(
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
    let compat_http = success(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admit_mutation_posture(
                    &server,
                    resolve_request_context(
                        &server,
                        request_input(
                            WorthServerSurfaceFamily::CompatHttp,
                            WorthServerTransportClass::CompatHttp,
                        ),
                    ),
                ),
                WorthServerQueryHandoffOperation::query_mutation("users.rename"),
            )),
    );

    assert_eq!(
        worth_native.canonical_digest(),
        compat_http.canonical_digest()
    );
    assert_eq!(
        worth_native.support_posture(),
        compat_http.support_posture()
    );
    assert_eq!(worth_native.operation(), compat_http.operation());
}

#[test]
fn prepare_keeps_future_surface_registration_out_of_query_handoff_truth() {
    let plain_server = test_server(TestWorkspaceProvider, false);
    let widened_server = test_server(TestWorkspaceProvider, true);

    let plain = success(
        plain_server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admit_read_posture(
                    &plain_server,
                    resolve_request_context(
                        &plain_server,
                        request_input(
                            WorthServerSurfaceFamily::WorthNative,
                            WorthServerTransportClass::WorthNativeInProcess,
                        ),
                    ),
                ),
                WorthServerQueryHandoffOperation::query_read("users.profile"),
            )),
    );
    let widened = success(widened_server.query_handoff().prepare(
        WorthServerQueryHandoffInput::new(
            admit_read_posture(
                &widened_server,
                resolve_request_context(
                    &widened_server,
                    request_input(
                        WorthServerSurfaceFamily::WorthNative,
                        WorthServerTransportClass::WorthNativeInProcess,
                    ),
                ),
            ),
            WorthServerQueryHandoffOperation::query_read("users.profile"),
        ),
    ));

    assert_eq!(plain.canonical_digest(), widened.canonical_digest());
    assert_eq!(plain.support_posture(), widened.support_posture());
}
