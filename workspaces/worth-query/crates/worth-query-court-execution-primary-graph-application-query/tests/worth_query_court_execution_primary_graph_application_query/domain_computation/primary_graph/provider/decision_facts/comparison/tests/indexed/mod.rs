use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::decision_facts::comparison::indexed::PreparedSelections;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::indexed_entity_selection::observe_indexed_entity_selection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation_on_application;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetDenialKind;
use std::collections::BTreeMap;
use worth_execution::ExecutionAllocationPolicy;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

#[test]
fn scoped_indexed_comparison_preserves_pinned_absence_limits_and_foreign_affinity() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let foreign = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let control =
        StorageControl::new_for_test(ExecutionAllocationPolicy::SystemAllocation, Some(&request));
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let selected = world.selected_product(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
            );
            let snapshot = selected.application_basis_for_test().snapshot_handle();
            let graph = (*world.application.runtime_for_test())
                .primary_graph_for_test()
                .unwrap();
            let field = AccountStatus::at("AccountStatus");
            let layout = (*graph.layout_for_test())
                .equality_field_for_test(field.entity(), field.aspect(), field.field())
                .unwrap();
            let index = (*layout.equality_index_id_for_test()).unwrap();
            let locator = (*layout.locator_for_test()).clone();
            let kind = *layout.entity_kind_for_test();
            let observe = |value: &str| {
                graph
                    .integration_handle_for_test()
                    .with_runtime(&borrowed_execution, |runtime| {
                        observe_indexed_entity_selection(
                            runtime,
                            execution,
                            snapshot,
                            index,
                            kind,
                            locator.clone(),
                            StringApplicationValueBinding::encode(&value.to_owned()).unwrap(),
                            2,
                        )
                        .unwrap()
                    })
            };
            let present = observe("open");
            let missing = observe("absent");
            let unrelated = observe("unrelated");
            let Fact::IndexedEntitySelection {
                candidates: open, ..
            } = &present
            else {
                unreachable!()
            };
            let Fact::IndexedEntitySelection {
                candidates: other, ..
            } = &unrelated
            else {
                unreachable!()
            };
            assert_eq!(open.len(), 1);
            assert_eq!(other.len(), 1);
            let mut selectors = PreparedSelections::new_for_test();
            for fact in [&present, &missing, &unrelated] {
                graph
                    .integration_handle_for_test()
                    .with_runtime(&borrowed_execution, |runtime| {
                        assert!(fact
                            .remains_equal_in_for_test(runtime, execution, snapshot)
                            .unwrap());
                        assert!(selectors
                            .remains_equal_for_test(
                                execution, fact, runtime, snapshot, None, control
                            )
                            .unwrap());
                    });
            }
            assert_eq!(
                (*selectors.selectors_for_test()).len(),
                1,
                "all distinct predicates share only the exact index admission"
            );
            (*(*(*foreign.application.runtime_for_test()).primary_provider_for_test())
                .graph_for_test())
            .with_runtime(&borrowed_execution, |runtime| {
                assert!(!selectors
                    .remains_equal_for_test(execution, &present, runtime, snapshot, None, control)
                    .unwrap());
            });
            publish_relational_mutation_on_application(
                execution,
                &world.application,
                WorkerIntentBatch::new("comparison-phantom").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: other[0],
                        fields: AspectFieldPatch::from(BTreeMap::from([(
                            locator.clone(),
                            StringApplicationValueBinding::encode(&"open".to_owned()).unwrap(),
                        )])),
                    }),
                )),
            );
            // The old comparison remains pinned. A newly selected root sees the phantom.
            graph
                .integration_handle_for_test()
                .with_runtime(&borrowed_execution, |runtime| {
                    assert!(selectors
                        .remains_equal_for_test(
                            execution, &present, runtime, snapshot, None, control
                        )
                        .unwrap())
                });
            let current = world.selected_product(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
            );
            let current_snapshot = current.application_basis_for_test().snapshot_handle();
            let mut current_selectors = PreparedSelections::new_for_test();
            graph
                .integration_handle_for_test()
                .with_runtime(&borrowed_execution, |runtime| {
                    assert!(!current_selectors
                        .remains_equal_for_test(
                            execution,
                            &present,
                            runtime,
                            current_snapshot,
                            None,
                            control
                        )
                        .unwrap());
                    let complete = observe_indexed_entity_selection(
                        runtime,
                        execution,
                        current_snapshot,
                        index,
                        kind,
                        locator.clone(),
                        StringApplicationValueBinding::encode(&"open".to_owned()).unwrap(),
                        2,
                    )
                    .unwrap();
                    let Fact::IndexedEntitySelection { candidates, .. } = &complete else {
                        unreachable!()
                    };
                    assert_eq!(candidates.len(), 2);
                    assert!(current_selectors
                        .remains_equal_for_test(
                            execution,
                            &complete,
                            runtime,
                            current_snapshot,
                            None,
                            control
                        )
                        .unwrap());
                    for limit in [0, 1, usize::MAX] {
                        let mut narrowed = complete.clone();
                        let Fact::IndexedEntitySelection {
                            candidate_limit, ..
                        } = &mut narrowed
                        else {
                            unreachable!()
                        };
                        *candidate_limit = limit;
                        assert!(
                            !current_selectors
                                .remains_equal_for_test(
                                    execution,
                                    &narrowed,
                                    runtime,
                                    current_snapshot,
                                    None,
                                    control
                                )
                                .unwrap(),
                            "invalid or overflowing selection must never reuse its old candidates"
                        );
                    }
                });
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn prepared_indexed_comparison_keeps_a_native_cancellation() {
    use std::cell::Cell;
    use std::rc::Rc;
    use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let source = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        None,
        source.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world.application.with_application_advancement(&scope, |phase| {
        let execution = phase.checked_execution_request_for(world.application.product_runtime()).unwrap();
        let selected = world.selected_product(&phase.test_host_request_for_test());
        let snapshot = selected.application_basis_for_test().snapshot_handle();
        let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
        let field = (*graph.layout_for_test()).equality_field_for_test("Account", worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus::at("AccountStatus").aspect(), "AccountStatus").unwrap();
        let (answer, seen) = graph.integration_handle_for_test().with_runtime(&execution, |runtime| {
            let fact = observe_indexed_entity_selection(runtime, &execution, snapshot,
                (*field.equality_index_id_for_test()).unwrap(), *field.entity_kind_for_test(), (*field.locator_for_test()).clone(),
                StringApplicationValueBinding::encode(&"open".to_owned()).unwrap(), 2).unwrap();
            let mut prepared = PreparedSelections::new_for_test();
            let control = StorageControl::new_for_test(ExecutionAllocationPolicy::SystemAllocation, None);
            assert!(prepared.remains_equal_for_test(&execution, &fact, runtime, snapshot, None, control).unwrap());
            let seen = Rc::new(Cell::new(0));
            let contacts = Rc::clone(&seen);
            // Query entry + Native admission precede the Native collector.
            // Cancellation is raised at that collector's first consultation.
            const NATIVE_COLLECTION_CONSULT: usize = 1 + 1 + 1;
            let _probe = worth_execution::ExecutionCheckpointObservationForTest::install_on_this_thread_for_test(move || {
                contacts.set(contacts.get() + 1);
                if contacts.get() == NATIVE_COLLECTION_CONSULT { source.cancel(); }
            });
            let answer = prepared.remains_equal_for_test(&execution, &fact, runtime, snapshot, None, control);
            (answer, seen.get())
        });
        assert_eq!(seen, 3);
        let stopped = answer.unwrap_err();
        assert_eq!(stopped.kind(), WorthQueryDecisionReadSetDenialKind::RequestInterrupted(
            WorthQueryRequestInterruption::Cancelled));
    }).unwrap();
}
