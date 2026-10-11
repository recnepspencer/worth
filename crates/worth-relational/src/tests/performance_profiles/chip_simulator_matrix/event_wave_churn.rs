use super::*;

pub(super) fn certify_event_wave_compile_churn_window(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let event_wave_churn_samples =
        capture_perf_samples(suite, "event_wave_compile_churn_window", || {
            let mut runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::ChipSimulation);
            runtime.configure_diagnostics_for_test(|profile| {
                profile.detailed_traces_enabled = false;
                profile.max_entries_per_artifact = 0;
            });
            let source =
                create_entity_in_partition(execution, &runtime, "event-driver", PartitionId(7));
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
                        &format!("event-sink-{index}"),
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
                    &format!("event-link-{index}"),
                    PartitionId(29),
                );
            }

            const ITERATIONS: usize = 24;
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
                    &format!("event-driver-step-{step}"),
                );
                total_update_micros += update_started_at.elapsed().as_micros();

                let commit = runtime
                    .history()
                    .latest_commit()
                    .expect("chip event-wave commit")
                    .clone();
                let compile_started_at = Instant::now();
                let artifact = runtime
                    .compiled_artifacts_authority()
                    .compile_execution_artifact(
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
                    .expect("chip event-wave compiled artifact");
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

            PerfMeasurement {
                elapsed_micros: total_update_micros + total_compile_micros + total_adjacency_micros,
                metrics: perf_metrics!({
                    "iterations": ITERATIONS,
                    "average_update_micros": total_update_micros / ITERATIONS as u128,
                    "average_compile_micros": total_compile_micros / ITERATIONS as u128,
                    "average_adjacency_micros": total_adjacency_micros / ITERATIONS as u128,
                    "max_compile_micros": max_compile_micros,
                    "max_outgoing_relation_count": max_outgoing_relation_count,
                    "counters": runtime.performance_access().counters(),
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "event_wave_compile_churn_window",
        &event_wave_churn_samples,
        &[
            ("average_update_micros", &["average_update_micros"]),
            ("average_compile_micros", &["average_compile_micros"]),
            ("average_adjacency_micros", &["average_adjacency_micros"]),
            ("max_compile_micros", &["max_compile_micros"]),
            (
                "max_outgoing_relation_count",
                &["max_outgoing_relation_count"],
            ),
        ],
    );
    assert!(event_wave_churn_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &event_wave_churn_samples,
        "chip event-wave churn should keep repeated compile windows supported and bounded under sustained stepping",
        |metrics| {
            metrics["iterations"].as_u64() == Some(24)
                && metrics["max_outgoing_relation_count"].as_u64() == Some(16)
                && metrics["max_compile_micros"].as_u64().unwrap_or(0) > 0
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "bulk_mutation_entity_target_count") == 24
        },
    );
}
