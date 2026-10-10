use crate::runtime::{
    WorthQueryEffectPolicy, WorthQueryRuntimeError, WorthQueryRuntimeFacadeFamily,
    WorthQueryRuntimeFamilySupportStatus,
};

use super::support_profile::in_memory_test_backend_support_profile;
use super::{in_memory_test_runtime, WorthQueryTestBackendSchema};

#[test]
fn in_memory_test_runtime_support_profile_is_honest_about_debt_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = task_workspace(request_execution);
    let matrix = workspace.public_support_matrix();

    assert_eq!(
        matrix_row_status(&matrix, WorthQueryRuntimeFacadeFamily::Read),
        WorthQueryRuntimeFamilySupportStatus::Supported
    );
    assert_eq!(
        matrix_row_status(&matrix, WorthQueryRuntimeFacadeFamily::Write),
        WorthQueryRuntimeFamilySupportStatus::Supported
    );
    assert_eq!(
        matrix_row_status(&matrix, WorthQueryRuntimeFacadeFamily::Intent),
        WorthQueryRuntimeFamilySupportStatus::Unsupported
    );
    assert_eq!(
        matrix_row_status(&matrix, WorthQueryRuntimeFacadeFamily::StoreBackedExecution),
        WorthQueryRuntimeFamilySupportStatus::DeferredDebt
    );
    assert_eq!(
        in_memory_test_backend_support_profile()
            .support_for(WorthQueryRuntimeFacadeFamily::BranchPreview)
            .expect("branch-preview row should exist")
            .effect_policies(),
        &[
            WorthQueryEffectPolicy::DeriveOnly,
            WorthQueryEffectPolicy::SandboxedWriteIntent
        ]
    );
    for row in matrix.rows() {
        if row.status() != WorthQueryRuntimeFamilySupportStatus::Supported {
            assert!(
                row.admission_fail_closed(),
                "row should fail closed: {row:?}"
            );
            assert!(
                !row.extension_rule().is_empty(),
                "row should expose extension rule: {row:?}"
            );
        }
    }
}

#[test]
fn in_memory_test_runtime_support_matrix_denies_unsupported_public_families() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = task_workspace(request_execution);
    for family in unsupported_public_families() {
        let error = workspace
            .admit_public_api_family(family)
            .expect_err("unsupported/deferred family should fail closed");
        match error {
            WorthQueryRuntimeError::UnsupportedFacadeFamily(denial) => {
                assert_eq!(denial.family(), family);
            }
            other => panic!("expected unsupported facade family denial, got {other:?}"),
        }
    }
}

fn task_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::runtime::WorthQueryWorkspace {
    in_memory_test_runtime()
        .with_schema(task_schema())
        .workspace(
            request_execution,
            "consumer-kit.test-backend.support-profile",
        )
        .expect("in-memory test runtime should build")
}

fn task_schema() -> WorthQueryTestBackendSchema {
    super::contract_fixtures::task_schema()
}

fn matrix_row_status(
    matrix: &crate::runtime::WorthQueryRuntimePublicSupportMatrix,
    family: WorthQueryRuntimeFacadeFamily,
) -> WorthQueryRuntimeFamilySupportStatus {
    matrix
        .rows()
        .iter()
        .find(|row| row.facade_family() == Some(family))
        .unwrap_or_else(|| panic!("support matrix should include `{family}`"))
        .status()
}

fn unsupported_public_families() -> [WorthQueryRuntimeFacadeFamily; 10] {
    [
        WorthQueryRuntimeFacadeFamily::Computed,
        WorthQueryRuntimeFacadeFamily::SharedRead,
        WorthQueryRuntimeFacadeFamily::Replay,
        WorthQueryRuntimeFacadeFamily::Effect,
        WorthQueryRuntimeFacadeFamily::Intent,
        WorthQueryRuntimeFacadeFamily::Temporal,
        WorthQueryRuntimeFacadeFamily::AsyncResource,
        WorthQueryRuntimeFacadeFamily::MixedCauseDelivery,
        WorthQueryRuntimeFacadeFamily::StoreBackedExecution,
        WorthQueryRuntimeFacadeFamily::DurableArtifacts,
    ]
}
