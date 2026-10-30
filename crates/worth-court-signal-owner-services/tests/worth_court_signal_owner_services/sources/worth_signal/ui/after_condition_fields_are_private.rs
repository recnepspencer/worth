use worth_signal::facade::{AfterCondition, ClockDomain, TemporalDuration};

fn main() {
    let _condition = AfterCondition {
        delay: TemporalDuration::temporal_duration(5).unwrap(),
        clock_domain: ClockDomain::MonotonicExecution,
    };
}
