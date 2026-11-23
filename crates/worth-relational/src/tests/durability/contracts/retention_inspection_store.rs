//! Fixture adaptation (C2.5): restores f9f15bf8c8 branch-head legacy pin
//! isolation over the Memory runtime (no retired test medium).
use super::*;

#[test]
fn durability_contract_branch_heads_do_not_mutate_legacy_record_pins() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "source");
    let entity = changed_entities(&created)[0];
    let inspection = runtime.inspect_what_happened();
    assert_eq!(
        inspection
            .inspect_record_retention(execution, RecordRef::Entity(entity))
            .unwrap()
            .expect("entity retention after create")
            .pins
            .branch_pins,
        0
    );

    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let inspection = runtime.inspect_what_happened();
    assert_eq!(
        inspection
            .inspect_record_retention(execution, RecordRef::Entity(entity))
            .unwrap()
            .expect("entity retention after branch create")
            .pins
            .branch_pins,
        0
    );

    update_entity(execution, &runtime, entity, "main");
    let inspection = runtime.inspect_what_happened();
    assert_eq!(
        inspection
            .inspect_record_retention(execution, RecordRef::Entity(entity))
            .unwrap()
            .expect("entity retention after main update")
            .pins
            .branch_pins,
        0
    );

    update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "feature",
        BranchId("feature".to_string()),
    );
    let inspection = runtime.inspect_what_happened();
    assert_eq!(
        inspection
            .inspect_record_retention(execution, RecordRef::Entity(entity))
            .unwrap()
            .expect("entity retention after feature update")
            .pins
            .branch_pins,
        0
    );
}
