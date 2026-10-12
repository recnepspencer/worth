use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_invariant_materialization_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let suite = "invariant_materialization_matrix";

    let custom_surface_samples =
        capture_perf_samples(suite, "custom_structural_surface_commit_wave", || {
            let runtime = runtime_with_test_schema_profile_and_custom_invariant(
                RelationalRuntimeProfile::CertificationCore,
            );
            let entities = (0..12)
                .map(|index| {
                    create_entity_in_partition(
                        execution,
                        &runtime,
                        &format!("invariant-node-{index}"),
                        PartitionId((index % 4) as u32 + 1),
                    )
                })
                .collect::<Vec<_>>();
            for index in 0..(entities.len() - 1) {
                create_relation_in_partition(
                    execution,
                    &runtime,
                    entities[index],
                    entities[index + 1],
                    &format!("invariant-link-{index}"),
                    PartitionId(20 + (index % 4) as u32),
                );
            }

            runtime.performance_access().reset_counters();
            let started_at = Instant::now();
            let outcome = create_relation_outcome(
                execution,
                &runtime,
                entities[2],
                entities[9],
                "invariant-wave-bridge",
            );
            let elapsed_micros = started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();
            let phase_timing = outcome.execution().phase_timing.clone();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "changed_records": outcome.changed_records.len(),
                    "phase_timing": {
                        "invariant_pre_check_micros": phase_timing.invariant_pre_check_micros,
                        "authoritative_mutation_micros": phase_timing.authoritative_mutation_micros,
                        "invariant_post_check_micros": phase_timing.invariant_post_check_micros,
                    },
                    "counters": counters,
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "custom_structural_surface_commit_wave",
        &custom_surface_samples,
        &[
            (
                "invariant_pre_check_micros",
                &["phase_timing", "invariant_pre_check_micros"],
            ),
            (
                "authoritative_mutation_micros",
                &["phase_timing", "authoritative_mutation_micros"],
            ),
            (
                "invariant_post_check_micros",
                &["phase_timing", "invariant_post_check_micros"],
            ),
        ],
    );
    assert!(custom_surface_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &custom_surface_samples,
        "custom invariant surface waves should execute touched-only invariant work without clone-heavy materialization",
        |metrics| {
            metrics["changed_records"].as_u64() == Some(1)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "custom_invariant_preparation_count") == 1
                && counter_u64(metrics, "custom_invariant_execution_count") == 1
                && counter_u64(metrics, "custom_invariant_panic_count") == 0
                && counter_u64(metrics, "custom_invariant_traversal_frontier_count") >= 1
                && counter_u64(metrics, "custom_invariant_traversal_step_count") >= 1
                && (counter_u64(metrics, "invariant_entity_slot_scans") >= 1
                    || counter_u64(metrics, "invariant_relation_slot_scans") >= 1)
                && counter_u64(metrics, "invariant_entity_slot_scans")
                    + counter_u64(metrics, "invariant_relation_slot_scans")
                    >= 1
        },
    );
}
