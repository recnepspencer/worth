//! Connected-rewire topology-region conflicts of merge planning, live and recovered.
use super::branch_history_publications::{delete_relation_on_branch, fork};
use super::merge_execution_authoring::create_relation_in_partition_on_branch;
use super::merge_execution_recovery::checkpoint_and_recover_durable;
use super::merge_execution_schema::durable_runtime_with_topology_identity_registry;
use super::merge_proof_journey::merge_request;
use super::merge_witness_authoring::{create_empty_entity, create_relation};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::merge::{
    LoweredMergeBlockedReason, MergeResolutionClass, RelationConflictPropagation,
    TopologyExecutionClass, TopologyRegionConflictReason,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

#[test]
fn topology_region_conflict_detection_preserves_neighborhood_evidence() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime_with_topology_identity_registry(&medium);
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
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
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
}

#[test]
fn topology_region_conflict_denial_is_stable_across_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime_with_topology_identity_registry(&medium);
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

    let live_artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
        )
        .expect("live topology-region planning artifact");
    let live_topology_records = live_artifact
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
    assert_eq!(live_topology_records.len(), 2);
    let live_record_refs = live_topology_records
        .iter()
        .map(|record| record.record.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let live_classifications = live_artifact
        .conflict_classification
        .classifications
        .iter()
        .filter(|classification| live_record_refs.contains(&classification.record))
        .collect::<Vec<_>>();
    assert_eq!(live_classifications.len(), 2);
    for classification in &live_classifications {
        let evidence = classification
            .relation_evidence
            .as_ref()
            .expect("live topology-region evidence");
        assert_eq!(
            evidence.propagation,
            RelationConflictPropagation::EscalatesToTopologyRegionConflict
        );
        assert_eq!(
            evidence.topology_region_conflict_reason,
            Some(TopologyRegionConflictReason::ConnectedRewireNeighborhood)
        );
        assert_eq!(evidence.topology_neighborhood_records.len(), 4);
        assert_eq!(evidence.topology_neighborhood_rewired_records.len(), 2);
    }

    let (_recovery, recovered) = checkpoint_and_recover_durable(
        execution,
        &runtime,
        &medium,
        durable_runtime_with_topology_identity_registry(&medium),
    );
    let recovered_artifact = recovered
        .merge()
        .inspect_planning_scope(
            execution,
            recovered
                .bind_merge_planning_request(merge_request().into())
                .unwrap(),
        )
        .expect("recovered topology-region planning artifact");

    assert_eq!(
        live_artifact.digest_basis.conflict,
        recovered_artifact.digest_basis.conflict
    );
    assert_eq!(
        live_artifact.digest_basis.lowered_plan,
        recovered_artifact.digest_basis.lowered_plan
    );
    assert_eq!(
        live_artifact.conflict_classification.classifications,
        recovered_artifact.conflict_classification.classifications
    );
    assert_eq!(
        live_artifact.lowered_plan.records,
        recovered_artifact.lowered_plan.records
    );
}
