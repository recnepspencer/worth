//! Merge inspection artifacts: input round-trip and execution-surface authority rows.
use super::branch_history_publications::{
    create_entity, delete_entity_on_branch, delete_relation_on_branch, fork, update_entity,
};
use super::merge_execution_authoring::create_relation_in_partition_on_branch;
use super::merge_proof_journey::merge_request;
use super::merge_witness_authoring::{create_empty_entity, create_relation};
use super::merge_witness_schema::runtime_with_relation_identity_registry;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::merge::{
    LoweredMergeBlockedReason, MergeResolutionClass, RelationalMergeInspectionAdmission,
    RelationalMergeInspectionArtifact, TopologyExecutionClass,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::durable_runtime;

#[test]
fn inspection_input_round_trips_deleted_vs_modified_without_host_projection() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    update_entity(execution, &runtime, entity, "main-modified");
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    let request = merge_request();
    let planning = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(request.clone().into())
                .unwrap(),
        )
        .expect("planning artifact");
    let input = planning.inspection_input();
    let from_input = RelationalMergeInspectionArtifact::from_input(input.clone());
    let from_access = runtime
        .merge()
        .inspect_execution_surface(
            execution,
            runtime
                .bind_merge_planning_request(request.clone().into())
                .unwrap(),
        )
        .expect("execution surface");
    let normalized_request = runtime
        .merge()
        .normalize_merge_request(runtime.bind_merge_execution_request(request).unwrap())
        .expect("normalized request");

    assert_eq!(input.request(), &normalized_request);
    assert_eq!(input.lowered_plan(), &planning.lowered_plan);
    assert_eq!(from_input, from_access);
    assert_eq!(from_input.request(), &normalized_request);
    assert_eq!(from_input.rows().len(), planning.lowered_plan.records.len());

    let denied = from_input
        .rows()
        .iter()
        .find(|row| row.blocked_reason() == Some(LoweredMergeBlockedReason::DeletedVsModified))
        .expect("deleted-vs-modified row");
    assert_eq!(
        denied.admission(),
        RelationalMergeInspectionAdmission::ExecutionDenied
    );
    assert_eq!(
        denied.blocked_reason(),
        Some(LoweredMergeBlockedReason::DeletedVsModified)
    );
}

#[test]
fn execution_surface_preserves_topology_region_conflict_authority_rows() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = runtime_with_relation_identity_registry(&medium);
    let source = create_empty_entity(execution, &runtime, "source");
    let target_a = create_empty_entity(execution, &runtime, "target-a");
    let target_b = create_empty_entity(execution, &runtime, "target-b");
    let target_c = create_empty_entity(execution, &runtime, "target-c");
    let target_d = create_empty_entity(execution, &runtime, "target-d");
    let relation_a = create_relation(execution, &runtime, source, target_a, "edge-a");
    let relation_b = create_relation(execution, &runtime, source, target_b, "edge-b");
    fork(execution, &runtime, "feature");
    delete_relation_on_branch(
        execution,
        &runtime,
        relation_a,
        BranchId("feature".to_string()),
    );
    delete_relation_on_branch(
        execution,
        &runtime,
        relation_b,
        BranchId("feature".to_string()),
    );
    create_relation_in_partition_on_branch(
        execution,
        &runtime,
        source,
        target_c,
        "edge-a",
        "edge-a",
        PartitionId::main(),
        BranchId("feature".to_string()),
    );
    create_relation_in_partition_on_branch(
        execution,
        &runtime,
        source,
        target_d,
        "edge-b",
        "edge-b",
        PartitionId::main(),
        BranchId("feature".to_string()),
    );

    let artifact = runtime
        .merge()
        .inspect_execution_surface(
            execution,
            runtime
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
        )
        .expect("execution surface");

    let topology_rows = artifact
        .rows()
        .iter()
        .filter(|row| {
            row.blocked_reason() == Some(LoweredMergeBlockedReason::TopologyRegionConflict)
                && matches!(
                    row.resolution_class(),
                    MergeResolutionClass::Topology(TopologyExecutionClass::TopologyRegionConflict)
                )
        })
        .collect::<Vec<_>>();
    assert_eq!(topology_rows.len(), 2);
    assert!(topology_rows
        .iter()
        .all(|row| { row.admission() == RelationalMergeInspectionAdmission::ExecutionDenied }));
}
