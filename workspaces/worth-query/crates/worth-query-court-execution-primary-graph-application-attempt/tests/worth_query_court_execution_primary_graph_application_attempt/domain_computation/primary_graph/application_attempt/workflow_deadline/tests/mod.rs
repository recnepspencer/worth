use std::time::Duration;

use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenial;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
    use worth_query_execution::court_doors_for_test::domain_computation::runtime_time::time_basis::WorthQueryRuntimeClock;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow_deadline::ensure_before;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow_deadline::start_deadline;

pub fn clock(now: Option<u64>) -> WorthQueryRuntimeClock {
    let source = std::sync::Arc::new(worth_execution::time::ControlledTime::at_unix_nanos(
        now.unwrap_or(0) * 1_000_000,
    ));
    if now.is_none() {
        source.step_wall_back(Duration::from_nanos(1));
    }
    source.into()
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
