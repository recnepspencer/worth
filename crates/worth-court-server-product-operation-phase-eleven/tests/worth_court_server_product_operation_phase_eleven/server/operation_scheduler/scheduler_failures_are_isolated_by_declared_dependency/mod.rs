// scheduler failures are isolated by declared dependency.
use {
    crate::operation_scheduler::ordinary_context::lower_shared_read_plan,
    crate::operation_scheduler::ordinary_context::operation_scheduler_runtime::SelectiveSharedReadWorkspaceProvider,
    worth_testkit_server::shared_support_query_handoff_fixture::test_server,
};

#[test]
fn scheduler_failures_are_isolated_by_declared_dependency() {
    let server = test_server(SelectiveSharedReadWorkspaceProvider, false);
    let executed = server
        .operation_scheduler()
        .schedule_shared_read_batch(vec![
            lower_shared_read_plan(
                &server,
                "workspace-shared-read-denied",
                worth_server::WorthServerSurfaceFamily::WorthNative,
                worth_server::WorthServerTransportClass::WorthNativeInProcess,
                worth_server::WorthServerQueryHandoffOperation::direct_read("users.profile"),
                worth_server::request_context::DiagnosticRichnessProfile::Standard,
            ),
            lower_shared_read_plan(
                &server,
                "workspace-shared-read-denied",
                worth_server::WorthServerSurfaceFamily::CompatHttp,
                worth_server::WorthServerTransportClass::CompatHttp,
                worth_server::WorthServerQueryHandoffOperation::direct_state("users.profile"),
                worth_server::request_context::DiagnosticRichnessProfile::Standard,
            ),
            lower_shared_read_plan(
                &server,
                "workspace-42",
                worth_server::WorthServerSurfaceFamily::WorthNative,
                worth_server::WorthServerTransportClass::WorthNativeInProcess,
                worth_server::WorthServerQueryHandoffOperation::direct_inspection("users.profile"),
                worth_server::request_context::DiagnosticRichnessProfile::Standard,
            ),
        ])
        .expect("shared-read plans should schedule")
        .execute();

    assert_eq!(executed.counters().isolated_failure_count(), 1);
    assert_eq!(executed.counters().dependent_failure_count(), 1);
    assert!(executed.outcomes()[2].response_envelope().is_some());
    match executed.outcomes()[0]
        .failure_posture()
        .expect("first slot should fail in isolation")
    {
        worth_server::WorthServerSchedulerFailurePosture::IsolatedRuntimeFailure {
            runtime_failure,
        } => {
            assert!(runtime_failure.detail().contains("shared-read"));
        }
        other => panic!("expected isolated runtime failure, got {other:?}"),
    }
    match executed.outcomes()[1]
        .failure_posture()
        .expect("second slot should fail as dependent")
    {
        worth_server::WorthServerSchedulerFailurePosture::DependentSharedBasisFailure {
            shared_basis_key,
            failed_slot_ordinal,
        } => {
            assert!(shared_basis_key.contains("basis-users-profile"));
            assert_eq!(*failed_slot_ordinal, 0);
        }
        other => panic!("expected dependent shared-basis failure, got {other:?}"),
    }
}
