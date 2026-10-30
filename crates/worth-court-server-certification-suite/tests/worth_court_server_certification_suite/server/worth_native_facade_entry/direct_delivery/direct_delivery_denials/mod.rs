use {
    crate::worth_native_facade_entry::ordinary_context::direct_delivery::direct_delivery_denied,
    crate::worth_native_facade_entry::ordinary_context::direct_delivery::direct_lease_success,
    crate::worth_native_facade_entry::ordinary_context::direct_delivery::direct_request,
};
use {
    worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider,
    worth_testkit_server::shared_support_worth_native_assertions::admitted_named_read,
    worth_testkit_server::shared_support_worth_native_assertions::worth_native_session,
    worth_testkit_server::shared_support_worth_native_runtime::build_server,
    worth_testkit_server::shared_support_worth_native_runtime::build_server_with_workspace_provider,
};

#[test]
fn direct_delivery_denies_durable_resume_debt() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = direct_request(worth_server::WorthServerQueryRequestedResume::durable());
    let denial = direct_delivery_denied(session.direct().negotiate_delivery(&lease, &request));
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::DurableResumeDeferred
    );
    assert!(denial.detail().contains("durable resume remains deferred"));
}

#[test]
fn direct_delivery_denies_runtime_backed_resume_without_basis() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = direct_request(
        worth_server::WorthServerQueryRequestedResume::runtime_backed(None::<String>),
    );
    let denial = direct_delivery_denied(session.direct().negotiate_delivery(&lease, &request));
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::RuntimeBackedResumeMissingBasis
    );
    assert!(denial.detail().contains(lease.resume_basis_digest()));
}

#[test]
fn direct_delivery_denies_runtime_backed_resume_with_stale_basis() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = direct_request(
        worth_server::WorthServerQueryRequestedResume::runtime_backed(Some("basis:drifted")),
    );
    let denial = direct_delivery_denied(session.direct().negotiate_delivery(&lease, &request));
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::RuntimeBackedResumeStaleBasis
    );
    assert!(denial.detail().contains("basis:drifted"));
    assert!(denial.detail().contains(lease.resume_basis_digest()));
}

#[test]
fn direct_delivery_fails_closed_when_live_family_is_not_admitted() {
    let server = build_server_with_workspace_provider(
        crate::worth_native_facade_entry::ordinary_context::direct_delivery_runtime::DeclarationAdmitsButLiveDeliveryDeniesProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                worth_query::facade::runtime::WorthQueryRuntimeFamilySupport::unsupported(
                    worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::Live,
                    "live delivery is intentionally denied in this hostile test profile",
                ),
            ),
        ),
        true,
    );
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let lease = direct_lease_success(session.direct().declare_lease(&declaration));
    let request = direct_request(worth_server::WorthServerQueryRequestedResume::none());
    let denial = direct_delivery_denied(session.direct().negotiate_delivery(&lease, &request));
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
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
        worth_testkit_server::shared_support_worth_native_runtime::worth_native_session_input_builder()
            .with_workspace_id("workspace-84")
            .build()
            .expect("session input should validate"),
    ) {
        worth_proof::TransitionOutcome::Success(session) => session,
        other => panic!("expected Worth-native session, got {other:?}"),
    };
    let denial = direct_delivery_denied(session_b.direct().negotiate_delivery(
        &lease,
        &direct_request(worth_server::WorthServerQueryRequestedResume::none()),
    ));
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::LeaseDeclarationContextMismatch
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
        worth_testkit_server::shared_support_worth_native_runtime::worth_native_session_input_builder()
            .with_branch_id("branch-9")
            .build()
            .expect("branch session input should validate"),
    ) {
        worth_proof::TransitionOutcome::Success(session) => session,
        other => panic!("expected branch Worth-native session, got {other:?}"),
    };
    let denial = direct_delivery_denied(branch_session.direct().negotiate_delivery(
        &lease,
        &direct_request(worth_server::WorthServerQueryRequestedResume::none()),
    ));
    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::LeaseDeclarationContextMismatch
    );
    assert!(denial.detail().contains("branch=`main`"));
    assert!(denial.detail().contains("branch=`branch:branch-9`"));
}
