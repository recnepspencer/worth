use super::*;

pub(super) fn certify_event_wave_compile_churn_window(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let event_wave_churn_samples = capture_counted_samples(|| {
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
        let mut total_update_work = 0u64;
        let mut total_compile_work = 0u64;
        let mut total_adjacency_work = 0u64;
        let mut max_compiled_record_count = 0usize;
        let mut max_outgoing_relation_count = 0usize;

        runtime.performance_access().reset_counters();
        for step in 0..ITERATIONS {
            let (_, update_work) = measure_counted_work(execution, || {
                update_entity(
                    execution,
                    &runtime,
                    source,
                    &format!("event-driver-step-{step}"),
                )
            });
            total_update_work += update_work;

            let commit = runtime
                .history()
                .latest_commit()
                .expect("chip event-wave commit")
                .clone();
            let (artifact, compile_work) = measure_counted_work(execution, || {
                runtime
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
                    .expect("chip event-wave compiled artifact")
            });
            total_compile_work += compile_work;
            max_compiled_record_count =
                max_compiled_record_count.max(artifact.compiled_record_count);

            let (outgoing_relations, adjacency_work) = measure_counted_work(execution, || {
                runtime
                    .storage_access()
                    .outgoing_relations_for_entity(execution, source, commit.version_id)
                    .expect("funded adjacency read succeeds")
            });
            total_adjacency_work += adjacency_work;
            max_outgoing_relation_count =
                max_outgoing_relation_count.max(outgoing_relations.len());
            assert_eq!(
                runtime
                    .compiled_artifacts()
                    .compiled_artifact_authority_status(artifact.artifact_id),
                CompiledArtifactAuthorityStatus::Authoritative
            );
        }

        CountedPerfSample {
            charged_work: total_update_work + total_compile_work + total_adjacency_work,
            metrics: perf_metrics!({
                "iterations": ITERATIONS,
                "total_update_work": total_update_work,
                "total_compile_work": total_compile_work,
                "total_adjacency_work": total_adjacency_work,
                "max_compiled_record_count": max_compiled_record_count,
                "max_outgoing_relation_count": max_outgoing_relation_count,
                "counters": runtime.performance_access().counters(),
            }),
        }
    });
    assert!(event_wave_churn_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &event_wave_churn_samples,
        "chip event-wave churn should keep repeated compile windows supported and bounded under sustained stepping",
        |metrics| {
            metrics["iterations"].as_u64() == Some(24)
                && metrics["max_outgoing_relation_count"].as_u64() == Some(16)
                && metrics["max_compiled_record_count"].as_u64().unwrap_or(0) > 0
                && metrics["total_update_work"].as_u64().unwrap_or(0) > 0
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "bulk_mutation_entity_target_count") == 24
        },
    );
}
