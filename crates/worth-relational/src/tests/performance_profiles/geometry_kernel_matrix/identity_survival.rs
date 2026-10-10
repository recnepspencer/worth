use super::*;

pub(super) fn certify_topology_identity_survival_recovery_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let topology_identity_samples = capture_perf_samples(
        suite,
        "topology_identity_survival_recovery_round_trip",
        || {
            let runtime = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::GeometryKernel,
            );
            let created = create_entity_outcome(&runtime, "topology-source");
            let entity = changed_entities(&created)[0];
            let start_lineage = runtime
                .lineage_access()
                .for_record(entity)
                .expect("initial lineage")
                .lineage_id;

            let update_started_at = Instant::now();
            let replacement = update_entity(&runtime, entity, "topology-source-updated");
            let update_commit_micros = update_started_at.elapsed().as_micros();
            let replaced_entity = changed_entities(&replacement)[0];
            let replacement_lineage = runtime
                .lineage_access()
                .for_record(replaced_entity)
                .expect("replacement lineage")
                .lineage_id;

            runtime.performance_access().reset_counters();
            let resolution_started_at = Instant::now();
            let resolution =
                runtime
                    .lineage_access()
                    .resolve_historical_lineage(HistoricalResolutionRequest {
                        branch_id: BranchId("main".to_string()),
                        lineage_id: start_lineage,
                        boundedness_basis:
                            HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
                    });
            let lineage_resolution_micros = resolution_started_at.elapsed().as_micros();
            let resolution_counters = runtime.performance_access().counters();

            let checkpoint_started_at = Instant::now();
            runtime
                .durability_authority()
                .checkpoint()
                .expect("geometry topology checkpoint");
            let checkpoint_micros = checkpoint_started_at.elapsed().as_micros();

            let plan = runtime.durability().recovery_plan(
                crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
            );
            let mut recovered = persisted_runtime_with_test_schema_profile(
                RelationalRuntimeProfile::GeometryKernel,
            );
            let recover_started_at = Instant::now();
            recovered
                .durability_recovery()
                .recover(execution, plan)
                .expect("geometry topology recovery");
            let recover_micros = recover_started_at.elapsed().as_micros();

            recovered.performance_access().reset_counters();
            let recovered_resolution_started_at = Instant::now();
            let recovered_resolution = recovered.lineage_access().resolve_historical_lineage(
                HistoricalResolutionRequest {
                    branch_id: BranchId("main".to_string()),
                    lineage_id: start_lineage,
                    boundedness_basis:
                        HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
                },
            );
            let recovered_lineage_resolution_micros =
                recovered_resolution_started_at.elapsed().as_micros();
            let recovered_counters = recovered.performance_access().counters();

            PerfMeasurement {
                elapsed_micros: update_commit_micros
                    + lineage_resolution_micros
                    + checkpoint_micros
                    + recover_micros
                    + recovered_lineage_resolution_micros,
                metrics: perf_metrics!({
                    "update_commit_micros": update_commit_micros,
                    "lineage_resolution_micros": lineage_resolution_micros,
                    "checkpoint_micros": checkpoint_micros,
                    "recover_micros": recover_micros,
                    "recovered_lineage_resolution_micros": recovered_lineage_resolution_micros,
                    "resolved_lineage_count": resolution.metrics.resolved_lineage_count,
                    "traversed_event_count": resolution.traversed_event_ids.len(),
                    "replacement_lineage_matches": resolution.resolved == vec![replacement_lineage],
                    "recovered_resolution_matches": recovered_resolution.resolved == resolution.resolved
                        && recovered_resolution.traversed_event_ids == resolution.traversed_event_ids
                        && recovered_resolution.digest_basis() == resolution.digest_basis(),
                    "counters": resolution_counters,
                    "recovered_counters": recovered_counters
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "topology_identity_survival_recovery_round_trip",
        &topology_identity_samples,
        &[
            ("update_commit_micros", &["update_commit_micros"]),
            ("lineage_resolution_micros", &["lineage_resolution_micros"]),
            ("checkpoint_micros", &["checkpoint_micros"]),
            ("recover_micros", &["recover_micros"]),
            (
                "recovered_lineage_resolution_micros",
                &["recovered_lineage_resolution_micros"],
            ),
            ("resolved_lineage_count", &["resolved_lineage_count"]),
            ("traversed_event_count", &["traversed_event_count"]),
        ],
    );
    assert!(topology_identity_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &topology_identity_samples,
        "geometry identity survival should preserve exact lineage truth across recovery",
        |metrics| {
            metrics["replacement_lineage_matches"].as_bool() == Some(true)
                && metrics["recovered_resolution_matches"].as_bool() == Some(true)
                && metrics["resolved_lineage_count"].as_u64() == Some(1)
                && metrics["checkpoint_micros"].as_u64().unwrap_or(0) > 0
                && metrics["recover_micros"].as_u64().unwrap_or(0) > 0
                && metrics["counters"]["lineage_historical_resolution_requests"].as_u64() == Some(1)
                && metrics["recovered_counters"]["lineage_historical_resolution_requests"].as_u64()
                    == Some(1)
        },
    );
}
