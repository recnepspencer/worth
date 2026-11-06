pub use std::time::Duration;
pub use std::time::UNIX_EPOCH;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationLiveCloseOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationLiveControls;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationLiveOutcome;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_access_fixture::admit_touch_account_capability;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::governed_live_account_parameters;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_live_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::live_account_parameters;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_status_mutation::revoke_current_capability;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Activity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::GovernedLiveAccountActivityCause;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedLiveAccountActivityQuery;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::governed_live_query::GovernedLiveAccountActivityResult;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::LiveAccountActivityCause;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::LiveAccountActivityQuery;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::LiveAccountActivityResult;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::live_delivery_support::commit_live_activity;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationBasisSelectionIdentity;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationDisclosed;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationDisclosureReceiptPosture;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn governed_live_delivery_reuses_only_query_owned_current_authority() {
    let world = installed_capability_live_world();
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let committer_external = world.authenticate("bob", Duration::from_secs(60), &request);
    let committer = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &committer_external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = world
        .application
        .installed_schema()
        .certification_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap();
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountIdentity::at("AccountIdentity"),
                    "account-1".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let capability = world
        .application
        .with_host_advancement(|phase| {
            admit_touch_account_capability(
                &(phase).test_host_request_for_test(),
                &world,
                &principal,
                &request,
            )
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let mut lease = world
        .application
        .with_host_advancement(|phase| {
            world
        .selected_product(&(phase).test_host_request_for_test(), )
        .open_governed_application_query_live::<
            GovernedLiveAccountActivityQuery,
            AccountSummaryParameters,
            GovernedLiveAccountActivityResult,
            _,
            _,
            Account,
            Activity,
            GovernedLiveAccountActivityCause,
            _,
            _,
            _,
        >(&phase,
            query,
            &principal,
            account,
            capability,
            governed_live_account_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(request.clone(), 4, 16).unwrap(),
        )
        .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let committed = commit_live_activity(&world, &committer, &request);

    let WorthQueryApplicationLiveOutcome::Delivered(update) = world
        .application
        .with_host_advancement(|phase| lease.next(&phase, &principal, &request))
        .expect("the fixture read owns its installed host call")
    else {
        panic!("current governed authority must deliver the committed cause");
    };
    assert_eq!(
        update.product_publication(),
        committed.committed_product_publication()
    );
    assert_eq!(update.result().account(), "account-1");
    assert_eq!(
        update.result().activities(),
        &[("activity-primary".to_owned(), 11)]
    );
    assert!(matches!(
        update.result().label(),
        WorthQueryApplicationDisclosed::Omitted(_)
    ));
    assert_eq!(
        update.receipt().disclosure().posture(),
        WorthQueryApplicationDisclosureReceiptPosture::Governed
    );
}

#[test]
fn revoked_capability_stops_governed_live_delivery_before_projection() {
    let world = installed_capability_live_world();
    world
        .authorization_time
        .script(vec![UNIX_EPOCH + Duration::from_secs(100); 32]);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let committer_external = world.authenticate("bob", Duration::from_secs(60), &request);
    let committer = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &committer_external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = world
        .application
        .installed_schema()
        .certification_query(GovernedLiveAccountActivityQuery::reference())
        .unwrap();
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountIdentity::at("AccountIdentity"),
                    "account-1".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let capability = world
        .application
        .with_host_advancement(|phase| {
            admit_touch_account_capability(
                &(phase).test_host_request_for_test(),
                &world,
                &principal,
                &request,
            )
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let mut lease = world
        .application
        .with_host_advancement(|phase| {
            world
        .selected_product(&(phase).test_host_request_for_test(), )
        .open_governed_application_query_live::<
            GovernedLiveAccountActivityQuery,
            AccountSummaryParameters,
            GovernedLiveAccountActivityResult,
            _,
            _,
            Account,
            Activity,
            GovernedLiveAccountActivityCause,
            _,
            _,
            _,
        >(&phase,
            query,
            &principal,
            account,
            capability,
            governed_live_account_parameters("account-1"),
            WorthQueryApplicationLiveControls::bounded(request.clone(), 4, 16).unwrap(),
        )
        .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    commit_live_activity(&world, &committer, &request);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            revoke_current_capability(
                &(phase).test_host_request_for_test(),
                &host_custody.borrowed_request_for_test(),
                &world,
            )
        })
        .expect("the fixture read owns its installed host call");

    let outcome = world
        .application
        .with_host_advancement(|phase| lease.next(&phase, &principal, &request))
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationLiveOutcome::AuthorizationDenied(denial) = outcome else {
        let posture = match outcome {
            WorthQueryApplicationLiveOutcome::Delivered(_) => "delivered",
            WorthQueryApplicationLiveOutcome::Pending => "pending",
            WorthQueryApplicationLiveOutcome::Overflow(_) => "overflow",
            WorthQueryApplicationLiveOutcome::StalePrincipal => "stale-principal",
            WorthQueryApplicationLiveOutcome::StaleScope => "stale-scope",
            WorthQueryApplicationLiveOutcome::ProjectionDenied(_) => "projection-denied",
            WorthQueryApplicationLiveOutcome::CauseDenied(_) => "cause-denied",
            WorthQueryApplicationLiveOutcome::Cancelled => "cancelled",
            WorthQueryApplicationLiveOutcome::DeadlineExceeded => "deadline-exceeded",
            WorthQueryApplicationLiveOutcome::Closed => "closed",
            WorthQueryApplicationLiveOutcome::Unavailable => "unavailable",
            WorthQueryApplicationLiveOutcome::ExecutionRequest(_) => "execution-request",
            WorthQueryApplicationLiveOutcome::AuthorizationDenied(_) => unreachable!(),
        };
        panic!("revoked governed live authority returned {posture}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleAuthorization
        )
    );
    assert!(denial.identity().is_some());
    assert_eq!(
        denial.causes(),
        [WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleAuthorization
        )]
    );
}

#[path = "committed_live_cause_projects_with_bounded_result_buffer_evidenc/mod.rs"]
pub mod committed_live_cause_projects_with_bounded_result_buffer_evidenc;
