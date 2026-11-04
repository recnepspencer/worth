pub use std::collections::BTreeMap;
pub use std::num::NonZeroUsize;
pub use std::time::Duration;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::movement::Movement;
pub use worth_foundational::facade::AspectFieldLocator;
pub use worth_foundational::facade::CanonicalFieldPath;
pub use worth_foundational::facade::LocatorAuthority;

pub use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
pub use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
pub use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::current_controls;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CrossRootQuery;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryBasisPosture;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn empty_indexed_root_set_stales_when_its_scoped_guard_becomes_a_match() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
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
            world
                .selected_product(&(phase).test_host_request_for_test())
                .resolve_entity(
                    &phase,
                    AccountIdentity::reference(),
                    "account-1".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_query::installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let parameters = || {
        ApplicationQueryParameterSet::new()
            .bind(status_parameter(), "closed".to_string())
            .unwrap()
    };
    let initial = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let empty = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    initial
                        .admit_application_query(
                            &phase,
                            &query,
                            &access,
                            parameters(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert!(empty.rows().is_empty());
    assert!(empty.observed_sources().is_empty());
    let selection = empty.result_set_observation().selection_for_test();
    assert_eq!((*selection.aspects_for_test()).len(), 1);
    let guard = &(*selection.aspects_for_test())[0];
    assert_eq!((*guard.entity_for_test()), account.entity_id_for_test());
    let fact = WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: (*guard.entity_for_test()),
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            (*guard.aspect_for_test()).clone(),
            CanonicalFieldPath::single((*guard.field_for_test()).clone()),
        ),
        native_revision: guard.native_revision,
    };
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            change_account_status(
                &host_custody.borrowed_request_for_test(),
                &world,
                account.entity_id_for_test(),
                "closed",
            )
        })
        .expect("the fixture read owns its installed host call");
    let current = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(!graph.integration_handle_for_test().with_runtime(
                request_execution,
                |runtime| {
                    fact.source_currentness_in_for_test(
                        runtime,
                        &host_custody.borrowed_request_for_test(),
                        current.application_basis_for_test().snapshot_handle(),
                    )
                    .unwrap()
                    .unwrap()
                    .movement_for_test()
                        == Movement::Unmoved
                }
            ))
        })
        .expect("the fixture read owns its installed host call");
    let matching = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    current
                        .admit_application_query(
                            &phase,
                            &query,
                            &access,
                            parameters(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(matching.rows().len(), 1);
    assert_ne!(
        empty
            .result_set_observation()
            .idempotency_identity_for_test(),
        matching
            .result_set_observation()
            .idempotency_identity_for_test()
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            change_account_status(
                &host_custody.borrowed_request_for_test(),
                &world,
                account.entity_id_for_test(),
                "open",
            )
        })
        .expect("the fixture read owns its installed host call");
    let reopened = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert!(!graph.integration_handle_for_test().with_runtime(
                request_execution,
                |runtime| {
                    fact.source_currentness_in_for_test(
                        runtime,
                        &host_custody.borrowed_request_for_test(),
                        reopened.application_basis_for_test().snapshot_handle(),
                    )
                    .unwrap()
                    .unwrap()
                    .movement_for_test()
                        == Movement::Unmoved
                }
            ))
        })
        .expect("the fixture read owns its installed host call");
    let reopened_result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(
                    &phase,
                    reopened
                        .admit_application_query(
                            &phase,
                            &query,
                            &access,
                            parameters(),
                            current_controls(&request),
                        )
                        .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert!(reopened_result.rows().is_empty());
    assert_eq!(reopened_result.rows(), empty.rows());
}

pub fn change_account_status(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    account: worth_relational::facade::identity::EntityId,
    status: &str,
) {
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap();
    let field = AccountStatus::reference();
    let locator = (*graph.layout_for_test())
        .field_locator_for_test(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        locator,
        StringApplicationValueBinding::encode(&status.to_string()).unwrap(),
    )]));
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation(
        execution,
        world,
        WorkerIntentBatch::new("change-status-after-product-selection").push(
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: account,
                    fields,
                },
            )),
        ),
    );
}

#[path = "root_path_guard_reads_its_pinned_truth_version/mod.rs"]
pub mod root_path_guard_reads_its_pinned_truth_version;
