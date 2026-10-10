//! cancellation and deadline terminalize all live resources.
use super::*;

#[test]
fn cancellation_and_deadline_terminalize_all_live_resources() {
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let context = LiveContext::new(request);
    let mut lease = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            context.open(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let bridge = observer(&lease);
    cancellation.cancel();
    context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(matches!(
                {
                    let installed_fixture_phase = lease.fixture_phase(&host_custody);
                    lease.next(
                        &installed_fixture_phase,
                        &context.principal,
                        &context.request,
                    )
                },
                WorthQueryApplicationLiveOutcome::Cancelled
            ))
        })
        .expect("the fixture read owns its installed host call");
    assert_terminal(
        &bridge,
        BridgeExecutionBasisLifecycleSignalStatus::Cancelled,
    );
    drop(lease);
    context.assert_source_released();

    let deadline_context =
        LiveContext::new(crate::domain_computation::primary_graph::tests::fixture::live_scope());
    let mut deadline_lease = deadline_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            deadline_context.open(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let deadline_bridge = observer(&deadline_lease);
    // Live-lease deadlines still sample wall-clock Instant, not the Gate
    // 8.3 injectable runtime clock. Open under a non-expiring scope, then
    // pass an already-expired deadline only to the delivery under test.
    let expired_request = deadline_context.already_expired_request();
    deadline_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(matches!(
                {
                    let installed_fixture_phase = deadline_lease.fixture_phase(&host_custody);
                    deadline_lease.next(
                        &installed_fixture_phase,
                        &deadline_context.principal,
                        &expired_request,
                    )
                },
                WorthQueryApplicationLiveOutcome::DeadlineExceeded
            ))
        })
        .expect("the fixture read owns its installed host call");
    assert_terminal(
        &deadline_bridge,
        BridgeExecutionBasisLifecycleSignalStatus::Cancelled,
    );
    drop(deadline_lease);
    deadline_context.assert_source_released();
}
