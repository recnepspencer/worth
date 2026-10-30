use worth_signal::facade::{
    ClockTick, SignalGraph, SignalRuntime, TemporalCondition, TemporalWakeAdmissionSummary,
};

fn main() {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let wake = runtime
        .schedule_temporal_wake(TemporalCondition::after(5).unwrap(), ClockTick::new(5))
        .unwrap();

    let _forged = TemporalWakeAdmissionSummary {
        scheduled: vec![wake],
        rescheduled: Vec::new(),
        reused: Vec::new(),
    };
}
