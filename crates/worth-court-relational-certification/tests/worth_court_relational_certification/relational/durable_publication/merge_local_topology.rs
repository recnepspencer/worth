//! Local endpoint-rewire topology denials of merge planning, live and recovered.
use super::branch_history_publications::{delete_relation_on_branch, fork};
use super::merge_execution_authoring::create_relation_in_partition_on_branch;
use super::merge_execution_recovery::checkpoint_and_recover_durable;
use super::merge_execution_schema::durable_runtime_with_topology_identity_registry;
use super::merge_proof_journey::merge_request;
use super::merge_witness_authoring::{create_empty_entity, create_relation};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::merge::{
    LoweredMergeBlockedReason, LoweredRecordDenialKind, MergeResolutionClass,
    RelationConflictPropagation, TopologyExecutionClass,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

#[test]
fn topology_endpoint_divergence_denial_is_stable_across_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime_with_topology_identity_registry(&medium);
    let source = create_empty_entity(execution, &runtime, "source");
    let target_a = create_empty_entity(execution, &runtime, "target-a");
    let target_b = create_empty_entity(execution, &runtime, "target-b");
    let relation = create_relation(execution, &runtime, source, target_a, "shared-edge");
    fork(execution, &runtime, "feature");
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
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime_with_topology_identity_registry(&medium);
    let source_left = create_empty_entity(execution, &runtime, "source-left");
    let target_left = create_empty_entity(execution, &runtime, "target-left");
    let target_left_rewired = create_empty_entity(execution, &runtime, "target-left-rewired");
    let source_right = create_empty_entity(execution, &runtime, "source-right");
    let target_right = create_empty_entity(execution, &runtime, "target-right");
    let target_right_rewired = create_empty_entity(execution, &runtime, "target-right-rewired");
    let relation_left = create_relation(execution, &runtime, source_left, target_left, "edge-left");
    let relation_right = create_relation(
        execution,
        &runtime,
        source_right,
        target_right,
        "edge-right",
    );
    fork(execution, &runtime, "feature");
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
        PartitionId::main(),
        BranchId("feature".to_string()),
    );
    create_relation_in_partition_on_branch(
        execution,
        &runtime,
        source_right,
        target_right_rewired,
        "edge-right",
        "edge-right",
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
}
