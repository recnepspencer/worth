pub use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
pub use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
pub use worth_query_execution::facade::domain_computation::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
pub use worth_relational::facade::indexes::DerivedIndexBuildRequest;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_branch::primary_relational_branch_id;
pub use worth_query_execution::facade::domain_computation::WorthQueryEntityResolutionDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn equal_version_snapshot_from_another_relational_runtime_is_rejected() {
    let first = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let second = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let first_graph = (*first.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let second_graph = (*second.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let installed = first_graph.retain_entity_resolution_context();
    let first_version = first
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let first_product = first.selected_product(
                &(host_custody.phase_for_application_for_test(&first.application))
                    .test_host_request_for_test(),
            );

            first_graph
                .integration_handle_for_test()
                .with_runtime_mut_for_test(&borrowed_execution, |_| {
                    first_product
                        .application_basis_for_test()
                        .snapshot_handle()
                        .version_id()
                })
        })
        .expect("the source version read owns its host call");
    second
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let _execution = &borrowed_execution;
            let second_product = second.selected_product(
                &(host_custody.phase_for_application_for_test(&second.application))
                    .test_host_request_for_test(),
            );
            second_graph
                .integration_handle_for_test()
                .with_runtime_mut_for_test(&borrowed_execution, |runtime| {
                    let snapshot = second_product
                        .application_basis_for_test()
                        .snapshot_handle();
                    assert_eq!(snapshot.version_id(), first_version);
                    let denial = match installed.at_snapshot_for_test(
                        runtime,
                        snapshot,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    ) {
                        Ok(_) => panic!("foreign runtime truth entered entity resolution"),
                        Err(denial) => denial,
                    };
                    assert_eq!(
                        denial.kind(),
                        WorthQueryEntityResolutionDenialKind::Refused(
                            ResolutionRefusal::ForeignResolutionTruth
                        )
                    );
                });
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn rebuilt_index_generation_preserves_stable_entity_meaning() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let identity = world
                .application
                .select_product_branch_for_test(
                    execution,
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &host_custody.phase_for_application_for_test(&world.application),
                    AccountStatus::at("AccountStatus"),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let graph = (*world.application.runtime_for_test())
                .primary_graph_for_test()
                .unwrap();
            let installed = graph.retain_entity_resolution_context();
            let selected = world.selected_product(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
            );
            let index_id = graph
                .layout_for_test()
                .equality_field_for_test("Account", worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus::at("AccountStatus").aspect(), "AccountStatus")
                .and_then(|field| (*field.equality_index_id_for_test()))
                .unwrap();

            graph
                .integration_handle_for_test()
                .with_runtime_mut_for_test(&borrowed_execution, |runtime| {
                    let head = runtime
                        .history()
                        .historical_latest_commit()
                        .unwrap()
                        .clone();
                    let build = runtime.index_authority().build_for_commit(
                        execution,
                        DerivedIndexBuildRequest {
                            source_commit_id: head.commit_id,
                            branch_id: primary_relational_branch_id(),
                            index_ids: vec![index_id],
                        },
                    );
                    assert!(build.failed_indexes.is_empty());
                    assert_ne!(
                        build.generations[0].generation_id,
                        identity.identity_index_generation().expect("committed resolution retains its actual index generation")
                    );
                    let snapshot = selected.application_basis_for_test().snapshot_handle();
                    let truth = installed
                        .at_snapshot_for_test(
                            runtime,
                            snapshot,
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .unwrap();
                    truth
                        .validate_entity_freshness_for_test(execution, &identity)
                        .unwrap();
                });
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn installed_context_derives_binding_layout_and_index_from_its_graph() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let installed = graph.retain_entity_resolution_context();
    world.application.with_host_advancement(|phase| {
let host_custody = phase.test_host_request_for_test();
let borrowed_execution = host_custody.borrowed_request_for_test();
let execution = &borrowed_execution;
let selected = world.selected_product(
            &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
        );
        graph.integration_handle_for_test().with_runtime_mut_for_test(&borrowed_execution, |runtime| {
        let snapshot = selected.application_basis_for_test().snapshot_handle();
        let truth = installed
            .at_snapshot_for_test( runtime, snapshot, WorthQueryPrincipalResolutionMode::Ordinary)
            .unwrap();
        let resolved = truth
            .resolve_for_test(execution,
                "Account",
                AccountStatus::at("AccountStatus").aspect(),
                "AccountStatus",
                StringApplicationValueBinding::encode(&"open".to_owned()).unwrap(),
            )
            .unwrap();
        let typed = resolved.into_application_identity_for_test::<IdentityExecutionSchema, worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account>();
        assert_eq!(typed.binding_identity(), graph.binding_identity());
        assert_eq!(typed.identity_index_id_for_test(), (*graph.layout_for_test().equality_field_for_test("Account", worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus::at("AccountStatus").aspect(), "AccountStatus").unwrap().equality_index_id_for_test()).unwrap());
    });

}).expect("the fixture block owns its installed host call")
}

#[path = "lookup_stops/mod.rs"]
pub mod lookup_stops;
