use worth_proof::TransitionOutcome;
use worth_query::facade::runtime::{
    WorthQueryAuthorityLane, WorthQueryInspection, WorthQueryRuntimeFacadeFamily,
    WorthQueryRuntimeFamilySupport, WorthQueryRuntimeStateKind, WorthQueryRuntimeSupportProfile,
};
use worth_server::{
    WorthServerDirectInspectionOutcome, WorthServerDirectReadOutcome,
    WorthServerDirectStateOutcome, WorthServerQueryHandoffDenialCode, WorthServerQueryHandoffInput,
    WorthServerQueryHandoffOperation, WorthServerSuccessKind, WorthServerSurfaceFamily,
    WorthServerTransportClass,
};

use {
    crate::shared_support_query_handoff_fixture::{
        admit_read_posture, request_input, resolve_request_context, success,
    },
    crate::shared_support_worth_native_assertions::{
        admitted_named_read, family_contract_digest, worth_native_session,
    },
    crate::shared_support_worth_native_runtime::{
        build_server, build_server_with_profiled_workspace,
    },
};

#[test]
fn direct_read_preserves_read_family_parity_with_compatibility_handoff() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");

    let direct = direct_read_success(session.direct().read(&declaration));
    let compat = success(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
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
            )),
    );

    assert_eq!(
        family_contract_digest(direct.support_posture()),
        family_contract_digest(compat.support_posture())
    );
    assert_eq!(direct.workspace_name(), compat.workspace().name());
    assert_eq!(
        direct
            .response_envelope()
            .success()
            .expect("success envelope")
            .payload()
            .kind(),
        WorthServerSuccessKind::DirectRead
    );
    assert_eq!(direct.read_result().receipt().view_name(), "users.profile");
    assert_eq!(direct.read_result().receipt().row_count(), 1);
}

#[test]
fn direct_state_preserves_query_owned_async_and_temporal_posture() {
    let server = build_server_with_workspace_provider();
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");

    let direct = direct_state_success(session.direct().state(&declaration));

    assert_eq!(
        direct.runtime_state().kind(),
        WorthQueryRuntimeStateKind::Ready
    );
    assert_eq!(
        direct.runtime_state().authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert!(direct.async_result_state().is_none());
    assert!(direct.temporal_state().is_none());
    assert_eq!(
        direct
            .response_envelope()
            .success()
            .expect("success envelope")
            .payload()
            .kind(),
        WorthServerSuccessKind::DirectState
    );
}

#[test]
fn direct_inspection_preserves_inspection_evidence_and_response_kind() {
    let server = build_server_with_workspace_provider();
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");

    let direct = direct_inspection_success(session.direct().inspect(&declaration));

    match direct.inspection_result().inspection() {
        WorthQueryInspection::LiveView(live) => {
            assert_eq!(live.view_name(), "users.profile");
            assert_eq!(
                live.authority_lane(),
                WorthQueryAuthorityLane::AuthoritativeTruth
            );
        }
        other => panic!("expected live inspection result, got {other:?}"),
    }
    assert_eq!(
        direct
            .response_envelope()
            .success()
            .expect("success envelope")
            .payload()
            .kind(),
        WorthServerSuccessKind::DirectInspection
    );
}

#[test]
fn direct_retained_artifacts_preserve_canonical_digests_across_independent_sessions() {
    let server = build_server_with_workspace_provider();
    let session_a = worth_native_session(&server);
    let session_b = worth_native_session(&server);
    let declaration_a = admitted_named_read(&session_a, "users.profile");
    let declaration_b = admitted_named_read(&session_b, "users.profile");

    let read_a = direct_read_success(session_a.direct().read(&declaration_a));
    let read_b = direct_read_success(session_b.direct().read(&declaration_b));
    let state_a = direct_state_success(session_a.direct().state(&declaration_a));
    let state_b = direct_state_success(session_b.direct().state(&declaration_b));
    let inspection_a = direct_inspection_success(session_a.direct().inspect(&declaration_a));
    let inspection_b = direct_inspection_success(session_b.direct().inspect(&declaration_b));

    assert_eq!(read_a.handoff_digest(), read_b.handoff_digest());
    assert_eq!(
        read_a.read_result().receipt().result_digest(),
        read_b.read_result().receipt().result_digest()
    );
    assert_eq!(read_a.canonical_digest(), read_b.canonical_digest());

    assert_eq!(state_a.handoff_digest(), state_b.handoff_digest());
    assert_eq!(
        state_a.runtime_state().state_digest(),
        state_b.runtime_state().state_digest()
    );
    assert_eq!(state_a.canonical_digest(), state_b.canonical_digest());

    assert_eq!(inspection_a.handoff_digest(), inspection_b.handoff_digest());
    assert_eq!(
        inspection_a.inspection_result().receipt().result_digest(),
        inspection_b.inspection_result().receipt().result_digest()
    );
    assert_eq!(
        inspection_a.canonical_digest(),
        inspection_b.canonical_digest()
    );
}

#[test]
fn direct_state_denies_missing_retained_named_read_artifact() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile.missing");

    let denial = direct_state_denied(session.direct().state(&declaration));

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::RetainedQueryArtifactUnavailable
    );
    assert_eq!(
        denial.detail(),
        "live view `users.profile.missing` has no retained subscription installation"
    );
}

#[test]
fn direct_inspection_fails_closed_when_inspect_family_is_not_admitted() {
    let server = build_server_with_profiled_workspace(
        WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
            WorthQueryRuntimeFamilySupport::unsupported(
                WorthQueryRuntimeFacadeFamily::Inspect,
                "inspect is intentionally denied in this hostile test profile",
            ),
        ),
    );
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");

    let denial = direct_inspection_denied(session.direct().inspect(&declaration));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert!(denial
        .detail()
        .contains("query workspace does not admit `inspect` facade family"));
}

#[test]
fn direct_surface_denies_missing_retained_named_read_artifact() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile.missing");

    let denial = direct_read_denied(session.direct().read(&declaration));

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::RetainedQueryArtifactUnavailable
    );
    assert_eq!(
        denial.detail(),
        "live view `users.profile.missing` has no retained subscription installation"
    );
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

fn direct_state_denied(
    outcome: WorthServerDirectStateOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct state, got {other:?}"),
    }
}

fn direct_inspection_denied(
    outcome: WorthServerDirectInspectionOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct inspection, got {other:?}"),
    }
}

fn direct_read_denied(
    outcome: WorthServerDirectReadOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct read, got {other:?}"),
    }
}

fn build_server_with_workspace_provider() -> worth_server::WorthServer {
    build_server_with_profiled_workspace(WorthQueryRuntimeSupportProfile::scaffold_backend_profile())
}
