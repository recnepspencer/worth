use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServerDirectDeliveryClass, WorthServerDirectDeliveryOutcome,
    WorthServerDirectDeliveryRequest, WorthServerDirectFreshnessMode,
    WorthServerDirectLeaseDeclarationOutcome, WorthServerQueryHandoffDenialCode,
    WorthServerQueryHandoffInput, WorthServerQueryHandoffOperation,
    WorthServerQueryRequestedResume, WorthServerSuccessKind, WorthServerSurfaceFamily,
    WorthServerTransportClass,
};

use {
    crate::shared_support_query_handoff_fixture::{
        admit_delivery_posture, request_input, resolve_request_context, success,
    },
    crate::shared_support_worth_native_assertions::{
        admitted_named_read, family_contract_digest, operator_evidence_record, worth_native_session,
    },
    crate::shared_support_worth_native_runtime::build_server,
};

mod direct_delivery_denials;

#[test]
fn direct_lease_declaration_preserves_admitted_declaration_identity() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");

    let lease = direct_lease_success(session.direct().declare_lease(&declaration));

    assert_eq!(lease.workspace_name(), declaration.workspace_name());
    assert_eq!(lease.principal_id(), "principal-7");
    assert_eq!(lease.tenant_id(), "tenant-a");
    assert_eq!(lease.workspace_id(), "workspace-42");
    assert_eq!(lease.branch_label(), "main");
    assert!(!lease.resume_basis_digest().is_empty());
    assert_eq!(lease.declaration_digest(), declaration.declaration_digest());
    assert_eq!(lease.declaration_binding_label(), "users.profile");
    assert_eq!(
        lease.declaration_canonical_label(),
        "named-read:users.profile"
    );
    assert_eq!(
        lease.support_digest(),
        declaration.support_snapshot().support_posture_digest()
    );
    assert_eq!(lease.lease_digest(), lease.canonical_digest());
}

#[test]
fn direct_lease_declarations_compare_equal_for_equivalent_inputs_and_unequal_for_distinct_views() {
    let server = build_server(true);
    let session_a = worth_native_session(&server);
    let session_b = worth_native_session(&server);
    let declaration_a = admitted_named_read(&session_a, "users.profile");
    let declaration_b = admitted_named_read(&session_b, "users.profile");
    let declaration_c = admitted_named_read(&session_b, "users.profile.saved");

    let lease_a = direct_lease_success(session_a.direct().declare_lease(&declaration_a));
    let lease_b = direct_lease_success(session_b.direct().declare_lease(&declaration_b));
    let lease_c = direct_lease_success(session_b.direct().declare_lease(&declaration_c));

    assert_eq!(lease_a.lease_digest(), lease_b.lease_digest());
    assert_eq!(lease_a.support_digest(), lease_b.support_digest());
    assert_ne!(lease_a.lease_digest(), lease_c.lease_digest());
}

#[test]
fn direct_lease_declaration_denies_missing_retained_artifact() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile.missing");

    let denial = direct_lease_denied(session.direct().declare_lease(&declaration));

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
fn direct_delivery_contract_preserves_query_handoff_posture_parity() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = direct_request(WorthServerQueryRequestedResume::none());

    let direct = direct_delivery_success(session.direct().negotiate_delivery(&lease, &request));
    let compat = success(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admit_delivery_posture(
                    &server,
                    resolve_request_context(
                        &server,
                        request_input(
                            WorthServerSurfaceFamily::CompatHttp,
                            WorthServerTransportClass::CompatHttp,
                        ),
                    ),
                    lease.resume_basis_digest(),
                ),
                WorthServerQueryHandoffOperation::downstream_delivery(
                    "users.profile",
                    WorthServerDirectFreshnessMode::LiveStrict,
                    WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                    WorthServerQueryRequestedResume::none(),
                ),
            )),
    );

    assert_eq!(
        family_contract_digest(direct.support_posture()),
        family_contract_digest(compat.support_posture())
    );
    assert_eq!(direct.workspace_name(), compat.workspace().name());
    assert_eq!(direct.handoff_digest(), compat.canonical_digest());
    assert_eq!(
        direct
            .downstream_delivery_contract()
            .contract_for_reporting(),
        compat
            .downstream_delivery_contract()
            .contract_for_reporting()
    );
    assert_eq!(
        direct.runtime_resume_support_posture(),
        compat
            .downstream_delivery_contract()
            .runtime_resume_support_posture()
    );
    assert_eq!(
        direct.durable_resume_support_posture(),
        compat
            .downstream_delivery_contract()
            .durable_resume_support_posture()
    );
    assert_eq!(
        direct
            .response_envelope()
            .success()
            .expect("success envelope")
            .payload()
            .kind(),
        WorthServerSuccessKind::DownstreamDelivery
    );
}

#[test]
fn direct_delivery_runtime_backed_resume_stays_explicit_and_stable() {
    let server = build_server(true);
    let session_a = worth_native_session(&server);
    let session_b = worth_native_session(&server);
    let declaration_a = admitted_named_read(&session_a, "users.profile");
    let declaration_b = admitted_named_read(&session_b, "users.profile");
    let lease_a = direct_lease_success(session_a.direct().declare_lease(&declaration_a));
    let lease_b = direct_lease_success(session_b.direct().declare_lease(&declaration_b));
    let request = WorthServerDirectDeliveryRequest::new(
        WorthServerDirectFreshnessMode::LiveStrict,
        WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        WorthServerQueryRequestedResume::runtime_backed(Some(lease_a.resume_basis_digest())),
    );

    let direct_a =
        direct_delivery_success(session_a.direct().negotiate_delivery(&lease_a, &request));
    let direct_b =
        direct_delivery_success(session_b.direct().negotiate_delivery(&lease_b, &request));

    assert!(direct_a.runtime_backed_resume_supported());
    assert!(!direct_a.durable_resume_supported());
    assert_eq!(direct_a.handoff_digest(), direct_b.handoff_digest());
    assert_eq!(direct_a.canonical_digest(), direct_b.canonical_digest());
    assert_eq!(
        direct_a.request().canonical_digest(),
        direct_b.request().canonical_digest()
    );
    assert_eq!(
        direct_a.runtime_resume_support_posture(),
        direct_b.runtime_resume_support_posture()
    );
}

#[test]
fn direct_delivery_request_identity_tracks_freshness_class_and_resume_axes() {
    let strict_authoritative = WorthServerDirectDeliveryRequest::new(
        WorthServerDirectFreshnessMode::LiveStrict,
        WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        WorthServerQueryRequestedResume::none(),
    );
    let coalesced_authoritative = WorthServerDirectDeliveryRequest::new(
        WorthServerDirectFreshnessMode::LiveCoalesced,
        WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        WorthServerQueryRequestedResume::none(),
    );
    let strict_replaceable = WorthServerDirectDeliveryRequest::new(
        WorthServerDirectFreshnessMode::LiveStrict,
        WorthServerDirectDeliveryClass::ReplaceableLatestState,
        WorthServerQueryRequestedResume::none(),
    );
    let strict_runtime_backed = WorthServerDirectDeliveryRequest::new(
        WorthServerDirectFreshnessMode::LiveStrict,
        WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        WorthServerQueryRequestedResume::runtime_backed(Some("basis:42")),
    );

    assert_eq!(
        strict_authoritative.request_digest(),
        strict_authoritative.canonical_digest()
    );
    assert_eq!(
        WorthServerDirectFreshnessMode::LiveStrict.as_str(),
        "live_strict"
    );
    assert_eq!(
        WorthServerDirectFreshnessMode::PresenceOnly.as_str(),
        "presence_only"
    );
    assert_ne!(
        strict_authoritative.request_digest(),
        coalesced_authoritative.request_digest()
    );
    assert_ne!(
        strict_authoritative.request_digest(),
        strict_replaceable.request_digest()
    );
    assert_ne!(
        strict_authoritative.request_digest(),
        strict_runtime_backed.request_digest()
    );
}

#[test]
fn direct_delivery_success_records_downstream_operator_evidence() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = direct_request(WorthServerQueryRequestedResume::none());

    let delivery = direct_delivery_success(session.direct().negotiate_delivery(&lease, &request));
    let evidence = operator_evidence_record(&server, delivery.response_envelope().clone());

    assert_eq!(
        evidence.classification(),
        &worth_server::WorthServerOperatorEvidenceClass::DownstreamDeliverySucceeded
    );
}

fn direct_request(
    requested_resume: WorthServerQueryRequestedResume,
) -> WorthServerDirectDeliveryRequest {
    WorthServerDirectDeliveryRequest::new(
        WorthServerDirectFreshnessMode::LiveStrict,
        WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        requested_resume,
    )
}

fn direct_delivery_success(
    outcome: WorthServerDirectDeliveryOutcome,
) -> worth_server::WorthServerDirectDeliveryContract {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct delivery contract, got {other:?}"),
    }
}

fn direct_lease_success(
    outcome: WorthServerDirectLeaseDeclarationOutcome,
) -> worth_server::WorthServerDirectLeaseDeclaration {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct lease declaration, got {other:?}"),
    }
}

fn direct_lease_denied(
    outcome: WorthServerDirectLeaseDeclarationOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct lease declaration, got {other:?}"),
    }
}

fn direct_delivery_denied(
    outcome: WorthServerDirectDeliveryOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct delivery contract, got {other:?}"),
    }
}
