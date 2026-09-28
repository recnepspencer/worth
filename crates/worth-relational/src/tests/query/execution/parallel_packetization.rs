use super::*;

#[test]
fn planned_query_execution_parallelizes_profitable_multi_seed_traversal_packets() {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(declared_aspect_schema_registry(
            CascadeDeletePolicy::CascadeDeleteRelations,
        ))
        .execution_model(crate::facade::runtime::RelationalExecutionModel::ParallelPreparation)
        .build();
    let seeds = vec![
        create_entity_in_partition(&runtime, "s0", PartitionId(7)),
        create_entity_in_partition(&runtime, "s1", PartitionId(11)),
        create_entity_in_partition(&runtime, "s2", PartitionId(13)),
        create_entity_in_partition(&runtime, "s3", PartitionId(17)),
        create_entity_in_partition(&runtime, "s4", PartitionId(19)),
    ];
    let neighbors = [
        create_entity_in_partition(&runtime, "n0", PartitionId(23)),
        create_entity_in_partition(&runtime, "n1", PartitionId(29)),
        create_entity_in_partition(&runtime, "n2", PartitionId(31)),
        create_entity_in_partition(&runtime, "n3", PartitionId(37)),
        create_entity_in_partition(&runtime, "n4", PartitionId(41)),
    ];
    let relations = seeds
        .iter()
        .zip(neighbors.iter())
        .enumerate()
        .map(|(index, (seed, neighbor))| {
            create_relation_in_partition(
                &runtime,
                *seed,
                *neighbor,
                &format!("edge-{index}"),
                PartitionId(43 + index as u32),
            )
        })
        .collect::<Vec<_>>();
    let snapshot = runtime.visibility_authority().snapshot();
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .expect("query plan context");
    let packet = PlannedQueryPacket {
        label: "parallel-traversal".to_string(),
        context_id: context,
        scope: QueryScope::OutgoingNeighborhood {
            seeds: Arc::from(seeds.clone()),
            relation_kind_scope: Some(Arc::from([KindId(2)])),
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(1001),
        target_count_hint: seeds.len(),
    };

    runtime.performance_access().reset_counters();
    let plan = runtime
        .read_truth()
        .plan_query_packet(&snapshot, packet)
        .expect("planned query packet");
    let outcome = runtime
        .read_truth()
        .execute_query_plan(plan)
        .expect("query execution outcome");
    let counters = runtime.performance_access().counters();
    let expected_packet_count = 2;

    assert_eq!(outcome.complexity.packet_count, expected_packet_count);
    assert_eq!(outcome.complexity.fragment_count, expected_packet_count);
    assert_eq!(counters.query_packet_count, expected_packet_count);
    assert_eq!(counters.query_packet_item_count, seeds.len());
    assert_eq!(counters.query_packet_peak_width_total, 4);
    assert_eq!(counters.query_parallel_legal_count, 1);
    assert_eq!(counters.query_parallel_profitable_count, 1);
    assert_eq!(counters.query_staged_parallel_strategy_count, 1);
    assert_eq!(counters.query_serial_strategy_count, 0);
    assert_eq!(
        outcome
            .result
            .entities
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        seeds
            .iter()
            .copied()
            .chain(neighbors.iter().copied())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        outcome
            .result
            .relations
            .iter()
            .map(|record| record.relation_id)
            .collect::<Vec<_>>(),
        relations
    );
}
