//! The scoped checkpointer observes genuine cancellation/deadline while borrowed.
use super::projection;
use crate::domain_computation::primary_graph::{
    tests::fixture::{installed_authorization_world, AccountStatus},
    HandlerInterruption,
};
use std::cell::Cell;
use std::time::{Duration, Instant};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn cancelled_before_admission_never_enters_the_raw_callback() {
    let world = installed_authorization_world(true);
    let source = WorthQueryCancellationSource::new();
    let request =
        WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(60), source.token());
    let called = Cell::new(false);
    let result = projection::with_reader(&world, &request, |reader| {
        let scope = reader.scope().clone();
        source.cancel();
        reader.field_with_predecode_admission(&scope, AccountStatus::reference(), |_, _| {
            called.set(true);
            Ok::<_, ()>(())
        })
    });
    assert_eq!(
        result
            .unwrap_err()
            .downcast::<HandlerInterruption>()
            .unwrap(),
        HandlerInterruption::Cancelled
    );
    assert!(!called.get());
}

#[test]
fn cancellation_during_preflight_is_observable_without_reborrowing_reader() {
    let world = installed_authorization_world(true);
    let source = WorthQueryCancellationSource::new();
    let request =
        WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(60), source.token());
    let observed = Cell::new(None);
    let result = projection::with_reader(&world, &request, |reader| {
        let scope = reader.scope().clone();
        reader.field_with_predecode_admission(
            &scope,
            AccountStatus::reference(),
            |_, checkpoint| {
                assert_eq!(checkpoint(), Ok(()));
                source.cancel();
                observed.set(checkpoint().err());
                // Even a caller's unrelated domain refusal must retain interruption.
                Err::<(), _>("owner-refusal")
            },
        )
    });
    assert_eq!(observed.get(), Some(HandlerInterruption::Cancelled));
    assert_eq!(
        result
            .unwrap_err()
            .downcast::<HandlerInterruption>()
            .unwrap(),
        HandlerInterruption::Cancelled
    );
}

#[test]
fn deadline_during_preflight_preserves_its_distinct_request_reason() {
    let world = installed_authorization_world(true);
    let source = WorthQueryCancellationSource::new();
    let deadline = Instant::now() + Duration::from_secs(1);
    let request = WorthQueryRequestScope::new(deadline, source.token());
    let observed = Cell::new(None);
    let result = projection::with_reader(&world, &request, |reader| {
        let scope = reader.scope().clone();
        reader.field_with_predecode_admission(
            &scope,
            AccountStatus::reference(),
            |_, checkpoint| {
                assert_eq!(checkpoint(), Ok(()));
                std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
                observed.set(checkpoint().err());
                Ok::<_, ()>(())
            },
        )
    });
    assert_eq!(observed.get(), Some(HandlerInterruption::DeadlineExceeded));
    assert_eq!(
        result
            .unwrap_err()
            .downcast::<HandlerInterruption>()
            .unwrap(),
        HandlerInterruption::DeadlineExceeded
    );
}
