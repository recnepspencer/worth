use super::*;

#[test]
fn topology_region_conflict_detection_reports_bounded_neighborhood_counters() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_topology_identity_registry();
    let source = create_entity(execution, &runtime, "source");
    let target_a = create_entity(execution, &runtime, "target-a");
    let target_b = create_entity(execution, &runtime, "target-b");
    let target_c = create_entity(execution, &runtime, "target-c");
    let target_d = create_entity(execution, &runtime, "target-d");
    let relation_a =
        crate::tests::support::create_relation(execution, &runtime, source, target_a, "edge-a");
    let relation_b =
        crate::tests::support::create_relation(execution, &runtime, source, target_b, "edge-b");
    create_branch_from_main(&runtime, "feature");
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
        crate::facade::identity::PartitionId::main(),
        BranchId("feature".to_string()),
    );
    create_relation_in_partition_on_branch(
        execution,
        &runtime,
        source,
        target_d,
        "edge-b",
        "edge-b",
        crate::facade::identity::PartitionId::main(),
        BranchId("feature".to_string()),
    );

    runtime.performance_access().reset_counters();
    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergeExecutionRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            )
            .into(),
        )
        .expect("topology-region planning artifact");

    let topology_records = artifact
        .lowered_plan
        .records
        .iter()
        .filter(|record| {
            matches!(
                record.resolution_class,
                MergeResolutionClass::Topology(TopologyExecutionClass::TopologyRegionConflict)
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(topology_records.len(), 2);
    for record in &topology_records {
        assert_eq!(
            record.blocked_reason,
            Some(LoweredMergeBlockedReason::TopologyRegionConflict)
        );
    }

    let topology_record_refs = topology_records
        .iter()
        .map(|record| record.record.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let topology_classifications = artifact
        .conflict_classification
        .classifications
        .iter()
        .filter(|classification| topology_record_refs.contains(&classification.record))
        .collect::<Vec<_>>();
    assert_eq!(topology_classifications.len(), 2);
    for classification in &topology_classifications {
        let evidence = classification
            .relation_evidence
            .as_ref()
            .expect("topology relation evidence");
        assert_eq!(
            evidence.propagation,
            RelationConflictPropagation::EscalatesToTopologyRegionConflict
        );
        assert_eq!(
            evidence.topology_region_conflict_reason,
            Some(TopologyRegionConflictReason::ConnectedRewireNeighborhood)
        );
        let neighborhood = evidence
            .topology_neighborhood_records
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        let rewired = evidence
            .topology_neighborhood_rewired_records
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(neighborhood.len(), 4);
        assert!(topology_record_refs.is_subset(&neighborhood));
        assert_eq!(rewired, topology_record_refs);
    }

    let counters = runtime.performance_access().counters();
    assert_eq!(counters.merge_topology_relation_candidates_scoped, 4);
    assert!(counters.merge_topology_endpoint_incidences_scoped >= 8);
    assert_eq!(counters.merge_topology_region_conflicts_detected, 1);
    assert_eq!(counters.merge_topology_region_records_escalated, 2);
}
