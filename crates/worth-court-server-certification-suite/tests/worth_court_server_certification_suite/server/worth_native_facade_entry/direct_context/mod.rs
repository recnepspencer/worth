#[allow(unused_imports)]
use crate::worth_native_facade_entry::ordinary_context::direct_context::*;
use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_proof::TransitionOutcome;
use {
    worth_query::facade::runtime::WorthQueryInspection,
    worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection,
    worth_query::facade::runtime::WorthQueryRuntimeRemaskReasonKind,
    worth_query::facade::runtime::WorthQueryRuntimeStateKind,
};
use {
    worth_server::WorthServerDirectInspectionOutcome, worth_server::WorthServerDirectReadOutcome,
    worth_server::WorthServerDirectRemaskDisposition, worth_server::WorthServerDirectStateOutcome,
    worth_server::WorthServerResponseInput, worth_server::WorthServerSurfaceFamily,
    worth_server::WorthServerTransportClass,
};

use {
    worth_testkit_server::shared_support_direct_context_runtime::RemaskWorkspaceProvider,
    worth_testkit_server::shared_support_query_handoff_fixture::admit_read_posture,
    worth_testkit_server::shared_support_query_handoff_fixture::request_input,
    worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context,
    worth_testkit_server::shared_support_query_handoff_fixture::success,
    worth_testkit_server::shared_support_worth_native_assertions::admitted_named_read,
    worth_testkit_server::shared_support_worth_native_assertions::direct_provenance_digest,
    worth_testkit_server::shared_support_worth_native_assertions::response_provenance_digest,
    worth_testkit_server::shared_support_worth_native_assertions::worth_native_session,
    worth_testkit_server::shared_support_worth_native_runtime::build_server,
    worth_testkit_server::shared_support_worth_native_runtime::build_server_with_workspace_provider,
    worth_testkit_server::shared_support_worth_native_runtime::worth_native_session_input_builder,
};

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
    let compat_response = server.responses().shape_with_defaults(
        worth_server::WorthServerResponseInput::query_handoff_success(compat_read_handoff(&server)),
    );

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
        worth_testkit_server::shared_support_direct_context_runtime::RemaskWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection::remasked(
                worth_query::facade::runtime::WorthQueryRuntimeRemaskReasonKind::PolicyDrift,
                "policy:test",
                "tenant-truth:test",
                "tenant-schema:test",
                "relationship-proof:test",
                "schema-context:test",
            ),
        ),
        true,
    );
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");

    let state = direct_state_success(session.direct().state(&declaration));
    let inspection = direct_inspection_success(session.direct().inspect(&declaration));

    assert_eq!(
        state.runtime_state().kind(),
        worth_query::facade::runtime::WorthQueryRuntimeStateKind::Remasked
    );
    assert_eq!(
        state.direct_context().remask_posture().disposition(),
        worth_server::WorthServerDirectRemaskDisposition::Remasked
    );
    assert_eq!(
        inspection.direct_context().remask_posture().disposition(),
        worth_server::WorthServerDirectRemaskDisposition::Remasked
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
        worth_query::facade::runtime::WorthQueryInspection::LiveView(live) => {
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
    let forensic_session = session_with_diagnostics(
        &forensic_server,
        worth_foundational::facade::DiagnosticRichnessProfile::Forensic,
    );
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
        worth_foundational::facade::DiagnosticRichnessProfile::Standard
    );
    assert_eq!(
        forensic.direct_context().provenance().diagnostics_profile(),
        worth_foundational::facade::DiagnosticRichnessProfile::Forensic
    );
}
