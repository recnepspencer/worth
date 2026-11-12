//! Store-issued no-effect ACKs retain discoverable work until exact expiry.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryInboundPendingReason;

#[test]
fn a_no_effect_terminal_ack_remains_maintainable_after_the_callback_caller_leaves() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, rail) = owners.open(execution, true, &[Outcome::Acknowledged]);
    let route = Route::install(&home, "ack-audience");
    let (_, scope) = pending_support::scope();
    let effect = emit(&home, 0x8421, &scope);
    let body = route.message(&owners, &effect, 51);
    let admitted = home
        .runtime()
        .runtime()
        .authenticate_inbound_occurrence(&route.handle, &body)
        .unwrap()
        .correlate()
        .unwrap()
        .accept()
        .unwrap();
    home.stop_next_prepared_dispatch_publication_for_test();
    let receipt = admitted.execute(&scope).unwrap();
    assert_eq!(receipt.posture(), Posture::AcceptedPending);
    assert_eq!(
        receipt.pending_reason(),
        Some(WorthQueryInboundPendingReason::TerminalCleanupUnavailable)
    );
    drop(receipt);
    let original = terminal(&home, &route, &effect);
    let report = maintain(&home, &route);
    assert_eq!(
        report.signed_available_before(),
        1,
        "ACK remains framework-owned after proven no effect"
    );
    assert_eq!(report.performed(), 1);
    assert_eq!(report.remaining(), 0);
    owners.expire_messages();
    assert_eq!(
        home.runtime()
            .runtime()
            .cleanup_completed_inbound_occurrences(&route.handle, NonZeroUsize::new(1).unwrap())
            .unwrap()
            .reclaimed(),
        1
    );
    assert_eq!(terminal(&home, &route, &effect), original);
    assert_eq!(rail.contacts().len(), 1);
    close(home, &execution);
}

#[test]
fn a_transport_winning_an_accepted_callback_retries_its_no_effect_ack_before_expiry() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, rail) = owners.open(
        execution,
        true,
        &[Outcome::Acknowledged, Outcome::Completed],
    );
    let route = Route::install(&home, "transport-ack-audience");
    let (_, scope) = pending_support::scope();
    let effect = emit(&home, 0x8422, &scope);
    let body = route.message(&owners, &effect, 52);
    let admitted = home
        .runtime()
        .runtime()
        .authenticate_inbound_occurrence(&route.handle, &body)
        .unwrap()
        .correlate()
        .unwrap()
        .accept()
        .unwrap();
    // Lower-owner race court: a second actual physical observation wins World
    // while the admitted callback still holds its publication claim.
    let correlation = effect.correlation;
    home.runtime()
        .runtime()
        .with_application_advancement(&scope, |phase| {
            let owner = home
                .runtime()
                .runtime()
                .observe_committed_dispatch_outbox_by_correlation(&phase, &correlation)
                .unwrap();
            home.runtime()
                .runtime()
                .perform_committed_external_dispatch_for_test(
                    &phase.borrowed_request_for_test(),
                    &phase,
                    rail.as_ref(),
                    owner,
                    &scope,
                )
        })
        .unwrap()
        .unwrap();
    let original = terminal(&home, &route, &effect);
    home.stop_next_prepared_dispatch_publication_for_test();
    let receipt = admitted.execute(&scope).unwrap();
    assert_eq!(receipt.posture(), Posture::AcceptedPending);
    assert_eq!(
        receipt.pending_reason(),
        Some(WorthQueryInboundPendingReason::TerminalCleanupUnavailable)
    );
    drop(receipt);
    let report = maintain(&home, &route);
    assert_eq!(report.signed_available_before(), 1);
    assert_eq!(report.performed(), 1);
    assert_eq!(report.remaining(), 0);
    owners.expire_messages();
    assert_eq!(
        home.runtime()
            .runtime()
            .cleanup_completed_inbound_occurrences(&route.handle, NonZeroUsize::new(1).unwrap())
            .unwrap()
            .reclaimed(),
        1
    );
    assert_eq!(terminal(&home, &route, &effect), original);
    assert_eq!(
        rail.contacts().len(),
        2,
        "only the two intentional physical observations"
    );
    close(home, &execution);
}
