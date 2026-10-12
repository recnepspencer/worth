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
            installed_two_principal_authorization_world, live_account_parameters, Account,
            AccountIdentity, AccountSummaryParameters, Activity, AuthorizationWorld,
            IdentityExecutionSchema, LiveAccountActivityCause, LiveAccountActivityQuery,
            LiveAccountActivityResult, Principal,
        },
        live_delivery_support::commit_live_activity_with_identity,
    };
    use crate::domain_computation::primary_graph::{
        WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveControls,
        WorthQueryApplicationLiveLease, WorthQueryApplicationLiveOutcome,
        WorthQueryAuthenticatedPrincipal, WorthQueryPrincipalResolutionMode,
    };

    type TestLiveLease<'runtime> = WorthQueryApplicationLiveLease<
        'runtime,
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
            let world = installed_two_principal_authorization_world(true);
            let external = world.authenticate("alice", Duration::from_secs(60), &request);
            let principal = world
                .application
                .with_host_advancement(|phase| {
                    let phase_custody = &phase;
                    let borrowed_execution = phase
                        .execution_request_for(world.application.product_runtime())
                        .expect("the installed fixture lends its own phase");
                    let execution = &borrowed_execution;
                    let principal = world
                        .application
                        .select_product_branch(
                            execution,
                            world.application.product_runtime().default_branch(),
                        )
                        .expect("the selected product branch remains admitted")
                        .resolve_authenticated_principal(
                            phase_custody,
                            &world.binding,
                            &external,
                            &request,
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .unwrap();

                    principal
                })
                .expect("the fixture host admits its identity reads");
            Self {
                world,
                request,
                principal,
            }
        }

        fn already_expired_request(&self) -> WorthQueryRequestScope {
            let live_cancellation = WorthQueryCancellationSource::new();
            // Instant deadline comparison is `now >= deadline`, so the
            // current instant makes the next delivery sample DeadlineExceeded
            // without sleeping or racing open.
            WorthQueryRequestScope::new(Instant::now(), live_cancellation.token())
        }

        fn principal(
            &self,
            phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
            identity: &str,
        ) -> WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64> {
            let borrowed_execution = phase_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;

            let external =
                self.world
                    .authenticate(identity, Duration::from_secs(60), &self.request);
            self.world
                .application
                .select_product_branch(
                    execution,
                    self.world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase_custody.phase_for_application(&self.world.application),
                    &self.world.binding,
                    &external,
                    &self.request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        }

        fn open(
            &self,
            phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
        ) -> TestLiveLease<'_> {
            let borrowed_execution = phase_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;

            let query = self
                .world
                .application
                .installed_schema()
                .certification_query(LiveAccountActivityQuery::reference())
                .unwrap();
            let account = self
                .world
                .application
                .select_product_branch(
                    execution,
                    self.world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase_custody.phase_for_application(&self.world.application),
                    AccountIdentity::reference(),
                    "account-1".to_owned(),
                    &self.request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            self.world
                .selected_product(&(phase_custody).test_host_request(), )
                .open_application_query_live::<
                    LiveAccountActivityQuery,
                    AccountSummaryParameters,
                    LiveAccountActivityResult,
                    Principal,
                    u64,
                    Account,
                    Activity,
                    LiveAccountActivityCause,
                >(&phase_custody.phase_for_application(&self.world.application),
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
        let lease = context
            .world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                context.open(&host_custody)
            })
            .expect("the fixture read owns its installed host call");
        let bridge = observer(&lease);
        let provider_session_baseline = context
            .world
            .application
            .provider_session_resource_count_for_test();
        assert_live(&bridge);

        let WorthQueryApplicationLiveCloseOutcome::Completed(completion) = lease.close() else {
            panic!("live close must complete its opening graph-read session");
        };
        assert_eq!(completion.release().released_reservation_count(), 1);
        assert!(completion.basis_release().released());
        assert_eq!(
            context
                .world
                .application
                .provider_session_resource_count_for_test(),
            provider_session_baseline
        );
        assert_terminal(
            &bridge,
            BridgeExecutionBasisLifecycleSignalStatus::Fulfilled,
        );
        context.assert_source_released();
    }

    #[path = "../lifecycle_tests/tests/cancellation_and_deadline_terminalize_all_live_resources.rs"]
    mod cancellation_and_deadline_terminalize_all_live_resources;

    #[path = "../lifecycle_tests/tests/source_close_overflow_and_abandoned_drop_release_all_live_resour.rs"]
    mod source_close_overflow_and_abandoned_drop_release_all_live_resour;

    #[test]
    fn fresh_principal_admission_precedes_empty_and_overflow_progress() {
        let empty_context = LiveContext::new(
            crate::domain_computation::primary_graph::tests::fixture::live_scope(),
        );
        let mut empty = empty_context
            .world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                empty_context.open(&host_custody)
            })
            .expect("the fixture read owns its installed host call");
        let foreign = empty_context
            .world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                empty_context.principal(&host_custody, "bob")
            })
            .expect("the fixture read owns its installed host call");
        empty_context
            .world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                assert!(matches!(
                    {
                        let installed_fixture_phase = empty.fixture_phase(&host_custody);
                        empty.next(&installed_fixture_phase, &foreign, &empty_context.request)
                    },
                    WorthQueryApplicationLiveOutcome::AuthorizationDenied(_)
                ))
            })
            .expect("the fixture read owns its installed host call");
        drop(empty);
        empty_context.assert_source_released();

        let overflow_context = LiveContext::new(
            crate::domain_computation::primary_graph::tests::fixture::live_scope(),
        );
        let mut overflow = overflow_context
            .world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                overflow_context.open(&host_custody)
            })
            .expect("the fixture read owns its installed host call");
        overflow_context.overflow_source();
        let foreign = overflow_context
            .world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                overflow_context.principal(&host_custody, "bob")
            })
            .expect("the fixture read owns its installed host call");
        overflow_context
            .world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();
                assert!(matches!(
                    {
                        let installed_fixture_phase = overflow.fixture_phase(&host_custody);
                        overflow.next(
                            &installed_fixture_phase,
                            &foreign,
                            &overflow_context.request,
                        )
                    },
                    WorthQueryApplicationLiveOutcome::AuthorizationDenied(_)
                ))
            })
            .expect("the fixture read owns its installed host call");
        drop(overflow);
        overflow_context.assert_source_released();
    }

    fn observer(lease: &TestLiveLease<'_>) -> BridgeExecutionBasisLifecycleObserver {
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
