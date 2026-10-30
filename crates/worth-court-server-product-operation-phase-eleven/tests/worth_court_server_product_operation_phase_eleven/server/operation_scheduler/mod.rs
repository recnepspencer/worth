#[allow(unused_imports)]
use crate::operation_scheduler::ordinary_context::*;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

#[allow(unused_imports)]
use worth_testkit_server::shared_support_query_handoff_fixture as query_handoff_fixture;

use worth_testkit_server::shared_support_query_handoff_fixture::test_server;
use {
    crate::operation_scheduler::ordinary_context::operation_scheduler_runtime::SchedulerWorkspaceProvider,
    crate::operation_scheduler::ordinary_context::operation_scheduler_runtime::SelectiveSharedReadWorkspaceProvider,
};

#[test]
fn concurrent_shared_read_scheduler_matches_serialized_control() {
    let server = test_server(SchedulerWorkspaceProvider, false);

    let concurrent = server
        .operation_scheduler()
        .schedule_shared_read_batch(build_shared_read_plans(&server))
        .expect("shared-read plans should schedule")
        .execute();
    let serialized = server
        .operation_scheduler()
        .schedule_shared_read_batch(build_shared_read_plans(&server))
        .expect("shared-read plans should schedule")
        .execute_serialized_control();
    let repeated = server
        .operation_scheduler()
        .schedule_shared_read_batch(build_shared_read_plans(&server))
        .expect("shared-read plans should schedule")
        .execute();

    assert_scheduler_equivalence(&concurrent, &serialized);
    assert_scheduler_equivalence(&concurrent, &repeated);
}

#[test]
fn shared_read_hot_path_reports_exact_zero_global_lock_acquisitions() {
    let server = test_server(SchedulerWorkspaceProvider, false);
    let executed = server
        .operation_scheduler()
        .schedule_shared_read_batch(build_shared_read_plans(&server))
        .expect("shared-read plans should schedule")
        .execute();

    assert_eq!(
        executed
            .counters()
            .forbidden_global_lock_acquisition_count(),
        0
    );
    assert_eq!(executed.counters().planned_batch_width(), 3);
    assert_eq!(executed.counters().completed_read_slot_count(), 3);
    for outcome in executed.outcomes() {
        assert_eq!(
            outcome
                .scheduler_counters()
                .forbidden_global_lock_acquisition_count(),
            0
        );
    }
}

#[test]
fn shared_read_lock_accounting_sabotage_reopens_exact_zero_posture() {
    let server = test_server(SchedulerWorkspaceProvider, false);
    let executed = server
        .operation_scheduler()
        .schedule_shared_read_batch(build_shared_read_plans(&server))
        .expect("shared-read plans should schedule")
        .execute_with_certification_sabotage([
            worth_server::WorthServerSchedulerCertificationSabotage::ForbiddenGlobalLockAfterAdmission {
                slot_ordinal: 1,
            },
        ]);

    assert_eq!(executed.counters().planned_batch_width(), 3);
    assert_eq!(executed.counters().completed_read_slot_count(), 3);
    assert_eq!(
        executed
            .counters()
            .forbidden_global_lock_acquisition_count(),
        1
    );
}

#[test]
fn dependency_grouping_isolated_for_basis_and_footprint_near_misses() {
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
            lower_shared_read_plan_with_basis(
                &server,
                "workspace-shared-read-denied",
                "basis-users-profile-other",
                worth_server::WorthServerSurfaceFamily::CompatHttp,
                worth_server::WorthServerTransportClass::CompatHttp,
                worth_server::WorthServerQueryHandoffOperation::direct_state("users.profile"),
                worth_server::request_context::DiagnosticRichnessProfile::Standard,
            ),
            lower_shared_read_plan_with_basis(
                &server,
                "workspace-43",
                "basis-users-profile",
                worth_server::WorthServerSurfaceFamily::WorthNative,
                worth_server::WorthServerTransportClass::WorthNativeInProcess,
                worth_server::WorthServerQueryHandoffOperation::direct_inspection("users.profile"),
                worth_server::request_context::DiagnosticRichnessProfile::Standard,
            ),
        ])
        .expect("shared-read plans should schedule")
        .execute();

    assert_eq!(executed.counters().isolated_failure_count(), 2);
    assert_eq!(executed.counters().dependent_failure_count(), 0);
    assert!(matches!(
        executed.outcomes()[0].failure_posture(),
        Some(worth_server::WorthServerSchedulerFailurePosture::IsolatedRuntimeFailure { .. })
    ));
    assert!(matches!(
        executed.outcomes()[1].failure_posture(),
        Some(worth_server::WorthServerSchedulerFailurePosture::IsolatedRuntimeFailure { .. })
    ));
    assert!(executed.outcomes()[2].response_envelope().is_some());
}

#[test]
fn scheduler_cancellation_records_only_after_admission() {
    let server = test_server(SchedulerWorkspaceProvider, false);
    let executed = server
        .operation_scheduler()
        .schedule_shared_read_batch(build_shared_read_plans(&server))
        .expect("shared-read plans should schedule")
        .execute_with_cancellation([
            worth_server::WorthServerSchedulerCancellationDirective::BeforeAdmission { slot_ordinal: 0 },
            worth_server::WorthServerSchedulerCancellationDirective::AfterAdmissionBeforeExecution {
                slot_ordinal: 1,
            },
            worth_server::WorthServerSchedulerCancellationDirective::DuringExecution { slot_ordinal: 2 },
        ]);

    assert_eq!(executed.counters().cancelled_before_admission_count(), 1);
    assert_eq!(
        executed
            .counters()
            .cancelled_after_admission_before_execution_count(),
        1
    );
    assert_eq!(executed.counters().cancelled_during_execution_count(), 1);
    assert_eq!(executed.counters().admitted_read_slot_count(), 2);
    assert_eq!(
        executed.outcomes()[0].cancellation_posture(),
        Some(worth_server::WorthServerSchedulerCancellationPosture::BeforeAdmission)
    );
    assert_eq!(
        executed.outcomes()[1].cancellation_posture(),
        Some(worth_server::WorthServerSchedulerCancellationPosture::AfterAdmissionBeforeExecution)
    );
    assert_eq!(
        executed.outcomes()[2].cancellation_posture(),
        Some(worth_server::WorthServerSchedulerCancellationPosture::DuringExecution)
    );
}

#[test]
fn unsupported_shared_read_operations_are_denied_at_schedule_boundary() {
    let server = test_server(SchedulerWorkspaceProvider, false);
    for (surface_family, transport_class) in [
        (
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerTransportClass::WorthNativeInProcess,
        ),
        (
            worth_server::WorthServerSurfaceFamily::CompatHttp,
            worth_server::WorthServerTransportClass::CompatHttp,
        ),
    ] {
        let denial = server
            .operation_scheduler()
            .schedule_shared_read_batch([lower_shared_read_plan(
                &server,
                "workspace-42",
                surface_family,
                transport_class,
                worth_server::WorthServerQueryHandoffOperation::direct_projection("users.profile"),
                worth_server::request_context::DiagnosticRichnessProfile::Standard,
            )])
            .expect_err("unsupported projection operations must deny at scheduling");

        assert_eq!(
            denial.code(),
            worth_server::WorthServerSchedulerConflictDenialCode::UnsupportedSharedReadOperation
        );
    }
}
#[path = "scheduler_failures_are_isolated_by_declared_dependency/mod.rs"]
pub(crate) mod scheduler_failures_are_isolated_by_declared_dependency;
