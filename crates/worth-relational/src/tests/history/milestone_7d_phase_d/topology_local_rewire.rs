use super::*;

#[test]
fn disjoint_rewire_neighborhoods_do_not_escalate_to_topology_region_conflict() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_topology_identity_registry();
    let source_left = create_entity(execution, &runtime, "source-left");
    let target_left = create_entity(execution, &runtime, "target-left");
    let target_left_rewired = create_entity(execution, &runtime, "target-left-rewired");
    let source_right = create_entity(execution, &runtime, "source-right");
    let target_right = create_entity(execution, &runtime, "target-right");
    let target_right_rewired = create_entity(execution, &runtime, "target-right-rewired");
    let relation_left = crate::tests::support::create_relation(
        execution,
        &runtime,
        source_left,
        target_left,
        "edge-left",
    );
    let relation_right = crate::tests::support::create_relation(
        execution,
        &runtime,
        source_right,
        target_right,
        "edge-right",
    );
    create_branch_from_main(&runtime, "feature");
    delete_relation_on_branch(
        execution,
        &runtime,
        relation_left,
        BranchId("feature".to_string()),
    );
    delete_relation_on_branch(
        execution,
        &runtime,
        relation_right,
        BranchId("feature".to_string()),
    );
    create_relation_in_partition_on_branch(
        execution,
        &runtime,
        source_left,
        target_left_rewired,
        "edge-left",
        "edge-left",
        crate::facade::identity::PartitionId::main(),
        BranchId("feature".to_string()),
    );
    create_relation_in_partition_on_branch(
        execution,
        &runtime,
        source_right,
        target_right_rewired,
        "edge-right",
        "edge-right",
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
        .expect("disjoint rewires planning artifact");

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
    assert_eq!(local_topology_records.len(), 2);
    assert_eq!(
        artifact
            .lowered_plan
            .records
            .iter()
            .filter(|record| {
                matches!(
                    record.resolution_class,
                    MergeResolutionClass::Topology(TopologyExecutionClass::TopologyRegionConflict)
                )
            })
            .count(),
        0
    );

    for record in &local_topology_records {
        assert_eq!(
            record.blocked_reason,
            Some(LoweredMergeBlockedReason::RelationEndpointRewiredEscalated)
        );
        let classification = artifact
            .conflict_classification
            .classifications
            .iter()
            .find(|classification| classification.record == record.record)
            .expect("local topology classification");
        let evidence = classification
            .relation_evidence
            .as_ref()
            .expect("local topology evidence");
        assert_eq!(
            evidence.propagation,
            RelationConflictPropagation::RelationLocalRewireCandidate
        );
        assert_eq!(evidence.topology_region_conflict_reason, None);
        assert_eq!(evidence.topology_neighborhood_rewired_records.len(), 1);
        assert_eq!(evidence.topology_neighborhood_records.len(), 2);
    }

    let counters = runtime.performance_access().counters();
    assert_eq!(counters.merge_topology_region_conflicts_detected, 0);
    assert_eq!(counters.merge_topology_region_records_escalated, 0);
}

#[path = "topology_local_rewire/neighborhood_detection.rs"]
mod neighborhood_detection;
