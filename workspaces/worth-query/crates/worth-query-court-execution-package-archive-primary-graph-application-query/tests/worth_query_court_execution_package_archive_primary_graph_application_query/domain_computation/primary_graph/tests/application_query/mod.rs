pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::current_controls;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::installed_forged_selector_query;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::installed_governed_query;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::installed_query;

pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
pub use std::num::NonZeroUsize;
pub use std::time::Duration;
pub use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_two_principal_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountSummaryQuery;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryResult;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ForgedSelectorQuery;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::forged_selector::ForgedSelectorResult;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::GovernedAccountSummaryQuery;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::LiveAccountActivityQuery;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::live_account_query::LiveAccountActivityResult;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::NestedAccountQuery;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::nested_account::NestedAccountResult;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::OrderedAccountSummaryQuery;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub(crate) use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::observed_source_fixture::observed_account_source;
#[test]
fn execution_runtime_mints_plan_from_exact_mapped_principal_and_typed_scope() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
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
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);

    let plan = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .unwrap(),
                    current_controls(&request),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(plan.query_identity(), query.identity());
    assert_eq!(plan.scope().binding_identity(), query.binding_identity());
    assert!(plan.graph_read_plan().is_admitted());
    assert_eq!(plan.graph_work_branch(), plan.basis_identity().branch_id());
    assert_eq!(
        plan.graph_work_principal_entity_id(),
        principal.principal_entity_id()
    );
    assert_eq!(plan.graph_work_scope_entity_id(), Some(account.entity_id()));
    assert!(plan.graph_work_runtime_ordinal() > 0);
    assert!(plan.graph_work_managed_run_identity().as_u64() > 0);
    assert!(plan.graph_work_decision_fact_count() > 0);
    assert!(!plan.graph_work_provider().is_empty());

    let result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].status(), "open");
    assert_eq!(result.rows()[0].label(), "primary");
    let source = result.observed_sources()[0].footprint_for_test();
    assert_eq!(
        source.aspects.len(),
        2,
        "each consumed field is recorded once even when both share one aspect"
    );
    assert_eq!(result.receipt().query_identity(), query.identity());
    assert_eq!(result.receipt().projected_record_count(), 1);
    assert_eq!(result.receipt().projected_field_count(), 2);
    assert!(
        result.receipt().reads().field_reads() > result.receipt().projected_field_count(),
        "the root guard reads a field in addition to result projection"
    );
    assert_eq!(result.receipt().reads().index_candidates_examined(), 0);
    assert_eq!(result.receipt().fallback_count(), 0);
    assert_eq!(result.receipt().reads().adjacency_edges_inspected(), 0);
    assert_eq!(result.receipt().per_result_neighbor_lookup_count(), 0);
    assert!(result.receipt().basis_released());
}
#[test]
fn mapped_stranger_cannot_admit_a_valid_foreign_account_scope() {
    let world = installed_two_principal_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("bob", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
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
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .select_product_branch(
                    &host_custody.borrowed_request_for_test(),
                    world.application.product_runtime().default_branch(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);

    let denial = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .unwrap(),
                    current_controls(&request),
                )
                .err()
                .expect("a mapped stranger must not receive query plan authority")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
            worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::PermissionDenied),
        )
    );
}

#[path = "correction_disclosure/mod.rs"]
pub mod correction_disclosure;
#[path = "derived_view/mod.rs"]
pub mod derived_view;
#[path = "disclosure_noninterference/mod.rs"]
pub mod disclosure_noninterference;
#[path = "foreign_scope_and_missing_disclosure_governance_open_no_plan_aut/mod.rs"]
pub mod foreign_scope_and_missing_disclosure_governance_open_no_plan_aut;
#[path = "freshness_stops/mod.rs"]
pub mod freshness_stops;
#[path = "graph_work_capacity/mod.rs"]
pub mod graph_work_capacity;
#[path = "identity_convergence/mod.rs"]
pub mod identity_convergence;
#[path = "lifecycle/mod.rs"]
pub mod lifecycle;
#[path = "lifecycle_mutations/mod.rs"]
pub mod lifecycle_mutations;
#[path = "optional_result_presence/mod.rs"]
pub mod optional_result_presence;
#[path = "ordering_support/mod.rs"]
pub mod ordering_support;
#[path = "result_projection/mod.rs"]
pub mod result_projection;
#[path = "root_guard_basis/mod.rs"]
pub mod root_guard_basis;
#[path = "root_selection/mod.rs"]
pub mod root_selection;
#[path = "runtime_support/mod.rs"]
pub mod runtime_support;
#[path = "scope_liveness/mod.rs"]
pub mod scope_liveness;
#[path = "shared_basis/mod.rs"]
pub mod shared_basis;
