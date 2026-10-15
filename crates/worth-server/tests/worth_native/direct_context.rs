use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_proof::TransitionOutcome;
use worth_query::facade::runtime::{
    WorthQueryInspection, WorthQueryRuntimeRemaskProjection, WorthQueryRuntimeRemaskReasonKind,
    WorthQueryRuntimeStateKind,
};
use worth_server::{
    WorthServerDirectInspectionOutcome, WorthServerDirectReadOutcome,
    WorthServerDirectRemaskDisposition, WorthServerDirectStateOutcome, WorthServerResponseInput,
    WorthServerSurfaceFamily, WorthServerTransportClass,
};

use {crate::shared_support_direct_context_runtime::RemaskWorkspaceProvider, crate::shared_support_query_handoff_fixture::{admit_read_posture, request_input, resolve_request_context, success}, crate::shared_support_worth_native_assertions::{
        admitted_named_read, direct_provenance_digest, response_provenance_digest,
        worth_native_session,
    }, crate::shared_support_worth_native_runtime::{
        build_server, build_server_with_workspace_provider, worth_native_session_input_builder,
    }};

#[test]
fn direct_context_preserves_branch_basis_and_provenance_across_equivalent_and_distinct_sessions() {
    let server = build_server(true);
    let main_session_a = worth_native_session(&server);
    let main_session_b = worth_native_session(&server);
    let branch_session = branch_session(&server, "branch-9");
    let main_declaration_a = admitted_named_read(&main_session_a, "users.profile");
    let main_declaration_b = admitted_named_read(&main_session_b, "users.profile");
    let branch_declaration = admitted_named_read(&branch_session, "users.profile");

    let read_a = direct_read_success(main_session_a.direct().read(&main_declaration_a));
    let read_b = direct_read_success(main_session_b.direct().read(&main_declaration_b));
    let read_branch = direct_read_success(branch_session.direct().read(&branch_declaration));

    assert_eq!(
        read_a.direct_context().branch_target().canonical_label(),
        "main"
    );
    assert_eq!(
        read_branch
            .direct_context()
            .branch_target()
            .canonical_label(),
        "branch:branch-9"
    );
    assert_eq!(
        read_a.direct_context().branch_digest(),
        read_b.direct_context().branch_digest()
    );
    assert_ne!(
        read_a.direct_context().branch_digest(),
        read_branch.direct_context().branch_digest()
    );
    assert_eq!(
        read_a.direct_context().basis_digest(),
        read_b.direct_context().basis_digest()
    );
    assert_ne!(
        read_a.direct_context().canonical_digest(),
        read_branch.direct_context().canonical_digest()
    );
    assert_eq!(
        read_a
            .direct_context()
            .workspace_target()
            .workspace_digest(),
        read_a.direct_context().workspace_digest()
    );
    assert_eq!(
        direct_provenance_digest(read_a.direct_context().provenance()),
        response_provenance_digest(read_a.response_envelope())
    );
    assert_eq!(
        response_provenance_digest(read_a.response_envelope()),
        response_provenance_digest(read_b.response_envelope())
    );
}

#[test]
fn direct_context_parity_preserves_branch_and_provenance_against_compatibility_flow() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");

    let direct = direct_read_success(session.direct().read(&declaration));
    let compat_handoff = compat_read_handoff(&server);
    let compat_response =
        server
            .responses()
            .shape_with_defaults(WorthServerResponseInput::query_handoff_success(
                compat_read_handoff(&server),
            ));

    assert_eq!(
        direct.direct_context().workspace_target().tenant_id(),
        compat_handoff
            .admission()
            .request_context()
            .workspace_target()
            .tenant_id()
    );
    assert_eq!(
        direct.direct_context().workspace_target().workspace_id(),
        compat_handoff
            .admission()
            .request_context()
            .workspace_target()
            .workspace_id()
    );
    assert_eq!(
        direct.direct_context().branch_target().canonical_label(),
        compat_handoff
            .admission()
            .request_context()
            .branch_target()
            .canonical_label()
    );
    assert_eq!(
        direct_provenance_digest(direct.direct_context().provenance()),
        response_provenance_digest(&compat_response)
    );
    assert_eq!(
        direct.support_posture().runtime_resume_support_posture(),
        compat_handoff
            .support_posture()
            .runtime_resume_support_posture()
    );
}

#[test]
fn direct_context_preserves_typed_remask_parity_across_state_and_inspection() {
    let server = build_server_with_workspace_provider(
        RemaskWorkspaceProvider::new(WorthQueryRuntimeRemaskProjection::remasked(
            WorthQueryRuntimeRemaskReasonKind::PolicyDrift,
            "policy:test",
            "tenant-truth:test",
            "tenant-schema:test",
            "relationship-proof:test",
            "schema-context:test",
        )),
        true,
    );
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");

    let state = direct_state_success(session.direct().state(&declaration));
    let inspection = direct_inspection_success(session.direct().inspect(&declaration));

    assert_eq!(
        state.runtime_state().kind(),
        WorthQueryRuntimeStateKind::Remasked
    );
    assert_eq!(
        state.direct_context().remask_posture().disposition(),
        WorthServerDirectRemaskDisposition::Remasked
    );
    assert_eq!(
        inspection.direct_context().remask_posture().disposition(),
        WorthServerDirectRemaskDisposition::Remasked
    );
    assert_eq!(
        state.direct_context().basis_digest(),
        Some(state.runtime_state().basis_for_reporting())
    );
    assert_eq!(
        state.direct_context().remask_posture().remask_digest(),
        inspection.direct_context().remask_posture().remask_digest()
    );
    assert_eq!(
        state
            .direct_context()
            .remask_posture()
            .artifact()
            .expect("remask artifact")
            .reason_kind(),
        "policy_drift"
    );
    match inspection.inspection_result().inspection() {
        WorthQueryInspection::LiveView(live) => {
            assert_eq!(
                inspection.direct_context().basis_digest(),
                Some(
                    live.basis_binding_identity()
                        .terminal_projection_for_reporting()
                )
            );
        }
        other => panic!("expected live inspection, got {other:?}"),
    }
}

#[test]
fn direct_context_richness_changes_provenance_detail_without_changing_branch_or_support_truth() {
    let standard_server = build_server(true);
    let forensic_server = build_server(true);
    let standard_session = worth_native_session(&standard_server);
    let forensic_session =
        session_with_diagnostics(&forensic_server, DiagnosticRichnessProfile::Forensic);
    let standard_declaration = admitted_named_read(&standard_session, "users.profile");
    let forensic_declaration = admitted_named_read(&forensic_session, "users.profile");

    let standard = direct_read_success(standard_session.direct().read(&standard_declaration));
    let forensic = direct_read_success(forensic_session.direct().read(&forensic_declaration));

    assert_eq!(
        standard.direct_context().branch_digest(),
        forensic.direct_context().branch_digest()
    );
    assert_eq!(
        standard.direct_context().basis_digest(),
        forensic.direct_context().basis_digest()
    );
    assert_eq!(
        standard.direct_context().support_posture_digest(),
        forensic.direct_context().support_posture_digest()
    );
    assert_ne!(
        standard.direct_context().provenance().provenance_digest(),
        forensic.direct_context().provenance().provenance_digest()
    );
    assert_eq!(
        direct_provenance_digest(standard.direct_context().provenance()),
        direct_provenance_digest(forensic.direct_context().provenance())
    );
    assert_eq!(
        standard.direct_context().provenance().diagnostics_profile(),
        DiagnosticRichnessProfile::Standard
    );
    assert_eq!(
        forensic.direct_context().provenance().diagnostics_profile(),
        DiagnosticRichnessProfile::Forensic
    );
}

fn branch_session(
    server: &worth_server::WorthServer,
    branch_id: &str,
) -> worth_server::WorthServerWorthNativeSession {
    match server.worth_native().session(
        worth_native_session_input_builder()
            .with_branch_id(branch_id)
            .build()
            .expect("branch session input should validate"),
    ) {
        TransitionOutcome::Success(session) => session,
        other => panic!("expected branch session, got {other:?}"),
    }
}

fn session_with_diagnostics(
    server: &worth_server::WorthServer,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> worth_server::WorthServerWorthNativeSession {
    match server.worth_native().session(
        worth_native_session_input_builder()
            .with_diagnostics_profile(diagnostics_profile)
            .build()
            .expect("diagnostics session input should validate"),
    ) {
        TransitionOutcome::Success(session) => session,
        other => panic!("expected diagnostics session, got {other:?}"),
    }
}

fn direct_read_success(
    outcome: WorthServerDirectReadOutcome,
) -> worth_server::WorthServerDirectRead {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct read, got {other:?}"),
    }
}

fn direct_state_success(
    outcome: WorthServerDirectStateOutcome,
) -> worth_server::WorthServerDirectState {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct state, got {other:?}"),
    }
}

fn direct_inspection_success(
    outcome: WorthServerDirectInspectionOutcome,
) -> worth_server::WorthServerDirectInspection {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct inspection, got {other:?}"),
    }
}

fn compat_read_handoff(
    server: &worth_server::WorthServer,
) -> worth_server::WorthServerQueryHandoff {
    success(
        server
            .query_handoff()
            .prepare(worth_server::WorthServerQueryHandoffInput::new(
                admit_read_posture(
                    server,
                    resolve_request_context(
                        server,
                        request_input(
                            WorthServerSurfaceFamily::CompatHttp,
                            WorthServerTransportClass::CompatHttp,
                        ),
                    ),
                ),
                worth_server::WorthServerQueryHandoffOperation::query_read("users.profile"),
            )),
    )
}
