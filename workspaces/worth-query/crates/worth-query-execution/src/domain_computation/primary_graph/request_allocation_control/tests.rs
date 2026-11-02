use super::*;
use std::time::Duration;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;

#[test]
fn live_query_cancellation_registration_stops_execution_token_until_scope_drop() {
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
        source.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let registration = CancellationRegistration::new(&request);
    assert!(!registration.cancellation.is_cancelled());
    source.cancel();
    assert!(registration.cancellation.is_cancelled());
    assert!(CancellationRegistration::new(&request)
        .cancellation
        .is_cancelled());

    let later = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        request.deadline(),
        later.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let registration = CancellationRegistration::new(&request);
    let detached = registration.cancellation.clone();
    drop(registration);
    later.cancel();
    assert!(
        !detached.is_cancelled(),
        "dropped registration must unregister its waker"
    );
}

#[test]
fn explicit_system_allocation_remains_uncharged_under_query_stop() {
    let source = WorthQueryCancellationSource::new();
    source.cancel();
    let request = WorthQueryRequestScope::new(
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now()),
        source.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let control =
        RequestAllocationControl::new(&request, ExecutionAllocationPolicy::SystemAllocation);
    assert!(matches!(
        control.policy(),
        ExecutionAllocationPolicy::SystemAllocation
    ));
    assert!(control.policy().check_live().is_ok());
    assert!(
        request.interruption().is_some(),
        "ordinary Query stop checks remain authoritative"
    );
}
