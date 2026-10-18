#[allow(unused_imports)]
use crate::shared_support_test_bootstrap_policy as test_bootstrap_policy;
use worth_server::{
    request_context::DiagnosticRichnessProfile, WorthServer, WorthServerLoweredOperationPlan,
    WorthServerOperationPlannerInput, WorthServerOperationRequestInput,
    WorthServerQueryHandoffOperation, WorthServerRequestContextInput,
    WorthServerSchedulerCancellationDirective, WorthServerSchedulerCancellationPosture,
    WorthServerSchedulerCertificationSabotage, WorthServerSchedulerConflictDenialCode,
    WorthServerSchedulerFailurePosture, WorthServerSurfaceFamily, WorthServerTransportClass,
};

#[path = "support/operation_scheduler/runtime.rs"]
mod operation_scheduler_runtime;
#[allow(unused_imports)]
use crate::shared_support_query_handoff_fixture as query_handoff_fixture;

use operation_scheduler_runtime::{
    SchedulerWorkspaceProvider, SelectiveSharedReadWorkspaceProvider,
};
use query_handoff_fixture::{admit_read_posture, resolve_request_context, test_server};

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
            WorthServerSchedulerCertificationSabotage::ForbiddenGlobalLockAfterAdmission {
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

#[path = "operation_scheduler/scheduler_failures_are_isolated_by_declared_dependency.rs"]
mod scheduler_failures_are_isolated_by_declared_dependency;

#[test]
fn dependency_grouping_isolated_for_basis_and_footprint_near_misses() {
    let server = test_server(SelectiveSharedReadWorkspaceProvider, false);
    let executed = server
        .operation_scheduler()
        .schedule_shared_read_batch(vec![
            lower_shared_read_plan(
                &server,
                "workspace-shared-read-denied",
                WorthServerSurfaceFamily::WorthNative,
                WorthServerTransportClass::WorthNativeInProcess,
                WorthServerQueryHandoffOperation::direct_read("users.profile"),
                DiagnosticRichnessProfile::Standard,
            ),
            lower_shared_read_plan_with_basis(
                &server,
                "workspace-shared-read-denied",
                "basis-users-profile-other",
                WorthServerSurfaceFamily::CompatHttp,
                WorthServerTransportClass::CompatHttp,
                WorthServerQueryHandoffOperation::direct_state("users.profile"),
                DiagnosticRichnessProfile::Standard,
            ),
            lower_shared_read_plan_with_basis(
                &server,
                "workspace-43",
                "basis-users-profile",
                WorthServerSurfaceFamily::WorthNative,
                WorthServerTransportClass::WorthNativeInProcess,
                WorthServerQueryHandoffOperation::direct_inspection("users.profile"),
                DiagnosticRichnessProfile::Standard,
            ),
        ])
        .expect("shared-read plans should schedule")
        .execute();

    assert_eq!(executed.counters().isolated_failure_count(), 2);
    assert_eq!(executed.counters().dependent_failure_count(), 0);
    assert!(matches!(
        executed.outcomes()[0].failure_posture(),
        Some(WorthServerSchedulerFailurePosture::IsolatedRuntimeFailure { .. })
    ));
    assert!(matches!(
        executed.outcomes()[1].failure_posture(),
        Some(WorthServerSchedulerFailurePosture::IsolatedRuntimeFailure { .. })
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
            WorthServerSchedulerCancellationDirective::BeforeAdmission { slot_ordinal: 0 },
            WorthServerSchedulerCancellationDirective::AfterAdmissionBeforeExecution {
                slot_ordinal: 1,
            },
            WorthServerSchedulerCancellationDirective::DuringExecution { slot_ordinal: 2 },
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
        Some(WorthServerSchedulerCancellationPosture::BeforeAdmission)
    );
    assert_eq!(
        executed.outcomes()[1].cancellation_posture(),
        Some(WorthServerSchedulerCancellationPosture::AfterAdmissionBeforeExecution)
    );
    assert_eq!(
        executed.outcomes()[2].cancellation_posture(),
        Some(WorthServerSchedulerCancellationPosture::DuringExecution)
    );
}

#[test]
fn unsupported_shared_read_operations_are_denied_at_schedule_boundary() {
    let server = test_server(SchedulerWorkspaceProvider, false);
    for (surface_family, transport_class) in [
        (
            WorthServerSurfaceFamily::WorthNative,
            WorthServerTransportClass::WorthNativeInProcess,
        ),
        (
            WorthServerSurfaceFamily::CompatHttp,
            WorthServerTransportClass::CompatHttp,
        ),
    ] {
        let denial = server
            .operation_scheduler()
            .schedule_shared_read_batch([lower_shared_read_plan(
                &server,
                "workspace-42",
                surface_family,
                transport_class,
                WorthServerQueryHandoffOperation::direct_projection("users.profile"),
                DiagnosticRichnessProfile::Standard,
            )])
            .expect_err("unsupported projection operations must deny at scheduling");

        assert_eq!(
            denial.code(),
            WorthServerSchedulerConflictDenialCode::UnsupportedSharedReadOperation
        );
    }
}

fn assert_scheduler_equivalence(
    left: &worth_server::WorthServerExecutedOperationBatch,
    right: &worth_server::WorthServerExecutedOperationBatch,
) {
    assert_eq!(left.outcomes().len(), right.outcomes().len());
    for (left, right) in left.outcomes().iter().zip(right.outcomes()) {
        assert_eq!(
            left.response_envelope()
                .expect("equivalence lane should succeed")
                .canonical_digest(),
            right
                .response_envelope()
                .expect("equivalence lane should succeed")
                .canonical_digest()
        );
        assert_eq!(
            left.shared_read_basis_identity(),
            right.shared_read_basis_identity()
        );
        assert_eq!(left.execution_digest(), right.execution_digest());
        assert!(
            left.execution_digest().is_some(),
            "shared-read success should carry a real execution digest"
        );
        assert_eq!(
            left.plan_proof().receipt().expected_scheduler_lane(),
            right.plan_proof().receipt().expected_scheduler_lane()
        );
    }
}

fn build_shared_read_plans(server: &WorthServer) -> Vec<WorthServerLoweredOperationPlan> {
    vec![
        lower_shared_read_plan(
            server,
            "workspace-42",
            WorthServerSurfaceFamily::WorthNative,
            WorthServerTransportClass::WorthNativeInProcess,
            WorthServerQueryHandoffOperation::direct_read("users.profile"),
            DiagnosticRichnessProfile::Standard,
        ),
        lower_shared_read_plan(
            server,
            "workspace-42",
            WorthServerSurfaceFamily::CompatHttp,
            WorthServerTransportClass::CompatHttp,
            WorthServerQueryHandoffOperation::direct_state("users.profile"),
            DiagnosticRichnessProfile::Standard,
        ),
        lower_shared_read_plan(
            server,
            "workspace-42",
            WorthServerSurfaceFamily::WorthNative,
            WorthServerTransportClass::WorthNativeInProcess,
            WorthServerQueryHandoffOperation::direct_inspection("users.profile"),
            DiagnosticRichnessProfile::Forensic,
        ),
    ]
}

fn lower_shared_read_plan(
    server: &WorthServer,
    workspace_id: &str,
    surface_family: WorthServerSurfaceFamily,
    transport_class: WorthServerTransportClass,
    operation: WorthServerQueryHandoffOperation,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerLoweredOperationPlan {
    lower_shared_read_plan_with_basis(
        server,
        workspace_id,
        "basis-users-profile",
        surface_family,
        transport_class,
        operation,
        diagnostics_profile,
    )
}

fn lower_shared_read_plan_with_basis(
    server: &WorthServer,
    workspace_id: &str,
    basis_digest: &str,
    surface_family: WorthServerSurfaceFamily,
    transport_class: WorthServerTransportClass,
    operation: WorthServerQueryHandoffOperation,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerLoweredOperationPlan {
    let resolved = resolve_request_context(
        server,
        WorthServerRequestContextInput::builder()
            .with_surface_family(surface_family)
            .with_transport_class(transport_class)
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id(workspace_id)
            .with_diagnostics_profile(diagnostics_profile)
            .build()
            .expect("scheduler test request context should validate"),
    );
    let admission = admit_read_posture(server, resolved);
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            admission.authorization_proof().admission(),
            WorthServerOperationRequestInput::builder()
                .with_operation_family(worth_server::WorthServerOperationFamily::QueryDirectRead)
                .with_operation_name("users.profile")
                .with_basis_digest(basis_digest)
                .build(),
        )
        .expect("read operation request should admit");
    let operation_admission = server
        .operation_admissions()
        .admit_declared(
            admission.authorization_proof().admission(),
            &operation_request,
        )
        .expect("read operation admission should admit");
    server
        .operation_planner()
        .lower(WorthServerOperationPlannerInput::new(
            operation_admission,
            operation,
        ))
        .expect("shared-read plan should lower")
}
