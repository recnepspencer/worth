#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

pub(crate) use {worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily, worth_query::facade::runtime::WorthQueryRuntimeFamilySupport, worth_query::facade::runtime::WorthQueryRuntimeSupportProfile};
pub(crate) use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::WorthServerDirectDeliveryClass, worth_server::WorthServerDirectFreshnessMode, worth_server::WorthServerOperationPlanDenialCode, worth_server::WorthServerOperationPlannerInput, worth_server::WorthServerQueryHandoffOperation, worth_server::WorthServerQueryRequestedResume, worth_server::WorthServerRequestContextInput, worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_two_runtime as compat_http_phase_two_runtime};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_query_handoff_fixture as query_handoff_fixture};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_worth_native_assertions as worth_native_assertions};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_worth_native_runtime as worth_native_runtime};

pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_two_runtime::build_phase_two_server, worth_testkit_server::shared_support_compat_http_phase_two_runtime::compat_execution_input, worth_testkit_server::shared_support_compat_http_phase_two_runtime::worth_native_named_read};
pub(crate) use {worth_testkit_server::shared_support_query_handoff_fixture::admit_delivery_posture, worth_testkit_server::shared_support_query_handoff_fixture::admit_read_posture, worth_testkit_server::shared_support_query_handoff_fixture::request_input, worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context, worth_testkit_server::shared_support_query_handoff_fixture::test_server};
pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider};
pub(crate) use {worth_testkit_server::shared_support_worth_native_runtime::server_with_request_context_default};

pub(crate) fn direct_read_success(
    outcome: worth_server::WorthServerDirectReadOutcome,
) -> worth_server::WorthServerDirectRead {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected direct read success, got {other:?}"),
    }
}

pub(crate) fn compat_read_success(
    outcome: worth_server::WorthServerCompatibilityExecutionOutcome<
        worth_server::WorthServerCompatibilityRead,
    >,
) -> worth_server::WorthServerCompatibilityRead {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility read success, got {other:?}"),
    }
}