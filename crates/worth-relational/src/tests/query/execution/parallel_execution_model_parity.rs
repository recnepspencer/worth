use super::*;

#[test]
fn planned_query_execution_parallelized_traversal_matches_serial_reference() {
    fn build_runtime(
        execution_model: crate::facade::runtime::RelationalExecutionModel,
    ) -> RelationalRuntime {
        RelationalRuntimeApi::builder()
            .profile(RelationalRuntimeProfile::CertificationCore)
            .schema_registry(declared_aspect_schema_registry(
                CascadeDeletePolicy::CascadeDeleteRelations,
            ))
            .execution_model(execution_model)
            .build()
    }

    fn build_fixture(runtime: &RelationalRuntime) -> (SnapshotHandle, PlannedQueryPacket) {
        let seeds = vec![
            create_entity_in_partition(runtime, "s0", PartitionId(7)),
            create_entity_in_partition(runtime, "s1", PartitionId(11)),
            create_entity_in_partition(runtime, "s2", PartitionId(13)),
            create_entity_in_partition(runtime, "s3", PartitionId(17)),
            create_entity_in_partition(runtime, "s4", PartitionId(19)),
        ];
        let neighbors = [
            create_entity_in_partition(runtime, "n0", PartitionId(23)),
            create_entity_in_partition(runtime, "n1", PartitionId(29)),
            create_entity_in_partition(runtime, "n2", PartitionId(31)),
            create_entity_in_partition(runtime, "n3", PartitionId(37)),
            create_entity_in_partition(runtime, "n4", PartitionId(41)),
        ];
        for (index, (seed, neighbor)) in seeds.iter().zip(neighbors.iter()).enumerate() {
            create_relation_in_partition(
                runtime,
                *seed,
                *neighbor,
                &format!("edge-{index}"),
                PartitionId(43 + index as u32),
            );
        }
        let snapshot = runtime.visibility_authority().snapshot();
        let context = runtime
            .read_truth()
            .query_plan_context(&snapshot)
            .expect("query plan context");
        let packet = PlannedQueryPacket {
            label: "parallel-traversal-parity".to_string(),
            context_id: context,
            scope: QueryScope::OutgoingNeighborhood {
                seeds: Arc::from(seeds),
                relation_kind_scope: Some(Arc::from([KindId(2)])),
            },
            locality: QueryLocalityClass::CrossPartitionTraversal,
            ordering: QueryOrderingContract::CanonicalTraversalOrder,
            access_contract: QueryAccessContract::AuthoritativeStorageOnly,
            execution_shape: QueryExecutionShape::BulkPacketized,
            reduction: ReductionDiscipline::DeterministicMerge,
            plan_key: DeterministicQueryPlanKey(1002),
            target_count_hint: 5,
        };
        (snapshot, packet)
    }

    let serial_runtime =
        build_runtime(crate::facade::runtime::RelationalExecutionModel::SingleLaneExecution);
    let (serial_snapshot, serial_packet) = build_fixture(&serial_runtime);
    let serial = serial_runtime
        .read_truth()
        .execute_query_plan(
            serial_runtime
                .read_truth()
                .plan_query_packet(&serial_snapshot, serial_packet)
                .expect("serial query plan"),
        )
        .expect("serial execution");

    let staged_runtime =
        build_runtime(crate::facade::runtime::RelationalExecutionModel::ParallelPreparation);
    let (staged_snapshot, staged_packet) = build_fixture(&staged_runtime);
    let staged = staged_runtime
        .read_truth()
        .execute_query_plan(
            staged_runtime
                .read_truth()
                .plan_query_packet(&staged_snapshot, staged_packet)
                .expect("staged query plan"),
        )
        .expect("staged execution");

    assert_eq!(serial.result, staged.result);
    assert_eq!(
        serial.complexity.target_count,
        staged.complexity.target_count
    );
    assert_eq!(
        serial
            .result
            .entities
            .iter()
            .map(|record| read_entity_name(record).unwrap().to_string())
            .collect::<Vec<_>>(),
        staged
            .result
            .entities
            .iter()
            .map(|record| read_entity_name(record).unwrap().to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn planned_query_execution_reports_workload_derived_scratch_reuse_consistently_across_execution_models(
) {
    fn build_runtime(
        execution_model: crate::facade::runtime::RelationalExecutionModel,
    ) -> RelationalRuntime {
        RelationalRuntimeApi::builder()
            .profile(RelationalRuntimeProfile::CertificationCore)
            .schema_registry(declared_aspect_schema_registry(
                CascadeDeletePolicy::CascadeDeleteRelations,
            ))
            .execution_model(execution_model)
            .build()
    }

    fn build_fixture(runtime: &RelationalRuntime) -> (SnapshotHandle, PlannedQueryPacket) {
        let seeds = vec![
            create_entity_in_partition(runtime, "s0", PartitionId(7)),
            create_entity_in_partition(runtime, "s1", PartitionId(11)),
            create_entity_in_partition(runtime, "s2", PartitionId(13)),
            create_entity_in_partition(runtime, "s3", PartitionId(17)),
        ];
        let neighbors = [
            create_entity_in_partition(runtime, "n0", PartitionId(19)),
            create_entity_in_partition(runtime, "n1", PartitionId(23)),
            create_entity_in_partition(runtime, "n2", PartitionId(29)),
            create_entity_in_partition(runtime, "n3", PartitionId(31)),
        ];
        for (index, (seed, neighbor)) in seeds.iter().zip(neighbors.iter()).enumerate() {
            create_relation_in_partition(
                runtime,
                *seed,
                *neighbor,
                &format!("edge-{index}"),
                PartitionId(41 + index as u32),
            );
        }
        let snapshot = runtime.visibility_authority().snapshot();
        let context = runtime
            .read_truth()
            .query_plan_context(&snapshot)
            .expect("query plan context");
        (
            snapshot,
            PlannedQueryPacket {
                label: "scratch-reuse-parity".to_string(),
                context_id: context,
                scope: QueryScope::OutgoingNeighborhood {
                    seeds: Arc::from(seeds),
                    relation_kind_scope: Some(Arc::from([KindId(2)])),
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(1017),
                target_count_hint: 4,
            },
        )
    }

    let serial_runtime =
        build_runtime(crate::facade::runtime::RelationalExecutionModel::SingleLaneExecution);
    let (serial_snapshot, serial_packet) = build_fixture(&serial_runtime);
    serial_runtime.performance_access().reset_counters();
    let serial = serial_runtime
        .read_truth()
        .execute_query_plan(
            serial_runtime
                .read_truth()
                .plan_query_packet(&serial_snapshot, serial_packet)
                .expect("serial query plan"),
        )
        .expect("serial execution");
    let serial_counters = serial_runtime.performance_access().counters();

    let staged_runtime =
        build_runtime(crate::facade::runtime::RelationalExecutionModel::ParallelPreparation);
    let (staged_snapshot, staged_packet) = build_fixture(&staged_runtime);
    staged_runtime.performance_access().reset_counters();
    let staged = staged_runtime
        .read_truth()
        .execute_query_plan(
            staged_runtime
                .read_truth()
                .plan_query_packet(&staged_snapshot, staged_packet)
                .expect("staged query plan"),
        )
        .expect("staged execution");
    let staged_counters = staged_runtime.performance_access().counters();

    assert_eq!(serial.result, staged.result);
    assert_eq!(
        serial_counters.query_fragment_scratch_reuse_count,
        staged_counters.query_fragment_scratch_reuse_count
    );
}

#[test]
fn planned_query_execution_parallelized_overlapping_seed_traversal_dedupes_and_matches_serial() {
    fn build_runtime(
        execution_model: crate::facade::runtime::RelationalExecutionModel,
    ) -> RelationalRuntime {
        RelationalRuntimeApi::builder()
            .profile(RelationalRuntimeProfile::CertificationCore)
            .schema_registry(declared_aspect_schema_registry(
                CascadeDeletePolicy::CascadeDeleteRelations,
            ))
            .execution_model(execution_model)
            .build()
    }

    fn build_fixture(runtime: &RelationalRuntime) -> (SnapshotHandle, PlannedQueryPacket) {
        let seed_a = create_entity_in_partition(runtime, "seed-a", PartitionId(7));
        let seed_b = create_entity_in_partition(runtime, "seed-b", PartitionId(11));
        let shared = create_entity_in_partition(runtime, "shared", PartitionId(13));
        let tail = create_entity_in_partition(runtime, "tail", PartitionId(17));
        create_relation_in_partition(runtime, seed_a, shared, "a-shared", PartitionId(23));
        create_relation_in_partition(runtime, seed_b, shared, "b-shared", PartitionId(29));
        create_relation_in_partition(runtime, shared, tail, "shared-tail", PartitionId(31));
        let snapshot = runtime.visibility_authority().snapshot();
        let context = runtime
            .read_truth()
            .query_plan_context(&snapshot)
            .expect("query plan context");
        (
            snapshot,
            PlannedQueryPacket {
                label: "overlap-traversal".to_string(),
                context_id: context,
                scope: QueryScope::ConnectivityTraversal {
                    seeds: Arc::from([seed_a, seed_b]),
                    relation_kind_scope: Some(Arc::from([KindId(2)])),
                    max_depth: Some(2),
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(1003),
                target_count_hint: 2,
            },
        )
    }

    let serial_runtime =
        build_runtime(crate::facade::runtime::RelationalExecutionModel::SingleLaneExecution);
    let (serial_snapshot, serial_packet) = build_fixture(&serial_runtime);
    let serial = serial_runtime
        .read_truth()
        .execute_query_plan(
            serial_runtime
                .read_truth()
                .plan_query_packet(&serial_snapshot, serial_packet)
                .expect("serial plan"),
        )
        .expect("serial outcome");

    let staged_runtime =
        build_runtime(crate::facade::runtime::RelationalExecutionModel::ParallelPreparation);
    let (staged_snapshot, staged_packet) = build_fixture(&staged_runtime);
    let staged = staged_runtime
        .read_truth()
        .execute_query_plan(
            staged_runtime
                .read_truth()
                .plan_query_packet(&staged_snapshot, staged_packet)
                .expect("staged plan"),
        )
        .expect("staged outcome");

    assert_eq!(serial.result, staged.result);
    assert_eq!(
        staged
            .result
            .entities
            .iter()
            .map(|record| record.entity_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        staged.result.entities.len()
    );
    assert_eq!(
        staged
            .result
            .relations
            .iter()
            .map(|record| record.relation_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        staged.result.relations.len()
    );
}
