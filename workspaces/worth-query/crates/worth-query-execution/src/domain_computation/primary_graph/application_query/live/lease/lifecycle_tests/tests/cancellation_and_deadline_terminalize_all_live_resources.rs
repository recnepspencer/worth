//! cancellation and deadline terminalize all live resources.
use super::*;

#[test]
fn cancellation_and_deadline_terminalize_all_live_resources() {
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
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

#[test]
fn read_stops_survive_a_failed_live_lease_termination() {
    for mode in 0..3 {
        let cancellation = WorthQueryCancellationSource::new();
        let context = LiveContext::new(WorthQueryRequestScope::new(None, cancellation.token()));
        let mut lease = context
            .world
            .application
            .with_host_advancement(|phase| context.open(&phase.test_host_request()))
            .unwrap();
        if mode == 2 {
            commit_live_activity_with_identity(
                &context.world,
                &context.principal,
                &context.request,
                "stopped-live-read",
                228,
                92,
            );
        }
        let refreshed = (mode == 2).then(|| {
            context
                .world
                .application
                .with_host_advancement(|phase| {
                    context.principal(&phase.test_host_request(), "alice")
                })
                .unwrap()
        });
        // Hold the real owned read work away from finalization: Bridge can end,
        // but the managed read cannot finish without this completion evidence.
        let held_work = lease
            .initial_read_work
            .take()
            .expect("an open lease owns read work");
        let expired = context.already_expired_request();
        if mode == 0 {
            cancellation.cancel();
        }
        let answer = context
            .world
            .application
            .with_host_advancement(|phase| {
                let host = phase.test_host_request();
                let installed_phase = lease.fixture_phase(&host);
                let execution = installed_phase
                    .execution_request_for(context.world.application.product_runtime())
                    .unwrap();
                execution
                    .run(
                        worth_execution::ExecutionWorkCeiling::new(if mode == 2 {
                            0
                        } else {
                            100_000
                        }),
                        |_| {
                            lease.next(
                                &installed_phase,
                                refreshed.as_ref().unwrap_or(&context.principal),
                                if mode == 1 {
                                    &expired
                                } else {
                                    &context.request
                                },
                            )
                        },
                    )
                    .unwrap()
                    .0
            })
            .unwrap();
        match (mode, answer) {
            (0, WorthQueryApplicationLiveOutcome::Cancelled) | (1, WorthQueryApplicationLiveOutcome::DeadlineExceeded) => {},
            (2, WorthQueryApplicationLiveOutcome::ExecutionRequest(kind)) => assert_eq!(kind, crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource { denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted, partition_identity: None, policy_ancestor: None }),
            (_, answer) => panic!("mode {mode}: the request stop remains the answer, got {:?}", std::mem::discriminant(&answer)),
        }
        assert!(lease.termination_failed());
        drop(held_work);
    }
}
