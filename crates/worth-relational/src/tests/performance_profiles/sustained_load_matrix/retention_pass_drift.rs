use super::*;

pub(super) fn certify_retention_pass_drift_stability(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    suite: &'static str,
) {
    let retention_pass_drift_samples =
        capture_perf_samples(suite, "retention_pass_drift_stability", || {
            let runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::CertificationCore);
            const ITERATIONS: usize = 48;
            let mut total_inspect_micros = 0u128;
            let mut total_run_pass_micros = 0u128;
            let mut total_entity_reclaimable = 0usize;
            let mut total_entity_reclaimed = 0usize;
            let mut max_reclaimable_entities = 0usize;

            runtime.performance_access().reset_counters();
            for index in 0..ITERATIONS {
                let created =
                    create_entity_outcome(execution, &runtime, &format!("retention-drift-{index}"));
                let entity = changed_entities(&created)[0];
                let deleted = delete_entity(execution, &runtime, entity);
                assert!(runtime
                    .visibility_authority()
                    .release_snapshot(&created.snapshot)
                    .is_ok());
                assert!(runtime
                    .visibility_authority()
                    .release_snapshot(&deleted.snapshot)
                    .is_ok());

                let inspect_started_at = Instant::now();
                let plan = runtime.retention().inspect_plan();
                total_inspect_micros += inspect_started_at.elapsed().as_micros();
                max_reclaimable_entities = max_reclaimable_entities.max(plan.reclaimable_entities);

                let run_pass_started_at = Instant::now();
                let pass = runtime.retention().run_pass();
                total_run_pass_micros += run_pass_started_at.elapsed().as_micros();
                total_entity_reclaimable += pass.entity_reclaimable;
                total_entity_reclaimed += pass.entity_reclaimed;
            }

            let trailing_plan = runtime.retention().inspect_plan();
            let elapsed_micros = total_inspect_micros + total_run_pass_micros;
            measurement_with_elapsed(elapsed_micros, || {
                perf_metrics!({
                    "iterations": ITERATIONS,
                    "average_inspect_micros": total_inspect_micros / ITERATIONS as u128,
                    "average_run_pass_micros": total_run_pass_micros / ITERATIONS as u128,
                    "total_entity_reclaimable": total_entity_reclaimable,
                    "total_entity_reclaimed": total_entity_reclaimed,
                    "max_reclaimable_entities": max_reclaimable_entities,
                    "trailing_reclaimable_entities": trailing_plan.reclaimable_entities,
                    "counters": runtime.performance_access().counters(),
                })
            })
        });
    emit_metric_summaries(
        suite,
        "retention_pass_drift_stability",
        &retention_pass_drift_samples,
        &[
            ("average_inspect_micros", &["average_inspect_micros"]),
            ("average_run_pass_micros", &["average_run_pass_micros"]),
            ("total_entity_reclaimable", &["total_entity_reclaimable"]),
            ("total_entity_reclaimed", &["total_entity_reclaimed"]),
            ("max_reclaimable_entities", &["max_reclaimable_entities"]),
        ],
    );
    assert!(retention_pass_drift_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &retention_pass_drift_samples,
        "retention drift windows should surface reclaimable released deletions without rebuild-heavy retention behavior",
        |metrics| {
            metrics["iterations"].as_u64() == Some(48)
                && metrics["total_entity_reclaimable"].as_u64().unwrap_or(0) >= 48
                && metrics["total_entity_reclaimed"].as_u64() == Some(0)
                && metrics["trailing_reclaimable_entities"].as_u64() == Some(48)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "snapshot_pin_full_rebuilds") == 0
        },
    );
}
