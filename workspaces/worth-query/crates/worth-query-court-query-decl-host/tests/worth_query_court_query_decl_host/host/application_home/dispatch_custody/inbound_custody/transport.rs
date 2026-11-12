//! Recorded Completed is continued through World without contacting a reopened rail.
use super::*;

#[test]
fn recorded_completed_transport_acquires_its_first_audience_and_publishes_after_reopen() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, rail) = owners.open(execution, true, &[Outcome::Completed]);
    let (_, scope) = pending_support::scope();
    let effect = emit(&home, 0x8411, &scope);
    assert_eq!(
        custody(&home, &effect.token).observation(),
        Some(Observation::Completed)
    );
    assert_eq!(rail.contacts().len(), 1);
    close(home, &execution);

    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "first-transport-audience");
    let report = maintain(&home, &route);
    assert_eq!(report.performed(), 1);
    assert_eq!(report.remaining(), 0);
    let original = terminal(&home, &route, &effect);
    let body = route.message(&owners, &effect, 41);
    assert_eq!(receive(&home, &route, &body), Ok(Posture::AlreadyCompleted));
    assert!(rail.contacts().is_empty());
    close(home, &execution);

    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "first-transport-audience");
    assert_eq!(terminal(&home, &route, &effect), original);
    assert_eq!(route.message(&owners, &effect, 41), body);
    assert_eq!(receive(&home, &route, &body), Ok(Posture::AlreadyCompleted));
    assert!(rail.contacts().is_empty());
    close(home, &execution);
}

#[test]
fn recorded_completed_transport_preserves_the_selected_audience_when_its_request_stops() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, rail) = owners.open(execution, true, &[Outcome::Completed]);
    let _route = Route::install(&home, "selected-transport-audience");
    let (cancellation, scope) = pending_support::scope();
    *rail.stops_at_contact.lock().unwrap() = Some(cancellation);
    let effect = emit(&home, 0x8412, &scope);
    assert_eq!(rail.contacts().len(), 1);
    close(home, &execution);

    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "replacement-verifier-audience");
    assert_eq!(maintain(&home, &route).performed(), 1);
    let original = terminal(&home, &route, &effect);
    assert_conflict(
        receive(&home, &route, &route.message(&owners, &effect, 42)),
        Conflict::CorrelationAlreadyOwned,
    );
    assert!(rail.contacts().is_empty());
    close(home, &execution);

    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "selected-transport-audience");
    assert_eq!(
        receive(&home, &route, &route.message(&owners, &effect, 43)),
        Ok(Posture::AlreadyCompleted)
    );
    assert_eq!(terminal(&home, &route, &effect), original);
    assert!(rail.contacts().is_empty());
    close(home, &execution);
}
