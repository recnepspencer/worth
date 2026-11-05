//! The scoped checkpointer observes genuine cancellation/deadline while borrowed.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_execution::facade::domain_computation::HandlerInterruption;
use std::cell::Cell;
use std::time::Duration;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[test]
fn cancelled_before_admission_never_enters_the_raw_callback() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
        source.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let called = Cell::new(false);
    let result = world
        .application
        .with_host_advancement(|phase| {
            crate::domain_computation::primary_graph::tests::application_attempt::predecode_admission::projection::with_reader(&phase, &world, &request, |reader| {
                let scope = reader.scope().clone();
                source.cancel();
                reader.field_with_predecode_admission(&scope, AccountStatus::reference(), |_, _| {
                    called.set(true);
                    Ok::<_, ()>(())
                })
            })
        })
        .expect("the fixture read owns its installed host call");
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
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
        source.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let observed = Cell::new(None);
    let result = world
        .application
        .with_host_advancement(|phase| {
            crate::domain_computation::primary_graph::tests::application_attempt::predecode_admission::projection::with_reader(&phase, &world, &request, |reader| {
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
            })
        })
        .expect("the fixture read owns its installed host call");
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
    let time = worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionTimeController::new().unwrap();
    let held = time.hold();
    let world = installed_authorization_world(true, time.clock());
    let source = WorthQueryCancellationSource::new();
    let deadline = held + Duration::from_secs(1);
    let request = WorthQueryRequestScope::new(Some(deadline), source.token(), time.clock());
    let observed = Cell::new(None);
    let result = world
        .application
        .with_host_advancement(|phase| {
            crate::domain_computation::primary_graph::tests::application_attempt::predecode_admission::projection::with_reader(&phase, &world, &request, |reader| {
                let scope = reader.scope().clone();
                reader.field_with_predecode_admission(
                    &scope,
                    AccountStatus::reference(),
                    |_, checkpoint| {
                        assert_eq!(checkpoint(), Ok(()));
                        time.advance(Duration::from_secs(1)).unwrap();
                        observed.set(checkpoint().err());
                        Ok::<_, ()>(())
                    },
                )
            })
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(observed.get(), Some(HandlerInterruption::DeadlineExceeded));
    assert_eq!(
        result
            .unwrap_err()
            .downcast::<HandlerInterruption>()
            .unwrap(),
        HandlerInterruption::DeadlineExceeded
    );
}
