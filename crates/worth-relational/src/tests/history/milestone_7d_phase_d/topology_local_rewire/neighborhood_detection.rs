use super::*;

#[test]
fn unrelated_relation_additions_do_not_inflate_topology_region_detection_counters() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_topology_identity_registry(
        crate::tests::support::TestDurableMedium::new(),
    );
    let source = create_entity(execution, &runtime, "source");
    let target_a = create_entity(execution, &runtime, "target-a");
    let target_b = create_entity(execution, &runtime, "target-b");
    let unrelated_source = create_entity(execution, &runtime, "unrelated-source");
    let unrelated_target_a = create_entity(execution, &runtime, "unrelated-target-a");
    let unrelated_target_b = create_entity(execution, &runtime, "unrelated-target-b");
    let relation = crate::tests::support::create_relation(
        execution,
        &runtime,
        source,
        target_a,
        "shared-edge",
    );
    create_branch_from_main(&runtime, "feature");
    delete_relation_on_branch(
        execution,
        &runtime,
        relation,
        BranchId("feature".to_string()),
    );
    create_relation_in_partition_on_branch(
        execution,
        &runtime,
        source,
        target_b,
        "shared-edge",
        "shared-edge",
        crate::facade::identity::PartitionId::main(),
        BranchId("feature".to_string()),
    );
    create_relation_in_partition_on_branch(
        execution,
        &runtime,
        unrelated_source,
        unrelated_target_a,
        "unrelated-a",
        "unrelated-a",
        crate::facade::identity::PartitionId::main(),
        BranchId("feature".to_string()),
    );
    create_relation_in_partition_on_branch(
        execution,
        &runtime,
        unrelated_source,
        unrelated_target_b,
        "unrelated-b",
        "unrelated-b",
        crate::facade::identity::PartitionId::main(),
        BranchId("feature".to_string()),
    );

    runtime.performance_access().reset_counters();
    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            }
            .into(),
        )
        .expect("planning artifact");

    let local_topology_records = artifact
        .lowered_plan
        .records
        .iter()
        .filter(|record| {
            matches!(
                record.resolution_class,
                MergeResolutionClass::Topology(
                    TopologyExecutionClass::RelationEndpointRewiredEscalated
                )
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(local_topology_records.len(), 1);

    let counters = runtime.performance_access().counters();
    assert_eq!(counters.merge_topology_relation_candidates_scoped, 2);
    assert_eq!(counters.merge_topology_endpoint_incidences_scoped, 5);
    assert_eq!(counters.merge_topology_region_conflicts_detected, 0);
    assert_eq!(counters.merge_topology_region_records_escalated, 0);
}
