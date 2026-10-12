use crate::tests::support::*;

#[test]
fn query_planning_context_binds_snapshot_runtime_and_schema_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &runtime, "first");

    let context = runtime
        .read_truth()
        .query_plan_context(&committed.snapshot)
        .expect("query plan context");

    assert_eq!(context.runtime_instance_id, runtime.runtime_instance_id());
    assert_eq!(context.snapshot_id, committed.snapshot.snapshot_id);
    assert_eq!(context.version_id, committed.snapshot.version_id);
    assert_eq!(context.schema_version, committed.envelope().schema_version);
    assert_eq!(
        context.descriptor_semantics_version,
        committed.envelope().descriptor_semantics_version
    );
    assert_eq!(
        context.evidence_basis,
        QueryPlanEvidenceBasis::CanonicalCommitEnvelope {
            commit_id: committed.commit.commit_id,
        }
    );
}

#[test]
fn packetized_query_planning_marks_single_target_packets_serial_preferred() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "single");
    let entity = changed_entities(&created)[0];
    let plan = planned_explicit_query(
        &runtime,
        &created.snapshot,
        "single-target",
        vec![RecordRef::Entity(entity)],
    );

    assert_eq!(plan.legality, QueryParallelLegality::LegalReadOnlySnapshot);
    assert_eq!(
        plan.profitability,
        QueryParallelProfitability::SerialPreferred {
            reason: QuerySerialReason::TinyPacket,
        }
    );
}

#[test]
fn packetized_query_planning_marks_single_chunk_packets_serial_preferred() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "first");
    let second = create_entity_outcome(execution, &runtime, "second");
    let targets = vec![
        RecordRef::Entity(changed_entities(&first)[0]),
        RecordRef::Entity(changed_entities(&second)[0]),
    ];

    let plan = planned_explicit_query(&runtime, &second.snapshot, "pair", targets);

    assert_eq!(plan.legality, QueryParallelLegality::LegalReadOnlySnapshot);
    assert_eq!(
        plan.profitability,
        QueryParallelProfitability::SerialPreferred {
            reason: QuerySerialReason::SingleChunkSurface,
        }
    );
}

#[test]
fn traversal_query_packets_are_legal_read_only_snapshots_and_narrow_traversals_stay_serial() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "seed");
    let seed = changed_entities(&created)[0];
    let context = runtime
        .read_truth()
        .query_plan_context(&created.snapshot)
        .expect("query plan context");

    let packet = PlannedQueryPacket {
        label: "connectivity".to_string(),
        context_id: context,
        scope: QueryScope::ConnectivityTraversal {
            seeds: std::sync::Arc::from([seed]),
            relation_kind_scope: None,
            max_depth: Some(2),
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(17),
        target_count_hint: 1,
    };

    let plan = runtime
        .read_truth()
        .plan_query_packet(&created.snapshot, packet)
        .expect("snapshot pinned plan");

    assert_eq!(plan.legality, QueryParallelLegality::LegalReadOnlySnapshot);
    assert_eq!(
        plan.profitability,
        QueryParallelProfitability::SerialPreferred {
            reason: QuerySerialReason::TinyPacket,
        }
    );
}

#[test]
fn multi_seed_traversal_query_packets_become_profitable_after_seed_packetization() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "seed-1");
    let second = create_entity_outcome(execution, &runtime, "seed-2");
    let third = create_entity_outcome(execution, &runtime, "seed-3");
    let fourth = create_entity_outcome(execution, &runtime, "seed-4");
    let fifth = create_entity_outcome(execution, &runtime, "seed-5");
    let seeds = [
        changed_entities(&first)[0],
        changed_entities(&second)[0],
        changed_entities(&third)[0],
        changed_entities(&fourth)[0],
        changed_entities(&fifth)[0],
    ];
    let context = runtime
        .read_truth()
        .query_plan_context(&fifth.snapshot)
        .expect("query plan context");

    let packet = PlannedQueryPacket {
        label: "connectivity-profit".to_string(),
        context_id: context,
        scope: QueryScope::ConnectivityTraversal {
            seeds: std::sync::Arc::from(seeds),
            relation_kind_scope: None,
            max_depth: Some(1),
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(18),
        target_count_hint: 5,
    };

    let plan = runtime
        .read_truth()
        .plan_query_packet(&fifth.snapshot, packet)
        .expect("snapshot pinned plan");

    assert_eq!(plan.legality, QueryParallelLegality::LegalReadOnlySnapshot);
    assert_eq!(plan.profitability, QueryParallelProfitability::Profitable);
}

#[test]
fn query_planning_rejects_packets_with_forged_context() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "forged");
    let entity = changed_entities(&created)[0];
    let mut context = runtime
        .read_truth()
        .query_plan_context(&created.snapshot)
        .expect("query plan context");
    context.version_id = crate::facade::identity::VersionId(context.version_id.0 + 1);

    let packet = PlannedQueryPacket {
        label: "forged-context".to_string(),
        context_id: context,
        scope: QueryScope::ExplicitTargets {
            targets: std::sync::Arc::from([RecordRef::Entity(entity)]),
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalRecordRefOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(99),
        target_count_hint: 1,
    };

    assert!(runtime
        .read_truth()
        .plan_query_packet(&created.snapshot, packet)
        .is_none());
}

#[test]
fn packetized_plan_key_is_deterministic_for_identical_inputs() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "plan-key");
    let entity = changed_entities(&created)[0];

    let first = planned_explicit_query(
        &runtime,
        &created.snapshot,
        "stable",
        vec![RecordRef::Entity(entity)],
    );
    let second = planned_explicit_query(
        &runtime,
        &created.snapshot,
        "stable",
        vec![RecordRef::Entity(entity)],
    );

    assert_ne!(first.packet.plan_key, DeterministicQueryPlanKey(0));
    assert_eq!(first.packet.plan_key, second.packet.plan_key);
}

#[test]
fn empty_runtime_query_planning_uses_explicit_genesis_basis() {
    let runtime = runtime_with_test_schema();
    let snapshot = runtime.visibility_authority().snapshot();

    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("genesis query plan context");

    assert_eq!(context.version_id, crate::facade::identity::VersionId(0));
    assert_eq!(
        context.evidence_basis,
        QueryPlanEvidenceBasis::GenesisRuntimeBootstrap
    );
}
