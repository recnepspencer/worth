#[allow(unused_imports)] use crate::worth_native_facade_entry::ordinary_context::direct_delivery::*;
use {worth_proof::TransitionOutcome};
use {worth_server::WorthServerDirectDeliveryClass, worth_server::WorthServerDirectDeliveryOutcome, worth_server::WorthServerDirectDeliveryRequest, worth_server::WorthServerDirectFreshnessMode, worth_server::WorthServerDirectLeaseDeclarationOutcome, worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerQueryHandoffInput, worth_server::WorthServerQueryHandoffOperation, worth_server::WorthServerQueryRequestedResume, worth_server::WorthServerSuccessKind, worth_server::WorthServerSurfaceFamily, worth_server::WorthServerTransportClass};

use {worth_testkit_server::shared_support_query_handoff_fixture::admit_delivery_posture, worth_testkit_server::shared_support_query_handoff_fixture::request_input, worth_testkit_server::shared_support_query_handoff_fixture::resolve_request_context, worth_testkit_server::shared_support_query_handoff_fixture::success, worth_testkit_server::shared_support_worth_native_assertions::admitted_named_read, worth_testkit_server::shared_support_worth_native_assertions::family_contract_digest, worth_testkit_server::shared_support_worth_native_assertions::operator_evidence_record, worth_testkit_server::shared_support_worth_native_assertions::worth_native_session, worth_testkit_server::shared_support_worth_native_runtime::build_server};


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
        worth_server::WorthServerQueryHandoffDenialCode::RetainedQueryArtifactUnavailable
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
    let request = direct_request(worth_server::WorthServerQueryRequestedResume::none());

    let direct = direct_delivery_success(session.direct().negotiate_delivery(&lease, &request));
    let compat = success(
        server
            .query_handoff()
            .prepare(worth_server::WorthServerQueryHandoffInput::new(
                admit_delivery_posture(
                    &server,
                    resolve_request_context(
                        &server,
                        request_input(
                            worth_server::WorthServerSurfaceFamily::CompatHttp,
                            worth_server::WorthServerTransportClass::CompatHttp,
                        ),
                    ),
                    lease.resume_basis_digest(),
                ),
                worth_server::WorthServerQueryHandoffOperation::downstream_delivery(
                    "users.profile",
                    worth_server::WorthServerDirectFreshnessMode::LiveStrict,
                    worth_server::WorthServerDirectDeliveryClass::AuthoritativeOrdered,
                    worth_server::WorthServerQueryRequestedResume::none(),
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
        worth_server::WorthServerSuccessKind::DownstreamDelivery
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
    let request = worth_server::WorthServerDirectDeliveryRequest::new(
        worth_server::WorthServerDirectFreshnessMode::LiveStrict,
        worth_server::WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        worth_server::WorthServerQueryRequestedResume::runtime_backed(Some(lease_a.resume_basis_digest())),
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
    let strict_authoritative = worth_server::WorthServerDirectDeliveryRequest::new(
        worth_server::WorthServerDirectFreshnessMode::LiveStrict,
        worth_server::WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        worth_server::WorthServerQueryRequestedResume::none(),
    );
    let coalesced_authoritative = worth_server::WorthServerDirectDeliveryRequest::new(
        worth_server::WorthServerDirectFreshnessMode::LiveCoalesced,
        worth_server::WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        worth_server::WorthServerQueryRequestedResume::none(),
    );
    let strict_replaceable = worth_server::WorthServerDirectDeliveryRequest::new(
        worth_server::WorthServerDirectFreshnessMode::LiveStrict,
        worth_server::WorthServerDirectDeliveryClass::ReplaceableLatestState,
        worth_server::WorthServerQueryRequestedResume::none(),
    );
    let strict_runtime_backed = worth_server::WorthServerDirectDeliveryRequest::new(
        worth_server::WorthServerDirectFreshnessMode::LiveStrict,
        worth_server::WorthServerDirectDeliveryClass::AuthoritativeOrdered,
        worth_server::WorthServerQueryRequestedResume::runtime_backed(Some("basis:42")),
    );

    assert_eq!(
        strict_authoritative.request_digest(),
        strict_authoritative.canonical_digest()
    );
    assert_eq!(
        worth_server::WorthServerDirectFreshnessMode::LiveStrict.as_str(),
        "live_strict"
    );
    assert_eq!(
        worth_server::WorthServerDirectFreshnessMode::PresenceOnly.as_str(),
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
    let request = direct_request(worth_server::WorthServerQueryRequestedResume::none());

    let delivery = direct_delivery_success(session.direct().negotiate_delivery(&lease, &request));
    let evidence = operator_evidence_record(&server, delivery.response_envelope().clone());

    assert_eq!(
        evidence.classification(),
        &worth_server::WorthServerOperatorEvidenceClass::DownstreamDeliverySucceeded
    );
}
#[path = "direct_delivery_denials/mod.rs"]
pub(crate) mod direct_delivery_denials;
