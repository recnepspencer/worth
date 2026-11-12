//! Authentic callbacks and accepted original audiences survive repeated Home openings.
use super::*;

#[test]
fn authentic_callback_completes_the_original_after_reopen_and_deduplicates_another_reopen() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, rail) = owners.open(execution, true, &[Outcome::Acknowledged]);
    let route = Route::install(&home, "original-audience");
    let (_, scope) = pending_support::scope();
    let effect = emit(&home, 0x8401, &scope);
    let body = route.message(&owners, &effect, 11);
    assert_eq!(rail.contacts().len(), 1);
    assert!(home
        .runtime()
        .runtime()
        .observe_inbound_terminal(&route.handle, effect.token)
        .is_none());
    close(home, &execution);

    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "original-audience");
    assert_eq!(route.message(&owners, &effect, 11), body);
    assert_eq!(receive(&home, &route, &body), Ok(Posture::Performed));
    let original = terminal(&home, &route, &effect);
    assert_ne!(
        original.0, original.1,
        "completion owns a later World occurrence"
    );
    assert_eq!(receive(&home, &route, &body), Ok(Posture::Performed));
    assert!(rail.contacts().is_empty());
    close(home, &execution);

    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "original-audience");
    assert_eq!(route.message(&owners, &effect, 11), body);
    assert_eq!(terminal(&home, &route, &effect), original);
    assert_eq!(receive(&home, &route, &body), Ok(Posture::AlreadyCompleted));
    let second = route.message(&owners, &effect, 12);
    assert_eq!(
        receive(&home, &route, &second),
        Ok(Posture::AlreadyCompleted)
    );
    let conflict = route.message_at(&owners, &effect, 11, 29);
    assert_conflict(
        receive(&home, &route, &conflict),
        Conflict::MessageIdentityConflict,
    );
    assert_eq!(terminal(&home, &route, &effect), original);
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 1);
    assert!(rail.contacts().is_empty());
    close(home, &execution);
}

#[test]
fn accepted_pending_message_keeps_its_original_audience_under_fresh_verifier_configuration() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, _) = owners.open(execution, true, &[Outcome::Acknowledged]);
    let route = Route::install(&home, "accepted-audience");
    let (_, scope) = pending_support::scope();
    let effect = emit(&home, 0x8402, &scope);
    let body = route.message(&owners, &effect, 21);
    let accepted = home
        .runtime()
        .runtime()
        .authenticate_inbound_occurrence(&route.handle, &body)
        .unwrap()
        .correlate()
        .unwrap()
        .accept()
        .unwrap();
    drop(accepted);
    assert!(home
        .runtime()
        .runtime()
        .observe_inbound_terminal(&route.handle, effect.token)
        .is_none());
    close(home, &execution);

    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "new-verifier-audience");
    let report = maintain(&home, &route);
    assert_eq!(report.performed(), 1);
    assert_eq!(report.remaining(), 0);
    let original = terminal(&home, &route, &effect);
    let incompatible = route.message(&owners, &effect, 22);
    assert_conflict(
        receive(&home, &route, &incompatible),
        Conflict::CorrelationAlreadyOwned,
    );
    assert!(rail.contacts().is_empty());
    close(home, &execution);

    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "accepted-audience");
    assert_eq!(route.message(&owners, &effect, 21), body);
    assert_eq!(receive(&home, &route, &body), Ok(Posture::AlreadyCompleted));
    assert_eq!(terminal(&home, &route, &effect), original);
    assert!(rail.contacts().is_empty());
    close(home, &execution);
}

#[test]
fn exact_message_expiry_reclaims_two_acknowledged_messages_without_erasing_completion() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, _) = owners.open(execution, true, &[Outcome::Acknowledged]);
    let route = Route::install(&home, "expiry-audience");
    let (_, scope) = pending_support::scope();
    let effect = emit(&home, 0x8403, &scope);
    assert_eq!(
        receive(&home, &route, &route.message(&owners, &effect, 31)),
        Ok(Posture::Performed)
    );
    assert_eq!(
        receive(&home, &route, &route.message(&owners, &effect, 32)),
        Ok(Posture::AlreadyCompleted)
    );
    let original = terminal(&home, &route, &effect);
    close(home, &execution);

    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "expiry-audience");
    owners.expire_messages();
    for _message in 0..2 {
        assert_eq!(
            home.runtime()
                .runtime()
                .cleanup_completed_inbound_occurrences(&route.handle, NonZeroUsize::new(1).unwrap())
                .unwrap()
                .reclaimed(),
            1
        );
    }
    assert_eq!(
        home.runtime()
            .runtime()
            .cleanup_completed_inbound_occurrences(&route.handle, NonZeroUsize::new(1).unwrap())
            .unwrap()
            .reclaimed(),
        0
    );
    assert_eq!(terminal(&home, &route, &effect), original);
    assert!(rail.contacts().is_empty());
    close(home, &execution);
}
