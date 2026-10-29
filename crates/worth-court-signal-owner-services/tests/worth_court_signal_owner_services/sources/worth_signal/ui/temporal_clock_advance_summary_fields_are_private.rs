use worth_signal::facade::{
    ClockAdvanceRequest, ClockDomain, ClockTick, SignalGraph, SignalRuntime,
    TemporalClockAdvanceSummary,
};

fn main() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let summary = runtime
        .advance_clock_with_summary(ClockAdvanceRequest::new(
            ClockDomain::MonotonicExecution,
            ClockTick::new(5),
        ))
        .unwrap();

    let _forged = TemporalClockAdvanceSummary {
        validated_advance: summary.validated_advance(),
        frontier_before: summary.frontier_before(),
        frontier_after: summary.frontier_after(),
        promoted_wake_count: summary.promoted_wake_count(),
        ready_selection_deferred: summary.ready_selection_deferred(),
    };
}
