use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::current_output::resolution::cardinality::classify_current_entities;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::current_output::resolution::cardinality::CurrentOutputCardinality;

#[test]
fn current_role_cardinality_ignores_repeated_identity_but_not_distinct_outputs() {
    assert_eq!(
        classify_current_entities(Vec::<u64>::new()),
        CurrentOutputCardinality::Missing
    );
    assert_eq!(
        classify_current_entities([11, 11]),
        CurrentOutputCardinality::Unique(11)
    );
    assert_eq!(
        classify_current_entities([11, 11, 12, 12]),
        CurrentOutputCardinality::Ambiguous(vec![11, 12])
    );
}

#[test]
fn postcommit_observation_cannot_certify_original_own_write_output() {
    {
        use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write;
        use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
        use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
        use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
        use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::retained_output_capacity::ReadRetainedAccount;
        use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::retained_output_capacity::RetainedFamily;
        use worth_query_execution::facade::domain_computation::WorthQueryCurrentOutputDenialKind;
        use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
        with_committed_own_write(|world, receipt, _, _| {
            world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request_for_test();
                    let request = live_scope();
                    let principal = authenticated_principal(&host_custody, world, &request);
                    let account = resolved_account(&host_custody, world, "2", &request);
                    let graph = (*world.application.runtime_for_test())
                        .primary_graph_for_test()
                        .unwrap();
                    let handle = graph.integration_handle_for_test();
                    handle
                        .output_lineage
                        .lock()
                        .unwrap()
                        .install_output_families_for_test(std::collections::BTreeMap::from([(
                            "test.retained-account-capacity".to_owned().into(),
                            vec![(
                                receipt.output_correspondence().binding_type().unwrap(),
                                "retained-account".to_owned(),
                            )],
                        )]));
                    let operation = world
                        .application
                        .installed_schema()
                        .installed_operation(ReadRetainedAccount::reference())
                        .unwrap();
                    let admitted = world
                        .selected_product(
                            &(host_custody.phase_for_application_for_test(&world.application))
                                .test_host_request_for_test(),
                        )
                        .authorize_operation(
                            &host_custody.phase_for_application_for_test(&world.application),
                            &principal,
                            &account,
                            &operation,
                            TypedMutationPreconditions::new(),
                            &request,
                        )
                        .unwrap();
                    let answer = world
                        .invariant
                        .project_admitted_operation(
                            &host_custody.phase_for_application_for_test(&world.application),
                            &admitted,
                            |reader, root| {
                                reader
                                    .current_output::<RetainedFamily, _>(root)
                                    .map(|_| ())
                                    .map_err(|denial| denial.kind())
                            },
                            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                        )
                        .unwrap()
                        .into_parts()
                        .0;
                    assert_eq!(
                answer,
                Err(WorthQueryCurrentOutputDenialKind::StaleSource),
                "rebased x=2 and an unchanged witness cannot certify output computed from x=1"
            );
                })
                .expect("the fixture reads borrow their installed world phase");
        });
    }
}
