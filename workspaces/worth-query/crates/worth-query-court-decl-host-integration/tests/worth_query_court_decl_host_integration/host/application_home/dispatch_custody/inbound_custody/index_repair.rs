//! Destroying a reopened disposable index preserves its actual original completion source.
use super::*;
#[test]
fn a_reopened_terminal_index_rebuilds_from_actual_sources_and_preserves_duplicate_meaning() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, _) = owners.open(execution, true, &[Outcome::Acknowledged]);
    let route = Route::install(&home, "repair-audience");
    let (_, scope) = pending_support::scope();
    let effect = emit(&home, 0x8431, &scope);
    let body = route.message(&owners, &effect, 61);
    assert_eq!(receive(&home, &route, &body), Ok(Posture::Performed));
    let original = terminal(&home, &route, &effect);
    close(home, &execution);

    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "repair-audience");
    assert_eq!(route.message(&owners, &effect, 61), body);
    home.runtime()
        .runtime()
        .runtime_for_test()
        .primary_provider_for_test()
        .destroy_completed_inbound_index_for_test();
    assert!(home
        .runtime()
        .runtime()
        .observe_inbound_terminal(&route.handle, effect.token)
        .is_none());
    let repaired = home
        .runtime()
        .runtime()
        .with_application_advancement(&scope, |phase| {
            home.runtime().runtime().repair_completed_inbound_index(
                &phase,
                &route.handle,
                NonZeroUsize::new(1).unwrap(),
                NonZeroUsize::new(32).unwrap(),
                NonZeroUsize::new(128).unwrap(),
            )
        })
        .unwrap()
        .unwrap();
    assert!(
        repaired,
        "the finite source roster contains exactly this terminal"
    );
    assert_eq!(terminal(&home, &route, &effect), original);
    assert_eq!(receive(&home, &route, &body), Ok(Posture::AlreadyCompleted));
    assert!(rail.contacts().is_empty());
    close(home, &execution);
}
