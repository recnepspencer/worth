use worth_signal::facade::{ClockDomain, ReadyTemporalEligibility, TemporalCondition};

fn main() {
    let condition = TemporalCondition::after(50).unwrap();
    let _ = ReadyTemporalEligibility {
        condition,
        clock_domain: ClockDomain::MonotonicExecution,
    };
}
