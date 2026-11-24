use super::*;
use worth_relational_contracts::internal_port::lineage::data::graph::LineageNodeFieldsPort as _;

pub(super) fn certify_lineage_branch_divergence_breadth(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let lineage_divergence_samples = capture_counted_samples(|| {
        let runtime = runtime_with_test_schema();
        let created = create_entity_outcome(execution, &runtime, "main");
        let start_lineage = *runtime
            .lineage_access()
            .for_record(execution, changed_entities(&created)[0])
            .unwrap()
            .expect("start lineage")
            .owner_lineage_id();
        create_branch_from_main(&runtime, "feature");
        let _ = create_entity_outcome_on_branch(
            execution,
            &runtime,
            "feature",
            BranchId("feature".to_string()),
        );

        runtime.performance_access().reset_counters();
        let ((divergence, resolution), charged_work) = measure_counted_work(execution, || {
            let divergence = runtime
                .lineage_access()
                .divergence_between_branches(
                    execution,
                    LineageDivergenceRequest {
                        left_branch: BranchId("main".to_string()),
                        right_branch: BranchId("feature".to_string()),
                        traversal_basis: LineageDivergenceTraversalBasis::FullBranchGraphComparison,
                    },
                )
                .unwrap();
            let resolution = runtime
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
                .unwrap();
            (divergence, resolution)
        });
        let counters = runtime.performance_access().counters();

        CountedPerfSample {
            charged_work,
            metrics: perf_metrics!({
                "charged_work": charged_work,
                "left_event_count": divergence.metrics().left_event_count,
                "right_event_count": divergence.metrics().right_event_count,
                "left_node_count": divergence.metrics().left_node_count,
                "right_node_count": divergence.metrics().right_node_count,
                "resolution_event_scans": resolution.metrics.event_visit_count,
                "resolution_traversed_events": resolution.metrics.traversed_event_count,
                "counters": counters,
            }),
        }
    });
    assert_counted_budget(
        &lineage_divergence_samples,
        "lineage divergence and branch-scoped resolution should report their true breadths",
        |metrics| {
            counter_u64(metrics, "lineage_branch_divergence_requests") == 1
                && counter_u64(metrics, "lineage_historical_resolution_requests") == 1
                && counter_u64(metrics, "lineage_branch_divergence_event_scans")
                    == metrics["left_event_count"].as_u64().unwrap_or(0)
                        + metrics["right_event_count"].as_u64().unwrap_or(0)
                && counter_u64(metrics, "lineage_branch_divergence_node_scans")
                    == metrics["left_node_count"].as_u64().unwrap_or(0)
                        + metrics["right_node_count"].as_u64().unwrap_or(0)
                && counter_u64(metrics, "lineage_historical_resolution_event_visits")
                    == metrics["resolution_event_scans"].as_u64().unwrap_or(0)
                && counter_u64(metrics, "lineage_historical_resolution_traversed_events")
                    == metrics["resolution_traversed_events"].as_u64().unwrap_or(0)
        },
    );
}
