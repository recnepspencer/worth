#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};
pub(crate) use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::WorthServer, worth_server::WorthServerLoweredOperationPlan, worth_server::WorthServerOperationPlannerInput, worth_server::WorthServerOperationRequestInput, worth_server::WorthServerQueryHandoffOperation, worth_server::WorthServerRequestContextInput, worth_server::WorthServerSchedulerCancellationDirective, worth_server::WorthServerSchedulerCancellationPosture, worth_server::WorthServerSchedulerCertificationSabotage, worth_server::WorthServerSchedulerConflictDenialCode, worth_server::WorthServerSchedulerFailurePosture, worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass};


#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_query_handoff_fixture as query_handoff_fixture};

pub(crate) use {crate::operation_scheduler::ordinary_context::operation_scheduler_runtime::SchedulerWorkspaceProvider, crate::operation_scheduler::ordinary_context::operation_scheduler_runtime::SelectiveSharedReadWorkspaceProvider};
pub(crate) use {worth_testkit_server::shared_support_query_handoff_fixture::admit_read_posture, worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context, worth_testkit_server::shared_support_query_handoff_fixture::test_server};

pub(crate) fn assert_scheduler_equivalence(
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

pub(crate) fn build_shared_read_plans(server: &WorthServer) -> Vec<WorthServerLoweredOperationPlan> {
    vec![
        lower_shared_read_plan(
            server,
            "workspace-42",
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerTransportClass::WorthNativeInProcess,
            worth_server::WorthServerQueryHandoffOperation::direct_read("users.profile"),
            worth_server::request_context::DiagnosticRichnessProfile::Standard,
        ),
        lower_shared_read_plan(
            server,
            "workspace-42",
            worth_server::WorthServerSurfaceFamily::CompatHttp,
            worth_server::WorthServerTransportClass::CompatHttp,
            worth_server::WorthServerQueryHandoffOperation::direct_state("users.profile"),
            worth_server::request_context::DiagnosticRichnessProfile::Standard,
        ),
        lower_shared_read_plan(
            server,
            "workspace-42",
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerTransportClass::WorthNativeInProcess,
            worth_server::WorthServerQueryHandoffOperation::direct_inspection("users.profile"),
            worth_server::request_context::DiagnosticRichnessProfile::Forensic,
        ),
    ]
}

pub(crate) fn lower_shared_read_plan(
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

pub(crate) fn lower_shared_read_plan_with_basis(
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
        worth_server::WorthServerRequestContextInput::builder()
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
            worth_server::WorthServerOperationRequestInput::builder()
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
        .lower(worth_server::WorthServerOperationPlannerInput::new(
            operation_admission,
            operation,
        ))
        .expect("shared-read plan should lower")
}