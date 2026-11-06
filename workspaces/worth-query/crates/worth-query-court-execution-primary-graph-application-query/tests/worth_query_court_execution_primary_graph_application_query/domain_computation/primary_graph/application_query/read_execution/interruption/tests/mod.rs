pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::read_execution::interruption::checkpoint_observer::observe_checkpoints_for_test;
pub use std::cell::Cell;
pub use std::cell::RefCell;
pub use std::num::NonZeroUsize;
pub use std::rc::Rc;
pub use std::time::Duration;

pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryResourceProfile;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world_with_resource_profile;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::NestedAccountQuery;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
pub use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

pub fn nz(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap()
}

#[test]
fn cancellation_during_real_nested_materialization_releases_all_read_resources() {
    let world = installed_authorization_world_with_resource_profile(
        WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(64).expect("concurrent graph work is non-zero"),
        ),
    );
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let principal = world
        .application
        .with_host_advancement(|phase| {
            selected
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
    let scope = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = world
        .application
        .installed_schema()
        .certification_query(NestedAccountQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            selected
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_owned())
                        .unwrap(),
                    WorthQueryProductQueryControls::new(nz(10), &request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let triggered = Rc::new(Cell::new(false));
    let witness = Rc::clone(&triggered);
    let _guard = observe_checkpoints_for_test(Box::new(move |subject| {
        if !witness.get() && subject.contains("relation") {
            witness.set(true);
            cancellation.cancel();
        }
    }));
    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .err()
                .expect("cancelled read cannot publish rows")
        })
        .expect("the fixture read owns its installed host call");
    assert!(
        triggered.get(),
        "must cancel inside nested traversal at a real nested relation checkpoint"
    );
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationOneShotDenialKind::Cancelled
    );

    assert_eq!(
        world
            .application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
    assert_eq!(
        world.application.provider_session_resource_count_for_test(),
        0
    );
}

#[test]
fn deadline_is_checked_at_kernel_safe_points() {
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now()),
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let denial = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::read_execution::interruption::checkpoint(&request, "root/relation[0]").unwrap_err();
    assert_eq!(
        denial.kind(),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::read_execution::denial::WorthQueryApplicationReadExecutionDenialKind::DeadlineExceeded
    );
}

#[path = "closure/mod.rs"]
pub mod closure;
#[path = "declared_components/mod.rs"]
pub mod declared_components;
#[path = "declared_orbit/mod.rs"]
pub mod declared_orbit;
