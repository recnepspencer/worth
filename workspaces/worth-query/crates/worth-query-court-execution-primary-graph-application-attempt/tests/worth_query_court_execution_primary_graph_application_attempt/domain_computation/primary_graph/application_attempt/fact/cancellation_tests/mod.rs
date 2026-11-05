//! Cancellation is a typed stop through native adjacency and indexed equality.

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::adjacency::AdjacencyObservationDenial;
pub use worth_relational::facade::identity::KindId;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::fact::WorthQueryApplicationAdjacencyDirection;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::indexed_entity_selection::observe_indexed_entity_selection;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountOwner;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind as Kind;
pub use worth_execution::CancellationSource;
pub use worth_execution::ExecutionRequest;
pub use worth_execution::SerialMemoryBudget;
pub use worth_execution::SerialRequest;
pub use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
pub use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;

#[test]
fn cancelled_indexed_selection_equality_keeps_the_native_stop() {
    with_indexed_fact(|runtime, execution, snapshot, fact, source, _| {
        let WorthQueryApplicationObservedFact::IndexedEntitySelection {
            index_id,
            definition,
            entity_kind,
            locator,
            value,
            candidate_limit,
            candidates,
        } = fact
        else {
            unreachable!()
        };
        let compare = || {
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::indexed_entity_selection::remains_equal(
                runtime,
                execution,
                snapshot,
                *index_id,
                definition,
                *entity_kind,
                locator,
                value,
                *candidate_limit,
                candidates,
            )
        };
        assert!(compare().unwrap());
        source.cancel();
        assert_eq!(compare().unwrap_err().kind(), Kind::Cancelled);
        assert_eq!(
            fact.remains_equal_in_for_test(runtime, execution, snapshot)
                .unwrap_err()
                .kind(),
            Kind::Cancelled
        );
    });
}

#[test]
fn cancelled_adjacency_read_and_equality_keep_the_native_stop() {
    with_indexed_fact(
        |runtime, execution, snapshot, fact, source, relation_kind| {
            let WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. } = fact
            else {
                unreachable!()
            };
            let anchor = candidates[0];
            let direction = WorthQueryApplicationAdjacencyDirection::Incoming;
            let expected = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::adjacency::observe_adjacency(
                runtime,
                execution,
                snapshot,
                relation_kind,
                anchor,
                direction,
                32,
            )
            .unwrap()
            .unwrap();
            assert!(
                !expected.is_empty(),
                "the fixture account has a real owner relation"
            );
            assert!(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::adjacency::remains_equal(
                runtime,
                execution,
                snapshot,
                relation_kind,
                anchor,
                direction,
                32,
                &expected
            )
            .unwrap());
            source.cancel();
            let stopped = [
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::adjacency::observe_adjacency_with_work(
                    runtime,
                    execution,
                    snapshot,
                    relation_kind,
                    anchor,
                    direction,
                    Some(32),
                )
                .unwrap_err(),
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::adjacency::observe_adjacency_checked(
                    runtime,
                    execution,
                    snapshot,
                    relation_kind,
                    anchor,
                    direction,
                    32,
                )
                .unwrap_err(),
            ];
            for denial in stopped {
                let AdjacencyObservationDenial::Interrupted(event) = denial else {
                    panic!("a cancelled read is its interruption");
                };
                assert_eq!(
                    event.interruption(),
                    worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled
                );
                assert_eq!(event.boundary(), worth_relational::facade::mvcc::RelationalInterruptionBoundary::ObservationAdmission);
            }
            assert_eq!(
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::adjacency::observe_adjacency(
                    runtime,
                    execution,
                    snapshot,
                    relation_kind,
                    anchor,
                    direction,
                    32
                )
                .unwrap_err()
                .kind(),
                Kind::Cancelled
            );
            assert_eq!(
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::adjacency::remains_equal(
                    runtime,
                    execution,
                    snapshot,
                    relation_kind,
                    anchor,
                    direction,
                    32,
                    &expected
                )
                .unwrap_err()
                .kind(),
                Kind::Cancelled
            );
            let fact = WorthQueryApplicationObservedFact::Adjacency {
                relation_kind,
                anchor,
                direction,
                maximum_work_units: 32,
                relations: expected,
            };
            assert_eq!(
                fact.remains_equal_in_for_test(runtime, execution, snapshot)
                    .unwrap_err()
                    .kind(),
                Kind::Cancelled
            );
        },
    );
}

pub fn with_indexed_fact<R>(
    body: impl FnOnce(
        &worth_relational::facade::runtime::RelationalRuntime,
        &ExecutionRequest<'_, '_>,
        &worth_relational::facade::snapshots::SnapshotHandle,
        &WorthQueryApplicationObservedFact,
        &CancellationSource,
        KindId,
    ) -> R,
) -> R {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request_for_test()))
        .unwrap();
    let snapshot = selected.application_basis_for_test().snapshot_handle();
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let field = AccountStatus::reference();
    let layout = (*graph.layout_for_test())
        .equality_field_for_test(field.entity(), field.aspect(), field.field())
        .unwrap();
    let relation_kind = (*(*graph.layout_for_test())
        .relation(AccountOwner::reference().name())
        .unwrap()
        .kind_for_test());
    let source = CancellationSource::new();
    let serial = SerialRequest::from_memory(SerialMemoryBudget::new(None), source.token(), None);
    let execution = ExecutionRequest::serial(&serial);
    let outcome = graph
        .integration_handle_for_test()
        .with_runtime(&execution, |runtime| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let fact = observe_indexed_entity_selection(
                    runtime,
                    &execution,
                    snapshot,
                    (*layout.equality_index_id_for_test()).unwrap(),
                    (*layout.entity_kind_for_test()),
                    (*layout.locator_for_test()).clone(),
                    StringApplicationValueBinding::encode(&"open".to_owned()).unwrap(),
                    2,
                )
                .unwrap();
                body(runtime, &execution, snapshot, &fact, &source, relation_kind)
            }))
        });
    match outcome {
        Ok(value) => value,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

#[test]
fn source_currentness_keeps_a_stop_raised_inside_the_indexed_read() {
    with_indexed_fact(|runtime, execution, snapshot, fact, source, _| {
        let observation =
            worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
        let cancel = source.clone();
        let mut consults = 0;
        let _probe =
            worth_execution::ExecutionCheckpointObservationForTest::install_on_this_thread_for_test(
                move || {
                    consults += 1;
                    if consults == 3 {
                        cancel.cancel();
                    }
                },
            );

        worth_query_execution::facade::domain_computation::fact_verification_on_this_thread_for_test();
        assert_eq!(fact.source_currentness_in_for_test(runtime, execution, snapshot), Err(worth_relational::facade::mvcc::CompanionPreflightStop::Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled)));
        let observed = worth_query_execution::facade::domain_computation::fact_verification_on_this_thread_for_test();
        assert!(
            observed.source_currentness_visits > 0,
            "the interrupted native comparison remains observed"
        );

        assert_eq!(worth_query_execution::facade::domain_computation::fact_verification_on_this_thread_for_test(), Default::default(), "taking the interval cannot recount it");
        // Leave a genuine stopped comparison unconsumed. The final listener
        // owns that diagnostic interval and must release it on drop.
        assert!(fact
            .source_currentness_in_for_test(runtime, execution, snapshot)
            .is_err());
        drop(observation);
        assert_eq!(worth_query_execution::facade::domain_computation::fact_verification_on_this_thread_for_test(), Default::default(), "dropping the observation clears untaken work");
    });
}

#[path = "counted/mod.rs"]
pub mod counted;
#[path = "exhaustion/mod.rs"]
pub mod exhaustion;
