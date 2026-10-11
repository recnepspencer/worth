use super::*;

#[test]
fn concurrent_relation_index_certification_parity_stays_stable_under_scheduler_pressure() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity_outcome(execution, &runtime, "source");
    let source_id = changed_entities(&source)[0];
    let targets = [
        create_entity_in_partition(execution, &runtime, "r0", PartitionId(7)),
        create_entity_in_partition(execution, &runtime, "r1", PartitionId(11)),
        create_entity_in_partition(execution, &runtime, "r2", PartitionId(13)),
    ];
    for (index, target) in targets.into_iter().enumerate() {
        create_relation_in_partition(
            execution,
            &runtime,
            source_id,
            target,
            if index < 2 { "fast" } else { "slow" },
            PartitionId(23 + index as u32),
        );
    }
    let commit = create_entity_outcome(execution, &runtime, "anchor");
    let relation_index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "relation.name".to_string(),
        kind: DerivedIndexKind::RelationField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: commit.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![relation_index.index_id],
        },
    );

    let snapshot = commit.snapshot.clone();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "relation-index-certification".to_string(),
        context_id: context,
        scope: QueryScope::RelationFieldEquals {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            value: string_aspect_value("fast"),
            partition_scope: None,
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalRelationIdOrder,
        access_contract: QueryAccessContract::DerivedIndexWithStorageParity,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(1401),
        target_count_hint: 0,
    };
    let baseline = runtime
        .index_access()
        .execute_query_plan_with_index_parity(
            runtime
                .read_truth()
                .plan_query_packet(&snapshot, packet.clone())
                .expect("baseline plan"),
            IndexParityMode::CertificationParity,
            execution,
        )
        .expect("request remains live")
        .expect("baseline outcome");
    let runtime = Arc::new(runtime);

    std::thread::scope(|scope| {
        let mut readers = Vec::new();
        for _ in 0..8 {
            let runtime = Arc::clone(&runtime);
            let snapshot = snapshot.clone();
            let packet = packet.clone();
            let expected = baseline.clone();
            readers.push(scope.spawn(move || {
                // Each independent reader call owns its request budget.
                let host_request = crate::test_execution_policy::serial_request();
                let execution = &worth_execution::ExecutionRequest::serial(&host_request);
                let outcome = runtime
                    .index_access()
                    .execute_query_plan_with_index_parity(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, packet)
                            .expect("thread plan"),
                        IndexParityMode::CertificationParity,
                        execution,
                    )
                    .expect("request remains live")
                    .expect("thread outcome");
                assert_eq!(outcome.access_path, expected.access_path);
                assert_eq!(outcome.execution.result, expected.execution.result);
                assert_eq!(outcome.parity_basis_digest, expected.parity_basis_digest);
            }));
        }

        for reader in readers {
            reader.join().unwrap();
        }
    });
}
