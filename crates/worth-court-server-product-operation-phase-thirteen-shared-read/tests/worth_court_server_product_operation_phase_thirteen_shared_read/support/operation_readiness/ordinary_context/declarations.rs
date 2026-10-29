#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};
pub(crate) use {serde_json::json};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_server::WorthServerOperationAuthorityDeclaration, worth_server::WorthServerOperationAuthorizationPolicy, worth_server::WorthServerOperationFamily, worth_server::WorthServerOperationInputEnvelope, worth_server::WorthServerOperationReadinessDenialCode, worth_server::WorthServerOperationRegistration, worth_server::WorthServerOperationRequestInput, worth_server::WorthServerPipelineInput, worth_server::WorthServerPipelineIntent, worth_server::WorthServerProductSupportPosture, worth_server::WorthServerSharedReadBasisKind, worth_server::WorthServerSurfaceFamily};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_operation_request_runtime as operation_request_runtime};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_query_handoff_fixture as query_handoff_fixture};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime};

pub(crate) use {worth_testkit_server::shared_support_operation_request_runtime::operation_request_test_server, worth_testkit_server::shared_support_operation_request_runtime::operation_request_test_server_with_operations, worth_testkit_server::shared_support_operation_request_runtime::worth_native_resolved_context};
pub(crate) fn query_read_admission(server: &worth_server::WorthServer) -> worth_server::WorthServerAdmission {
    match server.middleware().admit(worth_server::WorthServerPipelineInput::new(
        worth_native_resolved_context(server, None),
        worth_server::WorthServerPipelineIntent::query_read("users.profile"),
    )) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected query read admission, got {other:?}"),
    }
}

pub(crate) fn product_read_server(
    basis_kind: WorthServerSharedReadBasisKind,
    support_posture: WorthServerProductSupportPosture,
) -> worth_server::WorthServer {
    let mut registrations = worth_server::WorthServerOperationRegistration::phase_two_defaults();
    registrations.retain(|registration| {
        registration.family() != worth_server::WorthServerOperationFamily::ProductApplicationRead
    });
    registrations.push(
        worth_server::WorthServerOperationRegistration::enabled(
            worth_server::WorthServerOperationFamily::ProductApplicationRead,
        )
        .exposed_on([
            worth_server::WorthServerSurfaceFamily::WorthNative,
            worth_server::WorthServerSurfaceFamily::CompatHttp,
        ])
        .admit_operation_names(["editor.render"])
        .with_authority_declaration(
            worth_server::WorthServerOperationAuthorityDeclaration::product_shared_read_with_support_posture(
                basis_kind,
                support_posture,
            ),
        )
        .with_authorization_policy(worth_server::WorthServerOperationAuthorizationPolicy::allow_authenticated()),
    );
    operation_request_test_server_with_operations(registrations)
}

pub(crate) fn admitted_product_read_posture(
    server: &worth_server::WorthServer,
) -> worth_server::WorthServerOperationAdmissionPosture {
    let admission = query_read_admission(server);
    let request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            worth_server::WorthServerOperationRequestInput::builder()
                .with_operation_family(worth_server::WorthServerOperationFamily::ProductApplicationRead)
                .with_operation_name("editor.render")
                .with_basis_digest("basis-users-profile")
                .build(),
        )
        .expect("product read request should admit");
    server
        .operation_admissions()
        .admit_declared(&admission, &request)
        .expect("product read posture should admit")
}