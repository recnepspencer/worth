use super::super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        observe_indexed_entity_selection, WorthQueryApplicationObservedFact as Fact,
    },
    tests::fixture::{
        installed_authorization_world, live_scope, publish_relational_mutation_on_application,
        AccountStatus,
    },
};
use worth_execution::ExecutionAllocationPolicy;
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

#[test]
fn scoped_indexed_comparison_preserves_pinned_absence_limits_and_foreign_affinity() {
    let world = installed_authorization_world(true);
    let foreign = installed_authorization_world(true);
    let request = live_scope();
    let control = StorageControl::new(ExecutionAllocationPolicy::SystemAllocation, Some(&request));
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let selected = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            let snapshot = selected.application_basis().snapshot_handle();
            let graph = world.application.runtime.primary_graph().unwrap();
            let field = AccountStatus::reference();
            let layout = graph
                .layout
                .equality_field(field.entity(), field.aspect(), field.field())
                .unwrap();
            let index = layout.equality_index_id.unwrap();
            let locator = layout.locator.clone();
            let kind = layout.entity_kind;
            let observe = |value: &str| {
                graph
                    .integration_handle()
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
            let mut selectors = PreparedSelections::new();
            for fact in [&present, &missing, &unrelated] {
                graph
                    .integration_handle()
                    .with_runtime(&borrowed_execution, |runtime| {
                        assert!(fact.remains_equal_in(runtime, execution, snapshot).unwrap());
                        assert!(selectors
                            .remains_equal(execution, fact, runtime, snapshot, control)
                            .unwrap());
                    });
            }
            assert_eq!(
                selectors.selectors.len(),
                1,
                "all distinct predicates share only the exact index admission"
            );
            foreign.application.primary_provider.graph.with_runtime(
                &borrowed_execution,
                |runtime| {
                    assert!(!selectors
                        .remains_equal(execution, &present, runtime, snapshot, control)
                        .unwrap());
                },
            );
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
                .integration_handle()
                .with_runtime(&borrowed_execution, |runtime| {
                    assert!(selectors
                        .remains_equal(execution, &present, runtime, snapshot, control)
                        .unwrap())
                });
            let current = world.selected_product(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
            );
            let current_snapshot = current.application_basis().snapshot_handle();
            let mut current_selectors = PreparedSelections::new();
            graph
                .integration_handle()
                .with_runtime(&borrowed_execution, |runtime| {
                    assert!(!current_selectors
                        .remains_equal(execution, &present, runtime, current_snapshot, control)
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
                        .remains_equal(execution, &complete, runtime, current_snapshot, control)
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
                                .remains_equal(
                                    execution,
                                    &narrowed,
                                    runtime,
                                    current_snapshot,
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
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let source =
        worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new();
    let scope = worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
        None,
        source.token(),
    );
    world.application.with_application_advancement(&scope, |phase| {
        let execution = phase.checked_execution_request_for(world.application.product_runtime()).unwrap();
        let selected = world.selected_product(&phase.test_host_request());
        let snapshot = selected.application_basis().snapshot_handle();
        let graph = world.application.runtime.primary_graph().unwrap();
        let field = graph.layout.equality_field("Account", "AccountPolicy", "AccountStatus").unwrap();
        graph.integration_handle().with_runtime(&execution, |runtime| {
            let fact = observe_indexed_entity_selection(runtime, &execution, snapshot, field.equality_index_id.unwrap(), field.entity_kind, field.locator.clone(), StringApplicationValueBinding::encode(&"open".to_owned()).unwrap(), 2).unwrap();
            let mut prepared = PreparedSelections::new();
            let control = StorageControl::new(ExecutionAllocationPolicy::SystemAllocation, None);
            assert!(prepared.remains_equal(&execution, &fact, runtime, snapshot, control).unwrap());
            let _probe = worth_execution::ExecutionCheckpointObservationForTest::install_on_this_thread_for_test(move || source.cancel());
            let stopped = prepared.remains_equal(&execution, &fact, runtime, snapshot, control).unwrap_err();
            assert_eq!(stopped.kind(), WorthQueryDecisionReadSetDenialKind::RequestInterrupted(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled));
        });
    }).unwrap();
}
