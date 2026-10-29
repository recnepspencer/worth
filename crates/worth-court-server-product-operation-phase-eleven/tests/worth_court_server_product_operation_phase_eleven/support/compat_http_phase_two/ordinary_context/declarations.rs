#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_two_runtime as compat_http_phase_two_runtime};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_direct_context_runtime as direct_context_runtime};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_worth_native_assertions as worth_native_assertions};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_worth_native_runtime as worth_native_runtime};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_query::facade::runtime::WorthQueryInspection, worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection, worth_query::facade::runtime::WorthQueryRuntimeRemaskReasonKind};
pub(crate) use {worth_server::WorthServerCompatibilityExecutionInput, worth_server::WorthServerCompatibilityInspection, worth_server::WorthServerCompatibilityRead, worth_server::WorthServerCompatibilityState, worth_server::WorthServerQueryHandoffDenialCode};

pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_two_runtime::branch_head_execution_input, worth_testkit_server::shared_support_compat_http_phase_two_runtime::build_phase_two_server, worth_testkit_server::shared_support_compat_http_phase_two_runtime::build_phase_two_server_with_workspace_provider, worth_testkit_server::shared_support_compat_http_phase_two_runtime::compat_execution_input, worth_testkit_server::shared_support_compat_http_phase_two_runtime::prepared_read_request, worth_testkit_server::shared_support_compat_http_phase_two_runtime::read_input, worth_testkit_server::shared_support_compat_http_phase_two_runtime::worth_native_named_read};
pub(crate) use {worth_testkit_server::shared_support_direct_context_runtime::RemaskWorkspaceProvider};
pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::PanicOnReadTestWorkspaceProvider};
pub(crate) use {worth_testkit_server::shared_support_worth_native_assertions::direct_provenance_digest, worth_testkit_server::shared_support_worth_native_assertions::family_contract_digest, worth_testkit_server::shared_support_worth_native_assertions::response_provenance_digest};

pub(crate) fn compat_read_success(
    outcome: worth_server::WorthServerCompatibilityExecutionOutcome<WorthServerCompatibilityRead>,
) -> WorthServerCompatibilityRead {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility read success, got {other:?}"),
    }
}

pub(crate) fn compat_state_success(
    outcome: worth_server::WorthServerCompatibilityExecutionOutcome<WorthServerCompatibilityState>,
) -> WorthServerCompatibilityState {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility state success, got {other:?}"),
    }
}

pub(crate) fn compat_inspection_success(
    outcome: worth_server::WorthServerCompatibilityExecutionOutcome<
        WorthServerCompatibilityInspection,
    >,
) -> WorthServerCompatibilityInspection {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility inspection success, got {other:?}"),
    }
}

pub(crate) fn compat_denial<T: std::fmt::Debug>(
    outcome: worth_server::WorthServerCompatibilityExecutionOutcome<T>,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected compatibility denial, got {other:?}"),
    }
}

pub(crate) fn assert_private_cache_policy(cache_policy: &worth_server::WorthServerCompatibilityCachePolicy) {
    assert_eq!(cache_policy.cache_control(), "private, no-store");
    assert!(!cache_policy.publicly_reusable());
    assert_eq!(
        cache_policy.vary(),
        ["authorization", "x-Worth-branch", "x-Worth-diagnostics"].map(str::to_string)
    );
}