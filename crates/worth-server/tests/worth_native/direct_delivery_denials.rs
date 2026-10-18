use worth_proof::TransitionOutcome;
use worth_query::facade::runtime::{
    WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupport, WorthQueryRuntimeSupportProfile,
};
use worth_server::{WorthServerQueryHandoffDenialCode, WorthServerQueryRequestedResume};

use super::{direct_delivery_denied, direct_lease_success, direct_request};
use {
    crate::shared_support_query_handoff_runtime::TestWorkspaceProvider,
    crate::shared_support_worth_native_assertions::{admitted_named_read, worth_native_session},
    crate::shared_support_worth_native_runtime::{
        build_server, build_server_with_workspace_provider,
    },
    crate::worth_native_facade_entry::direct_delivery_runtime::DeclarationAdmitsButLiveDeliveryDeniesProvider,
};

#[test]
fn direct_delivery_denies_durable_resume_debt() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = direct_request(WorthServerQueryRequestedResume::durable());
    let denial = direct_delivery_denied(session.direct().negotiate_delivery(&lease, &request));
    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::DurableResumeDeferred
    );
    assert!(denial.detail().contains("durable resume remains deferred"));
}

#[test]
fn direct_delivery_denies_runtime_backed_resume_without_basis() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = direct_request(WorthServerQueryRequestedResume::runtime_backed(
        None::<String>,
    ));
    let denial = direct_delivery_denied(session.direct().negotiate_delivery(&lease, &request));
    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::RuntimeBackedResumeMissingBasis
    );
    assert!(denial.detail().contains(lease.resume_basis_digest()));
}

#[test]
fn direct_delivery_denies_runtime_backed_resume_with_stale_basis() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = direct_request(WorthServerQueryRequestedResume::runtime_backed(Some(
        "basis:drifted",
    )));
    let denial = direct_delivery_denied(session.direct().negotiate_delivery(&lease, &request));
    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::RuntimeBackedResumeStaleBasis
    );
    assert!(denial.detail().contains("basis:drifted"));
    assert!(denial.detail().contains(lease.resume_basis_digest()));
}

#[test]
fn direct_delivery_fails_closed_when_live_family_is_not_admitted() {
    let server = build_server_with_workspace_provider(
        DeclarationAdmitsButLiveDeliveryDeniesProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                WorthQueryRuntimeFamilySupport::unsupported(
                    WorthQueryRuntimeFacadeFamily::Live,
                    "live delivery is intentionally denied in this hostile test profile",
                ),
            ),
        ),
        true,
    );
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = direct_request(WorthServerQueryRequestedResume::none());
    let denial = direct_delivery_denied(session.direct().negotiate_delivery(&lease, &request));
    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert!(denial
        .detail()
        .contains("query workspace does not admit `live` facade family"));
}

#[test]
fn direct_delivery_denies_cross_workspace_lease_reuse() {
    let server_a = build_server(true);
    let server_b = build_server_with_workspace_provider(TestWorkspaceProvider, true);
    let session_a = worth_native_session(&server_a);
    let declaration_a = admitted_named_read(&session_a, "users.profile");
    let lease = direct_lease_success(session_a.direct().declare_lease(&declaration_a));
    let session_b = match server_b.worth_native().session(
        crate::shared_support_worth_native_runtime::worth_native_session_input_builder()
            .with_workspace_id("workspace-84")
            .build()
            .expect("session input should validate"),
    ) {
        TransitionOutcome::Success(session) => session,
        other => panic!("expected Worth-native session, got {other:?}"),
    };
    let denial = direct_delivery_denied(session_b.direct().negotiate_delivery(
        &lease,
        &direct_request(WorthServerQueryRequestedResume::none()),
    ));
    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::LeaseDeclarationContextMismatch
    );
    assert!(denial.detail().contains("workspace=`workspace-42`"));
}

#[test]
fn direct_delivery_denies_cross_branch_lease_reuse() {
    let server = build_server(true);
    let main_session = worth_native_session(&server);
    let declaration = admitted_named_read(&main_session, "users.profile");
    let lease = direct_lease_success(main_session.direct().declare_lease(&declaration));
    let branch_session = match server.worth_native().session(
        crate::shared_support_worth_native_runtime::worth_native_session_input_builder()
            .with_branch_id("branch-9")
            .build()
            .expect("branch session input should validate"),
    ) {
        TransitionOutcome::Success(session) => session,
        other => panic!("expected branch Worth-native session, got {other:?}"),
    };
    let denial = direct_delivery_denied(branch_session.direct().negotiate_delivery(
        &lease,
        &direct_request(WorthServerQueryRequestedResume::none()),
    ));
    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::LeaseDeclarationContextMismatch
    );
    assert!(denial.detail().contains("branch=`main`"));
    assert!(denial.detail().contains("branch=`branch:branch-9`"));
}
