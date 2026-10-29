use {worth_proof::TransitionOutcome, worth_server::WorthServerOperationAuthorityKind, worth_server::WorthServerOperationFamily, worth_server::WorthServerOperationRequestInput, worth_server::WorthServerOperationScope, worth_server::WorthServerPipelineInput, worth_server::WorthServerPipelineIntent, worth_server::WorthServerProductSessionCoordinationTarget, worth_testkit_server::shared_support_operation_request_runtime::operation_request_test_server, worth_testkit_server::shared_support_operation_request_runtime::worth_native_resolved_context};

#[test]
fn session_creation_coordination_admits_without_preexisting_session_identity() {
    let server = operation_request_test_server();
    let resolved = worth_native_resolved_context(&server, None);
    let admission = match server.middleware().admit(worth_server::WorthServerPipelineInput::new(
        resolved,
        worth_server::WorthServerPipelineIntent::worth_native_session("product_session.open_mutation"),
    )) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected Worth-native session admission, got {other:?}"),
    };
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            worth_server::WorthServerOperationRequestInput::builder()
                .with_operation_family(worth_server::WorthServerOperationFamily::ProductSessionCoordination)
                .with_operation_name("product_session.open_mutation")
                .with_basis_digest("basis-editor-1")
                .build(),
        )
        .expect("session creation request should admit without a fabricated identity");

    let posture = server
        .operation_admissions()
        .admit_declared(&admission, &operation_request)
        .expect("session creation posture should admit");

    assert_eq!(
        posture.authority_footprint().authority_kind(),
        worth_server::WorthServerOperationAuthorityKind::ProductSessionCoordination
    );
    assert!(matches!(
        posture
            .authority_metadata()
            .product_session_coordination_target(),
        Some((
            worth_server::WorthServerProductSessionCoordinationTarget::SessionCreation,
            "product-session"
        ))
    ));
    assert!(matches!(
        posture.authority_footprint().scope(),
        worth_server::WorthServerOperationScope::WorkspaceBranch { .. }
    ));
}
