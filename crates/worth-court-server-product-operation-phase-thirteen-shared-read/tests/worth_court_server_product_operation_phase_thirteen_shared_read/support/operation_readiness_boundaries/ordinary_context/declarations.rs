#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_server::WorthServerOperationFamily, worth_server::WorthServerOperationReadinessDenialCode, worth_server::WorthServerOperationRequestInput, worth_server::WorthServerPipelineInput, worth_server::WorthServerPipelineIntent, worth_server::WorthServerQueryHandoffDenialFamily, worth_server::WorthServerQueryHandoffInput, worth_server::WorthServerQueryHandoffOperation};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_three_runtime as compat_http_phase_three_runtime};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_operation_request_runtime as operation_request_runtime};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_query_handoff_fixture as query_handoff_fixture};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime};

pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_three_runtime::build_phase_three_server, worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_denied, worth_testkit_server::shared_support_compat_http_phase_three_runtime::mutation_input, worth_testkit_server::shared_support_compat_http_phase_three_runtime::prepared_mutation_request, worth_testkit_server::shared_support_compat_http_phase_three_runtime::single_insert_body};
pub(crate) use {worth_testkit_server::shared_support_operation_request_runtime::operation_request_test_server, worth_testkit_server::shared_support_operation_request_runtime::worth_native_resolved_context};
pub(crate) use {worth_testkit_server::shared_support_query_handoff_fixture::request_input, worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context, worth_testkit_server::shared_support_query_handoff_fixture::test_server};
pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider};

pub(crate) fn query_read_admission(server: &worth_server::WorthServer) -> worth_server::WorthServerAdmission {
    match server.middleware().admit(worth_server::WorthServerPipelineInput::new(
        worth_native_resolved_context(server, None),
        worth_server::WorthServerPipelineIntent::query_read("users.profile"),
    )) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected query read admission, got {other:?}"),
    }
}