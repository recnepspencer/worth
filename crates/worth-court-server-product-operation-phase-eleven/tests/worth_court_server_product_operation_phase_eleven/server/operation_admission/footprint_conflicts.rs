use super::*;

#[test]
fn conflicting_product_draft_footprints_fail_before_execution() {
    let server = operation_request_test_server();
    let resolved = worth_native_resolved_context(&server, None);
    let admission = match server
        .middleware()
        .admit(worth_server::WorthServerPipelineInput::new(
            resolved,
            worth_server::WorthServerPipelineIntent::query_mutation("editor.apply"),
        )) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected mutation admission, got {other:?}"),
    };
    let first = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            worth_server::WorthServerOperationRequestInput::builder()
                .with_operation_family(
                    worth_server::WorthServerOperationFamily::ProductApplicationMutation,
                )
                .with_operation_name("editor.apply")
                .with_basis_digest("basis-editor-1")
                .with_idempotency_key("idem-editor-1")
                .with_product_session_identity("session-1")
                .build(),
        )
        .expect("first product mutation should admit");
    let second = server
        .operation_requests()
        .admit_from_worth_native_admission(
            &admission,
            worth_server::WorthServerOperationRequestInput::builder()
                .with_operation_family(
                    worth_server::WorthServerOperationFamily::ProductApplicationMutation,
                )
                .with_operation_name("editor.apply")
                .with_basis_digest("basis-editor-2")
                .with_idempotency_key("idem-editor-2")
                .with_product_session_identity("session-1")
                .build(),
        )
        .expect("second product mutation should admit");

    let first_posture = server
        .operation_admissions()
        .admit_declared(&admission, &first)
        .expect("first product posture should admit");
    let second_posture = server
        .operation_admissions()
        .admit_declared(&admission, &second)
        .expect("second product posture should admit");

    let denial = WorthServerOperationConcurrencyFacade
        .classify_pair(&first_posture, &second_posture)
        .expect_err("same product draft scope should not admit concurrent mutation");

    assert_eq!(
        denial.code(),
        worth_server::WorthServerOperationConcurrencyDenialCode::ConflictingMutableAuthority
    );
    assert!(denial.detail().contains("product draft scope"));
}
