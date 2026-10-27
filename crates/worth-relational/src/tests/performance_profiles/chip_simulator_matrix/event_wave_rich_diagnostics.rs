use super::*;

pub(super) fn certify_event_wave_compile_churn_rich_diagnostics(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let event_wave_rich_diagnostics_samples =
        capture_perf_samples(suite, "event_wave_compile_churn_rich_diagnostics", || {
            let runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::ChipSimulation);
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();
            let source = create_entity_in_partition(
                execution,
                &runtime,
                "event-driver-rich",
                PartitionId(7),
            );
            let sinks = (0..16)
                .map(|index| {
                    let partition_id = match index % 4 {
                        0 => PartitionId(11),
                        1 => PartitionId(13),
                        2 => PartitionId(17),
                        _ => PartitionId(19),
                    };
                    create_entity_in_partition(
                        execution,
                        &runtime,
                        &format!("event-sink-rich-{index}"),
                        partition_id,
                    )
                })
                .collect::<Vec<_>>();
            for (index, sink) in sinks.iter().enumerate() {
                create_relation_in_partition(
                    execution,
                    &runtime,
                    source,
                    *sink,
                    &format!("event-link-rich-{index}"),
                    PartitionId(29),
                );
            }

            const ITERATIONS: usize = 16;
            let mut total_update_micros = 0u128;
            let mut total_compile_micros = 0u128;
            let mut total_adjacency_micros = 0u128;
            let mut max_compile_micros = 0u128;
            let mut max_outgoing_relation_count = 0usize;

            runtime.performance_access().reset_counters();
            for step in 0..ITERATIONS {
                let update_started_at = Instant::now();
                let _ = update_entity(
                    execution,
                    &runtime,
                    source,
                    &format!("event-driver-rich-step-{step}"),
                );
                total_update_micros += update_started_at.elapsed().as_micros();

                let commit = runtime
                    .history()
                    .latest_commit()
                    .expect("chip event-wave rich commit")
                    .clone();
                let compile_started_at = Instant::now();
                let artifact = runtime
                    .compiled_artifacts_authority()
                    .compile_execution_artifact(
                        execution,
                        commit.commit_id,
                        vec![
                            PartitionId(7),
                            PartitionId(11),
                            PartitionId(13),
                            PartitionId(17),
                            PartitionId(19),
                            PartitionId(29),
                        ],
                    )
                    .expect("chip event-wave rich compiled artifact");
                let compile_micros = compile_started_at.elapsed().as_micros();
                total_compile_micros += compile_micros;
                max_compile_micros = max_compile_micros.max(compile_micros);

                let adjacency_started_at = Instant::now();
                let outgoing_relations = runtime
                    .storage_access()
                    .outgoing_relations_for_entity(source, commit.version_id);
                total_adjacency_micros += adjacency_started_at.elapsed().as_micros();
                max_outgoing_relation_count =
                    max_outgoing_relation_count.max(outgoing_relations.len());
                assert_eq!(
                    runtime
                        .compiled_artifacts()
                        .compiled_artifact_authority_status(artifact.artifact_id),
                    CompiledArtifactAuthorityStatus::Authoritative
                );
            }

            let (diagnostic_artifact_count, detailed_trace_entries) =
                fresh_diagnostics_metrics(&runtime, diagnostics_start);
            PerfMeasurement {
                elapsed_micros: total_update_micros + total_compile_micros + total_adjacency_micros,
                metrics: perf_metrics!({
                    "iterations": ITERATIONS,
                    "average_update_micros": total_update_micros / ITERATIONS as u128,
                    "average_compile_micros": total_compile_micros / ITERATIONS as u128,
                    "average_adjacency_micros": total_adjacency_micros / ITERATIONS as u128,
                    "max_compile_micros": max_compile_micros,
                    "max_outgoing_relation_count": max_outgoing_relation_count,
                    "diagnostic_artifact_count": diagnostic_artifact_count,
                    "detailed_trace_entries": detailed_trace_entries,
                    "counters": runtime.performance_access().counters(),
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "event_wave_compile_churn_rich_diagnostics",
        &event_wave_rich_diagnostics_samples,
        &[
            ("average_update_micros", &["average_update_micros"]),
            ("average_compile_micros", &["average_compile_micros"]),
            ("average_adjacency_micros", &["average_adjacency_micros"]),
            ("max_compile_micros", &["max_compile_micros"]),
            (
                "max_outgoing_relation_count",
                &["max_outgoing_relation_count"],
            ),
            ("diagnostic_artifact_count", &["diagnostic_artifact_count"]),
            ("detailed_trace_entries", &["detailed_trace_entries"]),
        ],
    );
    assert!(event_wave_rich_diagnostics_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &event_wave_rich_diagnostics_samples,
        "chip event-wave rich diagnostics should keep compile windows supported while surfacing diagnostic cost clearly",
        |metrics| {
            metrics["iterations"].as_u64() == Some(16)
                && metrics["max_outgoing_relation_count"].as_u64() == Some(16)
                && metrics["max_compile_micros"].as_u64().unwrap_or(0) > 0
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 16
                && metrics["detailed_trace_entries"].as_u64().is_some()
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "bulk_mutation_entity_target_count") == 16
        },
    );
}
