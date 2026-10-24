use super::*;

#[test]
fn topology_endpoint_divergence_denial_is_stable_across_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = crate::tests::support::TestDurableMedium::new();
    let runtime = durable_runtime_with_topology_identity_registry(medium.clone());
    let source = create_entity(execution, &runtime, "source");
    let target_a = create_entity(execution, &runtime, "target-a");
    let target_b = create_entity(execution, &runtime, "target-b");
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

    let live_artifact = runtime
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
        .expect("live planning artifact");
    let live_record = live_artifact
        .lowered_plan
        .records
        .iter()
        .find(|record| {
            matches!(
                record.resolution_class,
                MergeResolutionClass::Topology(
                    TopologyExecutionClass::RelationEndpointRewiredEscalated
                )
            )
        })
        .expect("live topology lowered record");
    let live_classification = live_artifact
        .conflict_classification
        .classifications
        .iter()
        .find(|classification| classification.record == live_record.record)
        .expect("live topology classification");
    let live_index = live_artifact
        .lowered_plan
        .records
        .iter()
        .position(|record| {
            matches!(
                record.resolution_class,
                MergeResolutionClass::Topology(
                    TopologyExecutionClass::RelationEndpointRewiredEscalated
                )
            )
        })
        .expect("live topology index");

    let (_recovery, recovered) = checkpoint_and_recover_with(execution, &runtime, move || {
        durable_runtime_with_topology_identity_registry(medium.clone())
    });
    let recovered_artifact = recovered
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
        .expect("recovered planning artifact");
    let recovered_record = recovered_artifact
        .lowered_plan
        .records
        .iter()
        .find(|record| {
            matches!(
                record.resolution_class,
                MergeResolutionClass::Topology(
                    TopologyExecutionClass::RelationEndpointRewiredEscalated
                )
            )
        })
        .expect("recovered topology lowered record");
    let recovered_index = recovered_artifact
        .lowered_plan
        .records
        .iter()
        .position(|record| {
            matches!(
                record.resolution_class,
                MergeResolutionClass::Topology(
                    TopologyExecutionClass::RelationEndpointRewiredEscalated
                )
            )
        })
        .expect("recovered topology index");

    assert_eq!(
        live_record.blocked_reason,
        Some(LoweredMergeBlockedReason::RelationEndpointRewiredEscalated)
    );
    assert_eq!(
        live_classification
            .relation_evidence
            .as_ref()
            .expect("relation evidence")
            .propagation,
        RelationConflictPropagation::RelationLocalRewireCandidate
    );
    assert_eq!(
        live_classification
            .relation_evidence
            .as_ref()
            .expect("relation evidence")
            .topology_neighborhood_records
            .len(),
        2
    );
    assert_eq!(
        live_classification
            .relation_evidence
            .as_ref()
            .expect("relation evidence")
            .topology_neighborhood_rewired_records
            .len(),
        1
    );
    assert_eq!(
        live_classification
            .relation_evidence
            .as_ref()
            .expect("relation evidence")
            .topology_region_conflict_reason,
        None
    );
    assert_eq!(live_record.executable_class, None);
    assert_eq!(live_record, recovered_record);
    assert_eq!(
        live_artifact.digest_basis.lowered_plan.denial_bundle_kinds[live_index],
        Some(LoweredRecordDenialKind::BlockedRelationEndpointRewiredEscalated)
    );
    assert_eq!(
        recovered_artifact
            .digest_basis
            .lowered_plan
            .denial_bundle_kinds[recovered_index],
        Some(LoweredRecordDenialKind::BlockedRelationEndpointRewiredEscalated)
    );
    assert_eq!(
        live_artifact.digest_basis.lowered_plan,
        recovered_artifact.digest_basis.lowered_plan
    );
}

#[test]
fn disjoint_rewire_neighborhoods_do_not_escalate_to_topology_region_conflict() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_topology_identity_registry(
        crate::tests::support::TestDurableMedium::new(),
    );
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
