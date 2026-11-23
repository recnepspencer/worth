use super::*;

// Fixture adaptation (C2.5): restores f9f15bf8c8 schema-carrier deduplication
// over Memory durability_authority().checkpoint (no retired test medium).
#[test]
fn branch_roots_with_one_schema_deduplicate_the_durable_carrier() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "schema-carrier-deduplication");
    let entity = changed_entities(&created)[0];
    create_branch_from_main(&runtime, "feature");
    update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "schema-carrier-feature-root",
        BranchId("feature".to_owned()),
    );

    let checkpoint = runtime
        .durability_authority()
        .checkpoint(execution)
        .expect("production checkpoint succeeds");

    assert!(checkpoint.branch_roots.len() >= 2);
    assert_eq!(checkpoint.branch_root_schema_images.len(), 1);
}

#[path = "branch_root_schema_binding/exact_contracts.rs"]
mod exact_contracts;
