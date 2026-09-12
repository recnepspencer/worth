mod tests {
    use std::time::{Duration, Instant};

    use worth_query_admission::facade::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestScope,
    };
    use worth_runtime_bridge::facade::{
        BridgeExecutionBasisLifecycleObserver, BridgeExecutionBasisLifecycleSignalStatus,
    };

    use crate::domain_computation::primary_graph::tests::{
        fixture::{
            installed_authorization_world, live_account_parameters, Account, AccountIdentity,
            AccountSummaryParameters, Activity, AuthorizationWorld, IdentityExecutionSchema,
            LiveAccountActivityCause, LiveAccountActivityQuery, LiveAccountActivityResult,
            Principal,
        },
        live_delivery_support::commit_live_activity_with_identity,
    };
    use crate::domain_computation::primary_graph::{
        WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveControls,
        WorthQueryApplicationLiveLease, WorthQueryApplicationLiveOutcome,
        WorthQueryAuthenticatedPrincipal, WorthQueryPrincipalResolutionMode,
    };

    type TestLiveLease<'runtime, 'principal> = WorthQueryApplicationLiveLease<
        'runtime,
        'principal,
        IdentityExecutionSchema,
        LiveAccountActivityQuery,
        AccountSummaryParameters,
        LiveAccountActivityResult,
        Principal,
        u64,
        Account,
        Activity,
        LiveAccountActivityCause,
    >;

    struct LiveContext {
        world: AuthorizationWorld,
        request: WorthQueryRequestScope,
        principal: WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    }

    impl LiveContext {
        fn new(request: WorthQueryRequestScope) -> Self {
            let world = installed_authorization_world(true);
            let external = world.authenticate("alice", Duration::from_secs(60), &request);
            let principal = world
                .application
                .select_product_branch(world.application.product_runtime().default_branch())
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            Self {
                world,
                request,
                principal,
            }
        }

        fn with_already_expired_deadline_after_open(&self, lease: &mut TestLiveLease<'_, '_>) {
            let live_cancellation = WorthQueryCancellationSource::new();
            // Instant deadline comparison is `now >= deadline`, so binding the
            // current instant makes the next poll sample DeadlineExceeded
            // without sleeping or racing open.
            lease.replace_request(WorthQueryRequestScope::new(
                Instant::now(),
                live_cancellation.token(),
            ));
        }

        fn open(&self) -> TestLiveLease<'_, '_> {
            let query = self
                .world
                .application
                .installed_schema()
                .certification_query(LiveAccountActivityQuery::reference())
                .unwrap();
            let account = self
                .world
                .application
                .select_product_branch(self.world.application.product_runtime().default_branch())
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    AccountIdentity::reference(),
                    "account-1".to_owned(),
                    &self.request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            self.world
                .selected_product()
                .open_application_query_live::<
                    LiveAccountActivityQuery,
                    AccountSummaryParameters,
                    LiveAccountActivityResult,
                    Principal,
                    u64,
                    Account,
                    Activity,
                    LiveAccountActivityCause,
                >(
                    query,
                    &self.principal,
                    account,
                    live_account_parameters("account-1"),
                    WorthQueryApplicationLiveControls::bounded(
                        self.request.clone(),
                        4,
                        16,
                        2_048,
                    )
                    .unwrap(),
                )
                .unwrap()
        }

        fn close_source(&self) {
            self.world
                .application
                .primary_provider
                .live_delivery
                .close();
        }

        fn overflow_source(&self) {
            for ordinal in 1..=65_u8 {
                commit_live_activity_with_identity(
                    &self.world,
                    &self.principal,
                    &self.request,
                    &format!("overflow-{ordinal}"),
                    ordinal,
                    ordinal,
                );
            }
        }

        fn assert_source_released(&self) {
            assert_eq!(
                self.world
                    .application
                    .primary_provider
                    .active_application_commit_causality_partitions(),
                0
            );
        }
    }

    #[test]
    fn explicit_close_releases_bridge_signal_and_queue_resources() {
        let context = LiveContext::new(
            crate::domain_computation::primary_graph::tests::fixture::live_scope(),
        );
        let lease = context.open();
        let bridge = observer(&lease);
        let provider_session_baseline = context.world.application.provider_session_resource_count();
        assert_live(&bridge);

        let WorthQueryApplicationLiveCloseOutcome::Completed(completion) = lease.close() else {
            panic!("live close must complete its opening graph-read session");
        };
        assert_eq!(completion.release().released_reservation_count(), 1);
        assert!(completion.basis_release().released());
        assert_eq!(
            context.world.application.provider_session_resource_count(),
            provider_session_baseline
        );
        assert_terminal(
            &bridge,
            BridgeExecutionBasisLifecycleSignalStatus::Fulfilled,
        );
        context.assert_source_released();
    }

    #[test]
    fn cancellation_and_deadline_terminalize_all_live_resources() {
        let cancellation = WorthQueryCancellationSource::new();
        let request = WorthQueryRequestScope::new(
            Instant::now() + Duration::from_secs(60),
            cancellation.token(),
        );
        let context = LiveContext::new(request);
        let mut lease = context.open();
        let bridge = observer(&lease);
        cancellation.cancel();
        assert!(matches!(
            lease.poll(),
            WorthQueryApplicationLiveOutcome::Cancelled
        ));
        assert_terminal(
            &bridge,
            BridgeExecutionBasisLifecycleSignalStatus::Cancelled,
        );
        drop(lease);
        context.assert_source_released();

        let deadline_context = LiveContext::new(
            crate::domain_computation::primary_graph::tests::fixture::live_scope(),
        );
        let mut deadline_lease = deadline_context.open();
        let deadline_bridge = observer(&deadline_lease);
        // Live-lease deadlines still sample wall-clock Instant, not the Gate
        // 8.3 injectable runtime clock. Open under a non-expiring scope, then
        // bind an already-expired deadline only to the poll phase under test.
        deadline_context.with_already_expired_deadline_after_open(&mut deadline_lease);
        assert!(matches!(
            deadline_lease.poll(),
            WorthQueryApplicationLiveOutcome::DeadlineExceeded
        ));
        assert_terminal(
            &deadline_bridge,
            BridgeExecutionBasisLifecycleSignalStatus::Cancelled,
        );
        drop(deadline_lease);
        deadline_context.assert_source_released();
    }

    #[test]
    fn source_close_overflow_and_abandoned_drop_release_all_live_resources() {
        let closed_context = LiveContext::new(
            crate::domain_computation::primary_graph::tests::fixture::live_scope(),
        );
        let mut closed = closed_context.open();
        let closed_bridge = observer(&closed);
        closed_context.close_source();
        assert!(matches!(
            closed.poll(),
            WorthQueryApplicationLiveOutcome::Closed
        ));
        assert_terminal(
            &closed_bridge,
            BridgeExecutionBasisLifecycleSignalStatus::Fulfilled,
        );
        drop(closed);
        closed_context.assert_source_released();

        let overflow_context = LiveContext::new(
            crate::domain_computation::primary_graph::tests::fixture::live_scope(),
        );
        let mut overflow = overflow_context.open();
        let overflow_bridge = observer(&overflow);
        overflow_context.overflow_source();
        let WorthQueryApplicationLiveOutcome::Overflow(missed) = overflow.poll() else {
            panic!("retention loss must terminalize as overflow");
        };
        assert_eq!(missed.missed_commit_batches(), 1);
        assert_terminal(
            &overflow_bridge,
            BridgeExecutionBasisLifecycleSignalStatus::Cancelled,
        );
        drop(overflow);
        overflow_context.assert_source_released();

        let abandoned_context = LiveContext::new(
            crate::domain_computation::primary_graph::tests::fixture::live_scope(),
        );
        let abandoned = abandoned_context.open();
        let abandoned_bridge = observer(&abandoned);
        drop(abandoned);
        assert_terminal(
            &abandoned_bridge,
            BridgeExecutionBasisLifecycleSignalStatus::Cancelled,
        );
        abandoned_context.assert_source_released();
    }

    fn observer(lease: &TestLiveLease<'_, '_>) -> BridgeExecutionBasisLifecycleObserver {
        let basis = lease.basis.as_ref().expect("open lease retains its basis");
        basis.bridge.lifecycle_observer()
    }

    fn assert_live(bridge: &BridgeExecutionBasisLifecycleObserver) {
        let observation = bridge.observe().unwrap();
        assert!(observation.reservation_active());
        assert_eq!(
            observation.signal_status(),
            Some(BridgeExecutionBasisLifecycleSignalStatus::Active)
        );
        assert_eq!(
            observation
                .managed_queue_pressure()
                .expect("live lease owns one bounded queue")
                .queue_depth(),
            0
        );
    }

    fn assert_terminal(
        bridge: &BridgeExecutionBasisLifecycleObserver,
        status: BridgeExecutionBasisLifecycleSignalStatus,
    ) {
        let observation = bridge.observe().unwrap();
        assert!(!observation.reservation_active());
        assert_eq!(observation.signal_status(), Some(status));
        assert_eq!(
            observation
                .managed_queue_pressure()
                .expect("terminal request retains an empty queue observation")
                .queue_depth(),
            0
        );
    }
}
