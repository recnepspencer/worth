use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

pub(super) fn certify_checkpoint_window_recover_compile_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let checkpoint_recover_compile_samples = capture_perf_samples(
        suite,
        "checkpoint_window_recover_compile_round_trip",
        || {
            let runtime = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::ChipSimulation,
            );
            let source =
                create_entity_in_partition(execution, &runtime, "persisted-driver", PartitionId(7));
            let targets = (0..12)
                .map(|index| {
                    let partition_id = match index % 3 {
                        0 => PartitionId(11),
                        1 => PartitionId(13),
                        _ => PartitionId(17),
                    };
                    create_entity_in_partition(
                        execution,
                        &runtime,
                        &format!("persisted-sink-{index}"),
                        partition_id,
                    )
                })
                .collect::<Vec<_>>();
            for (index, target) in targets.iter().enumerate() {
                create_relation_in_partition(
                    execution,
                    &runtime,
                    source,
                    *target,
                    &format!("persisted-edge-{index}"),
                    PartitionId(29),
                );
            }

            let checkpoint_started_at = Instant::now();
            runtime
                .durability_authority()
                .checkpoint(execution)
                .expect("chip checkpoint window");
            let checkpoint_micros = checkpoint_started_at.elapsed().as_micros();

            let plan = runtime
                .durability()
                .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
                .unwrap();
            let mut recovered = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::ChipSimulation,
            );
            let recover_started_at = Instant::now();
            recovered
                .durability_recovery()
                .recover(execution, plan)
                .expect("chip checkpoint recovery");
            let recover_micros = recover_started_at.elapsed().as_micros();

            let recovered_commit = recovered
                .history()
                .latest_commit()
                .expect("recovered chip commit")
                .clone();
            let compile_started_at = Instant::now();
            let artifact = recovered
                .compiled_artifacts_authority()
                .compile_execution_artifact(
                    recovered_commit.commit_id,
                    vec![
                        PartitionId(7),
                        PartitionId(11),
                        PartitionId(13),
                        PartitionId(17),
                        PartitionId(29),
                    ],
                )
                .expect("recovered chip compiled artifact");
            let compile_micros = compile_started_at.elapsed().as_micros();

            let adjacency_started_at = Instant::now();
            let outgoing_relations = recovered
                .storage_access()
                .outgoing_relations_for_entity(source, recovered_commit.version_id);
            let adjacency_micros = adjacency_started_at.elapsed().as_micros();

            PerfMeasurement {
                elapsed_micros: checkpoint_micros
                    + recover_micros
                    + compile_micros
                    + adjacency_micros,
                metrics: perf_metrics!({
                    "checkpoint_micros": checkpoint_micros,
                    "recover_micros": recover_micros,
                    "compile_micros": compile_micros,
                    "adjacency_micros": adjacency_micros,
                    "recovered_segment_count": recovered
                        .durability()
                        .durable_log()
                        .len(),
                    "outgoing_relation_count": outgoing_relations.len(),
                    "compiled_artifact_authority_status": format!(
                        "{:?}",
                        recovered
                            .compiled_artifacts()
                            .compiled_artifact_authority_status(artifact.artifact_id)
                    ),
                    "counters": recovered.performance_access().counters(),
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "checkpoint_window_recover_compile_round_trip",
        &checkpoint_recover_compile_samples,
        &[
            ("checkpoint_micros", &["checkpoint_micros"]),
            ("recover_micros", &["recover_micros"]),
            ("compile_micros", &["compile_micros"]),
            ("adjacency_micros", &["adjacency_micros"]),
            ("recovered_segment_count", &["recovered_segment_count"]),
            ("outgoing_relation_count", &["outgoing_relation_count"]),
        ],
    );
    assert!(checkpoint_recover_compile_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &checkpoint_recover_compile_samples,
        "chip checkpoint window recovery should preserve compile-ready fanout truth after recovery",
        |metrics| {
            metrics["checkpoint_micros"].as_u64().unwrap_or(0) > 0
                && metrics["recover_micros"].as_u64().unwrap_or(0) > 0
                && metrics["outgoing_relation_count"].as_u64() == Some(12)
                && metrics["compiled_artifact_authority_status"].as_str()
                    == Some(&format!(
                        "{:?}",
                        CompiledArtifactAuthorityStatus::Authoritative
                    ))
                && counter_u64(metrics, "full_state_clones") == 0
        },
    );
}
