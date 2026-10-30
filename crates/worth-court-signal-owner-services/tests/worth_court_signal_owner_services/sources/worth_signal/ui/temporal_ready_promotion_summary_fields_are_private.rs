use worth_signal::facade::{
    ClockAdvanceRequest, ClockDomain, ClockTick, SignalGraph, SignalRuntime, TemporalCondition,
    TemporalReadyPromotionSummary,
};

fn main() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime
        .schedule_temporal_wake(TemporalCondition::after(5).unwrap(), ClockTick::new(5))
        .unwrap();
    runtime
        .advance_clock(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(5),
        ))
        .unwrap();
    let summary = runtime
        .promote_due_temporal_wakes_ready_with_summary()
        .unwrap();

    let _forged = TemporalReadyPromotionSummary {
        frontier_before: summary.frontier_before(),
        frontier_after: summary.frontier_after(),
        ready_wakes: summary.ready_wakes().to_vec(),
        promoted_wake_count: summary.promoted_wake_count(),
        broad_scan_denial_count_delta: summary.broad_scan_denial_count_delta(),
    };
}
