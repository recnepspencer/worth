use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenial;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
    use worth_query_execution::domain_computation::runtime_time::time_basis::WorthQueryRuntimeClock;
    use worth_query_execution::domain_computation::primary_graph::application_attempt::workflow_deadline::ensure_before;
    use worth_query_execution::domain_computation::primary_graph::application_attempt::workflow_deadline::now;
    use worth_query_execution::domain_computation::primary_graph::application_attempt::workflow_deadline::start_deadline;
    use worth_query_execution::facade::domain_computation::WorthQueryRuntimeTimeSource;
    use worth_query_execution::facade::domain_computation::WorthQueryRuntimeTimeSourceDenial;

/// Trusted time fixed at `Some(milliseconds)`, or unreadable at `None`.
pub struct FixedTime(Option<u64>);

impl WorthQueryRuntimeTimeSource for FixedTime {
    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        self.0
            .map(|milliseconds| UNIX_EPOCH + Duration::from_millis(milliseconds))
            .ok_or(WorthQueryRuntimeTimeSourceDenial::Unavailable)
    }
}

pub fn clock(now: Option<u64>) -> WorthQueryRuntimeClock {
    WorthQueryRuntimeClock::from_source(FixedTime(now))
}

pub fn kind<Value>(
    outcome: Result<Value, WorthQueryApplicationAttemptDenial>,
) -> Option<WorthQueryApplicationAttemptDenialKind> {
    outcome.err().map(|denial| denial.kind())
}

#[test]
fn a_deadline_bounds_steps_strictly_before_it() {
    let clock = clock(Some(10_000));
    assert!(ensure_before(&clock, 10_001).is_ok());
    assert_eq!(
        kind(ensure_before(&clock, 10_000)),
        Some(WorthQueryApplicationAttemptDenialKind::WorkflowInstanceDeadlineElapsed),
    );
}

#[test]
fn a_start_keeps_the_earlier_of_its_inherited_and_declared_deadline() {
    let clock = clock(Some(10_000));
    assert_eq!(
        start_deadline(&clock, None, Some(500)).ok(),
        Some(Some(10_500))
    );
    assert_eq!(
        start_deadline(&clock, Some(10_200), Some(500)).ok(),
        Some(Some(10_200)),
    );
    assert_eq!(
        start_deadline(&clock, Some(20_000), Some(500)).ok(),
        Some(Some(10_500)),
    );
    assert_eq!(
        start_deadline(&clock, Some(10_200), None).ok(),
        Some(Some(10_200))
    );
    assert_eq!(
        start_deadline(&clock, None, Some(u64::MAX)).ok(),
        Some(Some(u64::MAX)),
    );
}

#[test]
fn an_unreadable_clock_cannot_bound_a_deadline() {
    let clock = clock(None);
    assert_eq!(
        kind(ensure_before(&clock, 10_000)),
        Some(WorthQueryApplicationAttemptDenialKind::WorkflowTrustedTimeUnavailable),
    );
    assert_eq!(
        kind(start_deadline(&clock, None, Some(500))),
        Some(WorthQueryApplicationAttemptDenialKind::WorkflowTrustedTimeUnavailable),
    );
    assert_eq!(
        start_deadline(&clock, Some(10_200), None).ok(),
        Some(Some(10_200)),
        "a definition declaring no deadline never reads the clock",
    );
}
