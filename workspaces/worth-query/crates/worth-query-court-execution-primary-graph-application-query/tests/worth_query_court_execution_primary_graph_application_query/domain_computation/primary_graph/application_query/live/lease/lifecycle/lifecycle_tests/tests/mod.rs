use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::context::court_doors_for_test::ContextDoorsOps;
pub use std::time::Duration;

pub use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub use worth_runtime_bridge::facade::BridgeExecutionBasisLifecycleObserver;
pub use worth_runtime_bridge::facade::BridgeExecutionBasisLifecycleSignalStatus;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_two_principal_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::live_account_parameters;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Activity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::LiveAccountActivityCause;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::LiveAccountActivityQuery;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::LiveAccountActivityResult;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::live_delivery_support::commit_live_activity_with_identity;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationLiveCloseOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationLiveControls;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationLiveLease;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationLiveOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub type TestLiveLease<'runtime> = WorthQueryApplicationLiveLease<
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

pub struct LiveContext {
    pub world: AuthorizationWorld,
    pub request: WorthQueryRequestScope,
    pub principal: WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
}

impl LiveContext {
    pub fn new(request: WorthQueryRequestScope) -> Self {
        let world = installed_two_principal_authorization_world(true);
        let external = world.authenticate("alice", Duration::from_secs(60), &request);
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase;
                let borrowed_execution = phase
                    .execution_request_for_for_test(world.application.product_runtime())
                    .expect("the installed fixture lends its own phase");
                let execution = &borrowed_execution;

                world
                    .application
                    .select_product_branch_for_test(
                        execution,
                        world
                            .application
                            .product_runtime()
                            .default_branch_for_test(),
                    )
                    .expect("the selected product branch remains admitted")
                    .resolve_authenticated_principal(
                        phase_custody,
                        &world.binding,
                        &external,
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap()
            })
            .expect("the fixture host admits its identity reads");
        Self {
            world,
            request,
            principal,
        }
    }

    pub fn already_expired_request(&self) -> WorthQueryRequestScope {
        let live_cancellation = WorthQueryCancellationSource::new();
        // Instant deadline comparison is `now >= deadline`, so the
        // current instant makes the next delivery sample DeadlineExceeded
        // without sleeping or racing open.
        WorthQueryRequestScope::new(Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now()), live_cancellation.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system())
    }

    pub fn principal(
        &self,
        phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
        identity: &str,
    ) -> WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64> {
        let borrowed_execution = phase_custody.borrowed_request_for_test();
        let execution = &borrowed_execution;

        let external = self
            .world
            .authenticate(identity, Duration::from_secs(60), &self.request);
        self.world
            .application
            .select_product_branch_for_test(
                execution,
                self.world
                    .application
                    .product_runtime()
                    .default_branch_for_test(),
            )
            .expect("the selected product branch remains admitted")
            .resolve_authenticated_principal(
                &phase_custody.phase_for_application_for_test(&self.world.application),
                &self.world.binding,
                &external,
                &self.request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap()
    }

    pub fn open(
        &self,
        phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
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
            .select_product_branch_for_test(
                execution,
                self.world
                    .application
                    .product_runtime()
                    .default_branch_for_test(),
            )
            .expect("the selected product branch remains admitted")
            .resolve_entity(
                &phase_custody.phase_for_application_for_test(&self.world.application),
                AccountIdentity::reference(),
                "account-1".to_owned(),
                &self.request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        self.world
            .selected_product(&(phase_custody).test_host_request_for_test(), )
            .open_application_query_live::<
                LiveAccountActivityQuery,
                AccountSummaryParameters,
                LiveAccountActivityResult,
                Principal,
                u64,
                Account,
                Activity,
                LiveAccountActivityCause,
            >(&phase_custody.phase_for_application_for_test(&self.world.application),
                query,
                &self.principal,
                account,
                live_account_parameters("account-1"),
                WorthQueryApplicationLiveControls::bounded(self.request.clone(), 4, 16)
                .unwrap(),
            )
            .unwrap()
    }

    pub fn close_source(&self) {
        (*(*(*self.world.application.runtime_for_test()).primary_provider_for_test())
            .live_delivery_for_test())
        .close_for_test();
    }

    pub fn overflow_source(&self) {
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

    pub fn assert_source_released(&self) {
        assert_eq!(
            (*(*self.world.application.runtime_for_test()).primary_provider_for_test())
                .active_application_commit_causality_partitions_for_test(),
            0
        );
    }
}

#[test]
fn explicit_close_releases_bridge_signal_and_queue_resources() {
    let context =
        LiveContext::new(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope());
    let lease = context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
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

#[test]
fn fresh_principal_admission_precedes_empty_and_overflow_progress() {
    let empty_context =
        LiveContext::new(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope());
    let mut empty = empty_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            empty_context.open(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    let foreign = empty_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            empty_context.principal(&host_custody, "bob")
        })
        .expect("the fixture read owns its installed host call");
    empty_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(matches!(
                {
                    let installed_fixture_phase = empty.fixture_phase_for_test(&host_custody);
                    empty.next(&installed_fixture_phase, &foreign, &empty_context.request)
                },
                WorthQueryApplicationLiveOutcome::AuthorizationDenied(_)
            ))
        })
        .expect("the fixture read owns its installed host call");
    drop(empty);
    empty_context.assert_source_released();

    let overflow_context =
        LiveContext::new(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope());
    let mut overflow = overflow_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            overflow_context.open(&host_custody)
        })
        .expect("the fixture read owns its installed host call");
    overflow_context.overflow_source();
    let foreign = overflow_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            overflow_context.principal(&host_custody, "bob")
        })
        .expect("the fixture read owns its installed host call");
    overflow_context
        .world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(matches!(
                {
                    let installed_fixture_phase = overflow.fixture_phase_for_test(&host_custody);
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

pub fn observer(lease: &TestLiveLease<'_>) -> BridgeExecutionBasisLifecycleObserver {
    let basis = (*lease.basis_for_test())
        .as_ref()
        .expect("open lease retains its basis");
    basis.bridge.lifecycle_observer()
}

pub fn assert_live(bridge: &BridgeExecutionBasisLifecycleObserver) {
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

pub fn assert_terminal(
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

#[path = "cancellation_and_deadline_terminalize_all_live_resources/mod.rs"]
pub mod cancellation_and_deadline_terminalize_all_live_resources;
#[path = "source_close_overflow_and_abandoned_drop_release_all_live_resour/mod.rs"]
pub mod source_close_overflow_and_abandoned_drop_release_all_live_resour;
