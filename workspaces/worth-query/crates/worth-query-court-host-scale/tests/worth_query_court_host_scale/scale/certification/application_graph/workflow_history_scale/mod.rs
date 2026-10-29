pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_history_scale::*;

#[test]
#[ignore = "bounded diagnostic of native history-copy cost, not scale qualification"]
fn workflow_history_native_cost_diagnostic() {
    let mut court = HistoryCourt::new();
    for history in [100, 200, 400] {
        court.fill(history);
        court.measure_warm(1);
    }
}

#[test]
#[ignore = "scheduled history qualification; run query-workflow-history-scale"]
fn workflow_history_and_instance_scale() {
    let mut court = HistoryCourt::new();
    for history in [100, 1_000, 10_000] {
        court.fill(history);
        court.measure_warm(1);
    }
    court.reconstruct();
    drop(court);
    for instances in [1, 10, 100] {
        let mut court = HistoryCourt::new();
        court.fill(100);
        for _ in 1..instances {
            court.unrelated_instance();
        }
        court.measure_warm(instances);
    }
}
