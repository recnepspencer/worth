use super::*;

#[test]
fn session_creation_coordination_admits_without_preexisting_session_identity() {
    let server = operation_request_test_server();
    let resolved = worth_native_resolved_context(&server, None);
    let admission = match server.middleware().admit(WorthServerPipelineInput::new(
        resolved,
        WorthServerPipelineIntent::worth_native_session("product_session.open_mutation"),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected Worth-native session admission, got {other:?}"),
    };
    let operation_request = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            WorthServerOperationRequestInput::builder()
                .with_operation_family(WorthServerOperationFamily::ProductSessionCoordination)
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
        WorthServerOperationAuthorityKind::ProductSessionCoordination
    );
    assert!(matches!(
        posture
            .authority_metadata()
            .product_session_coordination_target(),
        Some((
            WorthServerProductSessionCoordinationTarget::SessionCreation,
            "product-session"
        ))
    ));
    assert!(matches!(
        posture.authority_footprint().scope(),
        WorthServerOperationScope::WorkspaceBranch { .. }
    ));
}
