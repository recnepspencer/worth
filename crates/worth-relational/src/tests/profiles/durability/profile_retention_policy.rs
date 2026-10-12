use crate::tests::support::*;

#[test]
fn geometry_profile_does_not_force_retention_pass_on_each_commit() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);

    runtime.performance_access().reset_counters();
    let created = create_entity_outcome(execution, &runtime, "geometry-hot-retention");
    let entity = changed_entities(&created)[0];
    let counters_after_create = runtime.performance_access().counters();

    assert_eq!(counters_after_create.retention_entity_slots_scanned, 0);
    assert_eq!(counters_after_create.retention_relation_slots_scanned, 0);

    runtime.performance_access().reset_counters();
    let deleted = delete_entity(execution, &runtime, entity);
    let counters_after_delete = runtime.performance_access().counters();

    assert_eq!(counters_after_delete.retention_entity_slots_scanned, 0);
    assert_eq!(counters_after_delete.retention_relation_slots_scanned, 0);

    assert!(runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .is_ok());
    assert!(runtime
        .visibility_authority()
        .release_snapshot(&deleted.snapshot)
        .is_ok());

    let plan = runtime.retention().inspect_plan();
    assert!(plan.reclaimable_entities >= 1);
}

#[test]
fn ai_profile_commit_and_release_only_make_reclamation_eligible() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::AiWorkflow);
    let entity = create_entity(execution, &runtime, "ai-retention");
    runtime.performance_access().reset_counters();

    let deleted = delete_entity(execution, &runtime, entity);
    runtime
        .snapshots()
        .release_snapshot(&deleted.snapshot)
        .unwrap();
    let ordinary = runtime.performance_access().counters();
    assert_eq!(ordinary.retention_entity_slots_scanned, 0);
    assert_eq!(ordinary.retention_relation_slots_scanned, 0);

    let maintenance = runtime.retention().run_pass();
    assert!(maintenance.entity_chunks_scanned > 0);
    let after_maintenance = runtime.performance_access().counters();
    assert!(after_maintenance.retention_entity_slots_scanned > 0);
}
