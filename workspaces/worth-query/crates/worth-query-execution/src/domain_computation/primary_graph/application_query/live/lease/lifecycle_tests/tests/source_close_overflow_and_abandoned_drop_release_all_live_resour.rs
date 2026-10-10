//! source close overflow and abandoned drop release all live resources.
use super::*;

#[test]
fn source_close_overflow_and_abandoned_drop_release_all_live_resources() {
    let closed_context =
        LiveContext::new(crate::domain_computation::primary_graph::tests::fixture::live_scope());
    let mut closed = closed_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            closed_context.open(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let closed_bridge = observer(&closed);
    closed_context.close_source();
    closed_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(matches!(
                {
                    let installed_fixture_phase = closed.fixture_phase(&host_custody);
                    closed.next(
                        &installed_fixture_phase,
                        &closed_context.principal,
                        &closed_context.request,
                    )
                },
                WorthQueryApplicationLiveOutcome::Closed
            ))
        })
        .expect("the fixture read owns its installed host call");
    assert_terminal(
        &closed_bridge,
        BridgeExecutionBasisLifecycleSignalStatus::Fulfilled,
    );
    drop(closed);
    closed_context.assert_source_released();

    let overflow_context =
        LiveContext::new(crate::domain_computation::primary_graph::tests::fixture::live_scope());
    let mut overflow = overflow_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            overflow_context.open(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let overflow_bridge = observer(&overflow);
    overflow_context.overflow_source();
    let WorthQueryApplicationLiveOutcome::Overflow(missed) = overflow_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            {
                let installed_fixture_phase = overflow.fixture_phase(&host_custody);
                overflow.next(
                    &installed_fixture_phase,
                    &overflow_context.principal,
                    &overflow_context.request,
                )
            }
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("retention loss must terminalize as overflow");
    };
    assert_eq!(missed.missed_commit_batches(), 1);
    assert_terminal(
        &overflow_bridge,
        BridgeExecutionBasisLifecycleSignalStatus::Cancelled,
    );
    drop(overflow);
    overflow_context.assert_source_released();

    let abandoned_context =
        LiveContext::new(crate::domain_computation::primary_graph::tests::fixture::live_scope());
    let abandoned = abandoned_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            abandoned_context.open(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let abandoned_bridge = observer(&abandoned);
    drop(abandoned);
    assert_terminal(
        &abandoned_bridge,
        BridgeExecutionBasisLifecycleSignalStatus::Cancelled,
    );
    abandoned_context.assert_source_released();
}
