#[path = "support/test_bootstrap_policy.rs"]
mod test_bootstrap_policy;

use worth_query::facade::runtime::{
    WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupport, WorthQueryRuntimeSupportProfile,
};
use worth_server::{
    request_context::DiagnosticRichnessProfile, WorthServerDirectDeliveryClass,
    WorthServerDirectFreshnessMode, WorthServerOperationPlanDenialCode,
    WorthServerOperationPlannerInput, WorthServerQueryHandoffOperation,
    WorthServerQueryRequestedResume, WorthServerRequestContextInput, WorthServerSurfaceFamily,
    WorthServerTransportClass,
};

#[path = "support/compat_http/phase_two_runtime.rs"]
mod compat_http_phase_two_runtime;
#[path = "support/query_handoff/fixture.rs"]
mod query_handoff_fixture;
#[path = "support/query_handoff/runtime.rs"]
mod query_handoff_runtime;
#[path = "support/worth_native/assertions.rs"]
mod worth_native_assertions;
#[path = "support/worth_native/runtime.rs"]
mod worth_native_runtime;

use compat_http_phase_two_runtime::{
    build_phase_two_server, compat_execution_input, worth_native_named_read,
};
use query_handoff_fixture::{
    admit_delivery_posture, admit_read_posture, request_input, resolve_request_context, test_server,
};
use query_handoff_runtime::ProfiledTestWorkspaceProvider;
use worth_native_runtime::server_with_request_context_default;

#[test]
fn equivalent_operation_requests_lower_to_identical_plans() {
    let server = test_server(query_handoff_runtime::TestWorkspaceProvider, false);
    let worth_native = server
        .operation_planner()
        .lower(WorthServerOperationPlannerInput::new(
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
        ))
        .expect("Worth-native plan should lower");
    let compat_http = server
        .operation_planner()
        .lower(WorthServerOperationPlannerInput::new(
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
        ))
        .expect("compat-http plan should lower");

    assert_eq!(
        worth_native.receipt().plan_identity(),
        compat_http.receipt().plan_identity()
    );
    assert_eq!(
        worth_native.canonical_digest(),
        compat_http.canonical_digest()
    );
    assert_eq!(
        worth_native.counters().strategy_choice(),
        compat_http.counters().strategy_choice()
    );
}

#[test]
fn plan_identity_excludes_diagnostics_richness_but_includes_support_and_strategy() {
    let standard_server = test_server(query_handoff_runtime::TestWorkspaceProvider, false);
    let forensic_server = test_server(query_handoff_runtime::TestWorkspaceProvider, false);
    let standard = standard_server
        .operation_planner()
        .lower(WorthServerOperationPlannerInput::new(
            admit_read_posture(
                &standard_server,
                resolve_request_context(
                    &standard_server,
                    WorthServerRequestContextInput::builder()
                        .with_surface_family(WorthServerSurfaceFamily::WorthNative)
                        .with_transport_class(WorthServerTransportClass::WorthNativeInProcess)
                        .with_authenticated_principal_id("principal-7")
                        .with_tenant_id("tenant-a")
                        .with_workspace_id("workspace-42")
                        .with_diagnostics_profile(DiagnosticRichnessProfile::Standard)
                        .build()
                        .expect("standard request context should validate"),
                ),
            ),
            WorthServerQueryHandoffOperation::query_read("users.profile"),
        ))
        .expect("standard plan should lower");
    let forensic = forensic_server
        .operation_planner()
        .lower(WorthServerOperationPlannerInput::new(
            admit_read_posture(
                &forensic_server,
                resolve_request_context(
                    &forensic_server,
                    WorthServerRequestContextInput::builder()
                        .with_surface_family(WorthServerSurfaceFamily::WorthNative)
                        .with_transport_class(WorthServerTransportClass::WorthNativeInProcess)
                        .with_authenticated_principal_id("principal-7")
                        .with_tenant_id("tenant-a")
                        .with_workspace_id("workspace-42")
                        .with_diagnostics_profile(DiagnosticRichnessProfile::Forensic)
                        .build()
                        .expect("forensic request context should validate"),
                ),
            ),
            WorthServerQueryHandoffOperation::query_read("users.profile"),
        ))
        .expect("forensic plan should lower");
    let unsupported_server = test_server(
        ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                WorthQueryRuntimeFamilySupport::unsupported(
                    WorthQueryRuntimeFacadeFamily::Read,
                    "read is intentionally denied in this hostile test profile",
                ),
            ),
        ),
        false,
    );
    let unsupported =
        unsupported_server
            .operation_planner()
            .lower(WorthServerOperationPlannerInput::new(
                admit_read_posture(
                    &unsupported_server,
                    resolve_request_context(
                        &unsupported_server,
                        request_input(
                            WorthServerSurfaceFamily::WorthNative,
                            WorthServerTransportClass::WorthNativeInProcess,
                        ),
                    ),
                ),
                WorthServerQueryHandoffOperation::query_read("users.profile"),
            ));

    assert_eq!(
        standard.receipt().plan_identity(),
        forensic.receipt().plan_identity()
    );
    assert_ne!(
        standard.receipt().evidence_identity(),
        forensic.receipt().evidence_identity()
    );
    assert!(unsupported.is_err());
}

#[test]
fn plan_counters_explain_strategy_selection() {
    let server = test_server(query_handoff_runtime::TestWorkspaceProvider, false);
    let plan = server
        .operation_planner()
        .lower(WorthServerOperationPlannerInput::new(
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
        ))
        .expect("plan should lower");

    assert_eq!(
        plan.counters().strategy_choice().as_str(),
        "shared-read-execution"
    );
    assert_eq!(plan.counters().footprint_breadth(), 2);
    assert_eq!(plan.counters().support_rows_consulted(), 2);
    assert_eq!(plan.receipt().expected_scheduler_lane(), "shared-read");
}

#[test]
fn mutation_plan_counters_reflect_branch_scoped_authority() {
    let server = test_server(query_handoff_runtime::TestWorkspaceProvider, true);
    let plan = server
        .operation_planner()
        .lower(WorthServerOperationPlannerInput::new(
            admit_delivery_posture(
                &server,
                resolve_request_context(
                    &server,
                    request_input(
                        WorthServerSurfaceFamily::WorthNative,
                        WorthServerTransportClass::WorthNativeInProcess,
                    ),
                ),
                "basis-users-profile",
            ),
            WorthServerQueryHandoffOperation::downstream_delivery(
                "users.profile",
                WorthServerDirectFreshnessMode::LiveStrict,
                WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                WorthServerQueryRequestedResume::none(),
            ),
        ))
        .expect("delivery plan should lower");

    assert_eq!(
        plan.counters().strategy_choice().as_str(),
        "lease-coordination"
    );
    assert_eq!(plan.counters().footprint_breadth(), 4);
    assert_eq!(
        plan.receipt().expected_scheduler_lane(),
        "serialize-deterministically"
    );
}

#[test]
fn unsupported_query_support_denies_at_planning() {
    let server = test_server(
        ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                WorthQueryRuntimeFamilySupport::unsupported(
                    WorthQueryRuntimeFacadeFamily::Read,
                    "read is intentionally denied in this hostile test profile",
                ),
            ),
        ),
        false,
    );

    let denial = server
        .operation_planner()
        .lower(WorthServerOperationPlannerInput::new(
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
        ))
        .expect_err("unsupported read should deny during planning");

    assert_eq!(
        denial.code(),
        WorthServerOperationPlanDenialCode::SupportDenied
    );
    assert!(denial
        .detail()
        .contains("query workspace does not admit `read` facade family"));
}

#[test]
fn real_direct_and_compat_reads_expose_identical_lowered_plan_proofs() {
    let server = build_phase_two_server();
    let (session, declaration) = worth_native_named_read(&server, "users.profile");

    let direct = direct_read_success(session.direct().read(&declaration));
    let compat = compat_read_success(
        server
            .compat_http()
            .read(compat_execution_input(&server, "users.profile")),
    );

    assert_ne!(
        direct.plan_proof().receipt().plan_identity(),
        compat.plan_proof().receipt().plan_identity()
    );
    assert_eq!(
        direct.plan_proof().receipt().evidence_identity(),
        compat.plan_proof().receipt().evidence_identity()
    );
    assert_eq!(
        direct.plan_proof().receipt().expected_scheduler_lane(),
        compat.plan_proof().receipt().expected_scheduler_lane()
    );
    assert_eq!(
        direct.plan_proof().counters().strategy_choice(),
        compat.plan_proof().counters().strategy_choice()
    );
    assert_eq!(
        direct.plan_proof().counters().footprint_breadth(),
        compat.plan_proof().counters().footprint_breadth()
    );
    assert_eq!(
        direct.plan_proof().counters().support_rows_consulted(),
        compat.plan_proof().counters().support_rows_consulted()
    );
}

#[test]
fn real_direct_reads_keep_plan_identity_stable_across_diagnostics_profiles() {
    let standard_server = server_with_request_context_default(DiagnosticRichnessProfile::Standard);
    let forensic_server = server_with_request_context_default(DiagnosticRichnessProfile::Forensic);
    let (standard_session, standard_declaration) =
        worth_native_named_read(&standard_server, "users.profile");
    let (forensic_session, forensic_declaration) =
        worth_native_named_read(&forensic_server, "users.profile");

    let standard = direct_read_success(standard_session.direct().read(&standard_declaration));
    let forensic = direct_read_success(forensic_session.direct().read(&forensic_declaration));

    assert_eq!(
        standard.plan_proof().receipt().plan_identity(),
        forensic.plan_proof().receipt().plan_identity()
    );
    assert_ne!(
        standard.plan_proof().receipt().evidence_identity(),
        forensic.plan_proof().receipt().evidence_identity()
    );
    assert_eq!(
        standard.plan_proof().counters().strategy_choice(),
        forensic.plan_proof().counters().strategy_choice()
    );
    assert_eq!(
        standard.plan_proof().counters().footprint_breadth(),
        forensic.plan_proof().counters().footprint_breadth()
    );
    assert_eq!(
        standard.plan_proof().counters().support_rows_consulted(),
        forensic.plan_proof().counters().support_rows_consulted()
    );
}

fn direct_read_success(
    outcome: worth_server::WorthServerDirectReadOutcome,
) -> worth_server::WorthServerDirectRead {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected direct read success, got {other:?}"),
    }
}

fn compat_read_success(
    outcome: worth_server::WorthServerCompatibilityExecutionOutcome<
        worth_server::WorthServerCompatibilityRead,
    >,
) -> worth_server::WorthServerCompatibilityRead {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility read success, got {other:?}"),
    }
}
