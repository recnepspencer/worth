use super::*;

pub(super) fn certify_retention_pass_drift_stability(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let retention_pass_drift_samples = capture_counted_samples(|| {
        let runtime =
            runtime_with_test_schema_profile(RelationalRuntimeProfile::CertificationCore);
        const ITERATIONS: usize = 48;
        let mut total_inspect_work = 0u64;
        let mut total_run_pass_work = 0u64;
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

            let (plan, inspect_work) = measure_counted_work(execution, || {
                runtime.retention().inspect_plan(execution).unwrap()
            });
            total_inspect_work += inspect_work;
            max_reclaimable_entities = max_reclaimable_entities.max(plan.reclaimable_entities);

            let (pass, run_pass_work) = measure_counted_work(execution, || {
                runtime.retention().run_pass(execution).unwrap()
            });
            total_run_pass_work += run_pass_work;
            total_entity_reclaimable += pass.entity_reclaimable;
            total_entity_reclaimed += pass.entity_reclaimed;
        }

        let trailing_plan = runtime.retention().inspect_plan(execution).unwrap();
        CountedPerfSample {
            charged_work: total_inspect_work + total_run_pass_work,
            metrics: perf_metrics!({
                "iterations": ITERATIONS,
                "total_inspect_work": total_inspect_work,
                "total_run_pass_work": total_run_pass_work,
                "total_entity_reclaimable": total_entity_reclaimable,
                "total_entity_reclaimed": total_entity_reclaimed,
                "max_reclaimable_entities": max_reclaimable_entities,
                "trailing_reclaimable_entities": trailing_plan.reclaimable_entities,
                "counters": runtime.performance_access().counters(),
            }),
        }
    });
    assert_counted_budget(
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
