use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

pub(super) fn certify_topology_identity_survival_recovery_round_trip(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let topology_identity_samples = capture_counted_samples(|| {
        let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
        let created = create_entity_outcome(execution, &runtime, "topology-source");
        let entity = changed_entities(&created)[0];
        let start_lineage = runtime
            .lineage_access()
            .for_record(execution, entity)
            .unwrap()
            .expect("initial lineage")
            .lineage_id;

        let (replacement, update_commit_work) = measure_counted_work(execution, || {
            update_entity(execution, &runtime, entity, "topology-source-updated")
        });
        let replaced_entity = changed_entities(&replacement)[0];
        let replacement_lineage = runtime
            .lineage_access()
            .for_record(execution, replaced_entity)
            .unwrap()
            .expect("replacement lineage")
            .lineage_id;

        runtime.performance_access().reset_counters();
        let (resolution, lineage_resolution_work) = measure_counted_work(execution, || {
            runtime
                .lineage_access()
                .resolve_historical_lineage(
                    execution,
                    HistoricalResolutionRequest {
                        branch_id: BranchId("main".to_string()),
                        lineage_id: start_lineage,
                        boundedness_basis:
                            HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
                    },
                )
                .unwrap()
        });
        let resolution_counters = runtime.performance_access().counters();

        let (_, checkpoint_work) = measure_counted_work(execution, || {
            runtime
                .durability_authority()
                .checkpoint(execution)
                .expect("geometry topology checkpoint")
        });

        let plan = runtime
            .durability()
            .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
            .unwrap();
        let mut recovered =
            runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
        let (recovery_outcome, recover_work) = measure_counted_work(execution, || {
            recovered
                .durability_recovery()
                .recover(execution, plan)
                .expect("geometry topology recovery")
        });

        recovered.performance_access().reset_counters();
        let (recovered_resolution, recovered_lineage_resolution_work) =
            measure_counted_work(execution, || {
                recovered
                    .lineage_access()
                    .resolve_historical_lineage(
                        execution,
                        HistoricalResolutionRequest {
                            branch_id: BranchId("main".to_string()),
                            lineage_id: start_lineage,
                            boundedness_basis:
                                HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
                        },
                    )
                    .unwrap()
            });
        let recovered_counters = recovered.performance_access().counters();

        CountedPerfSample {
            charged_work: update_commit_work
                + lineage_resolution_work
                + checkpoint_work
                + recover_work
                + recovered_lineage_resolution_work,
            metrics: perf_metrics!({
                "update_commit_work": update_commit_work,
                "lineage_resolution_work": lineage_resolution_work,
                "checkpoint_work": checkpoint_work,
                "recover_work": recover_work,
                "recovered_lineage_resolution_work": recovered_lineage_resolution_work,
                "recovered_commits": recovery_outcome.recovered_commits,
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
    });
    assert!(topology_identity_samples
        .iter()
        .all(|sample| sample.charged_work > 0));
    assert_counted_budget(
        &topology_identity_samples,
        "geometry identity survival should preserve exact lineage truth across recovery",
        |metrics| {
            metrics["replacement_lineage_matches"].as_bool() == Some(true)
                && metrics["recovered_resolution_matches"].as_bool() == Some(true)
                && metrics["resolved_lineage_count"].as_u64() == Some(1)
                && metrics["recovered_commits"].as_u64().unwrap_or(0) >= 1
                && metrics["counters"]["lineage_historical_resolution_requests"].as_u64() == Some(1)
                && metrics["recovered_counters"]["lineage_historical_resolution_requests"].as_u64()
                    == Some(1)
        },
    );
}
