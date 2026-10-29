use super::*;

#[test]
fn direct_mutation_canonicalizes_operation_name_before_handoff() {
    let server = crate::shared_support_worth_native_runtime::build_server_with_workspace_provider(
        RealMutationWorkspaceProvider,
        true,
    );
    let mutation = direct_mutation_success(worth_native_session(&server).direct().mutate(
        &WorthServerQueryOperation::single_mutation("Tasks.Insert", insert_task("task-1")),
    ));

    assert_eq!(
        mutation.operation_request().identity().operation_name(),
        "tasks.insert"
    );
}

#[test]
fn direct_mutation_denies_backend_verified_assertions_at_the_server_boundary() {
    let server = build_server(true);
    let denial = direct_mutation_denied(worth_native_session(&server).direct().mutate(
        &WorthServerQueryOperation::single_mutation(
            "tasks.verify-existing",
            verify_existing_task("authority:task-1", "task-1"),
        ),
    ));

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::DirectMutationAssertionDenied
    );
    assert!(denial.detail().contains("does not admit backend-verified"));
}

#[test]
fn direct_mutation_preserves_query_contract_denials() {
    let server = crate::shared_support_worth_native_runtime::build_server_with_workspace_provider(
        RealMutationWorkspaceProvider,
        true,
    );
    let invalid = WorthQueryAspectMutationBuilder::new()
        .aspect("identity.id", "task-1")
        .aspect("title.value", 42_i64)
        .build_insert("Task")
        .expect("authored mutation should remain untrusted until runtime contract admission");

    let denial = direct_mutation_denied(worth_native_session(&server).direct().mutate(
        &WorthServerQueryOperation::single_mutation("tasks.insert", invalid),
    ));

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::DirectMutationContractDenied
    );
    assert!(denial
        .detail()
        .contains("native mutation contract admission denied"));
}
