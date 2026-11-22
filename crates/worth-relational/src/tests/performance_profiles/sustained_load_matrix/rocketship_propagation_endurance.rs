use super::*;

pub(super) fn certify_rocketship_propagation_endurance(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let rocketship_endurance_node_count = rocketship_node_count();
    let rocketship_propagation_endurance_samples = capture_counted_samples(|| {
        let query_target_count = rocketship_query_target_count(rocketship_endurance_node_count);
        let mut runtime = runtime_with_extra_entity_aspects_profile_and_chunks(
            RelationalRuntimeProfile::GeometryKernel,
            ROCKETSHIP_CHUNK_SIZE,
            ROCKETSHIP_CHUNK_SIZE,
            vec![
                crate::tests::support::entity_field_aspect(
                    crate::tests::support::aspect_key("section"),
                    crate::tests::support::field_key("section"),
                ),
                crate::tests::support::entity_field_aspect(
                    crate::tests::support::aspect_key("subsystem"),
                    crate::tests::support::field_key("subsystem"),
                ),
                crate::tests::support::entity_field_aspect(
                    crate::tests::support::aspect_key("aspect"),
                    crate::tests::support::field_key("aspect"),
                ),
                crate::tests::support::entity_u64_field_aspect(
                    crate::tests::support::aspect_key("ordinal"),
                    crate::tests::support::field_key("ordinal"),
                ),
            ],
        );
        apply_perf_diagnostics_policy(
            &mut runtime,
            PerfDiagnosticsPolicy::GeometryOperationalHotPath,
        );
        let seeded = seed_pseudorealistic_rocketship_world(
            execution,
            &runtime,
            rocketship_endurance_node_count,
            query_target_count,
        );

        const ITERATIONS: usize = 96;
        const WINDOW: usize = 16;
        let mut cycle_samples = Vec::with_capacity(ITERATIONS);
        let mut total_update_work = 0u64;
        let mut total_propagation_work = 0u64;
        let mut total_explicit_query_work = 0u64;
        let mut max_packets_per_iteration = 0usize;
        let mut max_scope_units_per_iteration = 0usize;
        let mut previous_packets = 0usize;
        let mut previous_scope_units = 0usize;

        runtime.performance_access().reset_counters();
        for index in 0..ITERATIONS {
            let target = seeded.traversal_seeds[index % seeded.traversal_seeds.len()];
            let (_, update_work) = measure_counted_work(execution, || {
                update_entity(
                    execution,
                    &runtime,
                    target,
                    &format!("rocket.endurance.propagation.{index}"),
                )
            });
            total_update_work += update_work;

            let snapshot = runtime
                .visibility_authority()
                .snapshot()
                .expect("current snapshot fixture is admitted");
            let context = runtime
                .read_truth()
                .query_plan_context(&snapshot)
                .expect("rocketship endurance propagation context");
            let propagation_seeds = vec![
                seeded.traversal_seeds[index % seeded.traversal_seeds.len()],
                seeded.traversal_seeds[(index + 1) % seeded.traversal_seeds.len()],
                seeded.traversal_seeds[(index + 9) % seeded.traversal_seeds.len()],
                seeded.traversal_seeds[(index + 10) % seeded.traversal_seeds.len()],
            ];
            let propagation_packet = PlannedQueryPacket {
                label: "rocketship-endurance-propagation".to_string(),
                context_id: context,
                scope: QueryScope::ConnectivityTraversal {
                    seeds: Arc::from(propagation_seeds),
                    relation_kind_scope: Some(Arc::from([KindId(2)])),
                    max_depth: Some(3),
                },
                locality: QueryLocalityClass::CrossPartitionTraversal,
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                access_contract: QueryAccessContract::AuthoritativeStorageOnly,
                execution_shape: QueryExecutionShape::BulkPacketized,
                reduction: ReductionDiscipline::DeterministicMerge,
                plan_key: DeterministicQueryPlanKey(92_250),
                target_count_hint: 4,
            };
            let (_, propagation_work) = measure_counted_work(execution, || {
                runtime
                    .read_truth()
                    .execute_query_plan(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, propagation_packet)
                            .expect("planned rocketship endurance propagation query"),
                        execution,
                    )
                    .expect("request remains live")
                    .map(|read| read.outcome)
                    .expect("rocketship endurance propagation outcome")
            });
            total_propagation_work += propagation_work;

            let explicit_targets = seeded
                .mixed_query_targets
                .iter()
                .cycle()
                .skip(index)
                .take(12)
                .cloned()
                .collect::<Vec<_>>();
            let explicit_packet = explicit_query_packet(
                &runtime,
                &snapshot,
                "rocketship-endurance-explicit-broad",
                explicit_targets,
            );
            let (_, explicit_query_work) = measure_counted_work(execution, || {
                runtime
                    .read_truth()
                    .execute_query_plan(
                        runtime
                            .read_truth()
                            .plan_query_packet(&snapshot, explicit_packet)
                            .expect("planned rocketship endurance explicit broad query"),
                        execution,
                    )
                    .expect("request remains live")
                    .map(|read| read.outcome)
                    .expect("rocketship endurance explicit broad outcome")
            });
            total_explicit_query_work += explicit_query_work;
            assert!(runtime
                .visibility_authority()
                .release_snapshot(&snapshot)
                .is_ok());

            cycle_samples.push(update_work + propagation_work + explicit_query_work);

            let counters = runtime.performance_access().counters();
            max_packets_per_iteration = max_packets_per_iteration
                .max(counters.query_packet_count.saturating_sub(previous_packets));
            max_scope_units_per_iteration = max_scope_units_per_iteration.max(
                counters
                    .query_scope_unit_count
                    .saturating_sub(previous_scope_units),
            );
            previous_packets = counters.query_packet_count;
            previous_scope_units = counters.query_scope_unit_count;
        }

        let first_window_average_cycle_work =
            cycle_samples.iter().take(WINDOW).copied().sum::<u64>() / WINDOW as u64;
        let last_window_average_cycle_work = cycle_samples
            .iter()
            .rev()
            .take(WINDOW)
            .copied()
            .sum::<u64>()
            / WINDOW as u64;
        let charged_work =
            total_update_work + total_propagation_work + total_explicit_query_work;
        CountedPerfSample {
            charged_work,
            metrics: perf_metrics!({
                "iterations": ITERATIONS,
                "resident_node_count": seeded.entities.len(),
                "resident_relation_count": seeded.relation_count,
                "average_update_work": total_update_work / ITERATIONS as u64,
                "average_propagation_work": total_propagation_work / ITERATIONS as u64,
                "average_explicit_query_work": total_explicit_query_work / ITERATIONS as u64,
                "first_window_average_cycle_work": first_window_average_cycle_work,
                "last_window_average_cycle_work": last_window_average_cycle_work,
                "max_packets_per_iteration": max_packets_per_iteration,
                "max_scope_units_per_iteration": max_scope_units_per_iteration,
                "counters": runtime.performance_access().counters(),
            }),
        }
    });
    assert!(rocketship_propagation_endurance_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &rocketship_propagation_endurance_samples,
        "rocketship propagation endurance should keep broad-wave cycles bounded across extended 100k-node operation",
        |metrics| {
            let first_window = metrics["first_window_average_cycle_work"]
                .as_u64()
                .unwrap_or(0);
            let last_window = metrics["last_window_average_cycle_work"]
                .as_u64()
                .unwrap_or(0);
            metrics["iterations"].as_u64() == Some(96)
                && metrics["resident_node_count"].as_u64()
                    == Some(rocketship_endurance_node_count as u64)
                && metrics["resident_relation_count"].as_u64().unwrap_or(0)
                    >= rocketship_endurance_node_count as u64
                && first_window > 0
                && last_window <= first_window.saturating_mul(2)
                && metrics["max_packets_per_iteration"].as_u64().unwrap_or(0) <= 24
                && metrics["max_scope_units_per_iteration"].as_u64().unwrap_or(0) <= 24
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "bulk_mutation_batch_count") == 96
        },
    );
}
