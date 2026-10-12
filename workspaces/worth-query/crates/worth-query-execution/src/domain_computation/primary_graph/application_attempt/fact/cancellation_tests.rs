//! Cancellation is a typed stop through native adjacency and indexed equality.
use super::*;
use crate::domain_computation::primary_graph::{
    tests::fixture::{installed_authorization_world, AccountOwner, AccountStatus},
    WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};

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
            indexed_entity_selection::remains_equal(
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
            fact.remains_equal_in(runtime, execution, snapshot)
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
            let expected = adjacency::observe_adjacency(
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
            assert!(adjacency::remains_equal(
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
                adjacency::observe_adjacency_with_work(
                    runtime,
                    execution,
                    snapshot,
                    relation_kind,
                    anchor,
                    direction,
                    32,
                )
                .unwrap_err(),
                adjacency::observe_adjacency_checked(
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
                adjacency::observe_adjacency(
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
                adjacency::remains_equal(
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
                fact.remains_equal_in(runtime, execution, snapshot)
                    .unwrap_err()
                    .kind(),
                Kind::Cancelled
            );
        },
    );
}

fn with_indexed_fact(
    body: impl FnOnce(
        &worth_relational::facade::runtime::RelationalRuntime,
        &ExecutionRequest<'_, '_>,
        &worth_relational::facade::snapshots::SnapshotHandle,
        &WorthQueryApplicationObservedFact,
        &CancellationSource,
        KindId,
    ),
) {
    let world = installed_authorization_world(true);
    world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            let selected = world.selected_product(&custody);
            let snapshot = selected.application_basis().snapshot_handle();
            let graph = world.application.runtime.primary_graph().unwrap();
            let field = AccountStatus::reference();
            let layout = graph
                .layout
                .equality_field(field.entity(), field.aspect(), field.field())
                .unwrap();
            let relation_kind = graph
                .layout
                .relation(AccountOwner::reference().name())
                .unwrap()
                .kind;
            let source = CancellationSource::new();
            let serial =
                SerialRequest::from_memory(SerialMemoryBudget::new(64 << 20), source.token(), None);
            let execution = ExecutionRequest::serial(&serial);
            graph
                .integration_handle()
                .with_runtime(&execution, |runtime| {
                    let fact = observe_indexed_entity_selection(
                        runtime,
                        &execution,
                        snapshot,
                        layout.equality_index_id.unwrap(),
                        layout.entity_kind,
                        layout.locator.clone(),
                        StringApplicationValueBinding::encode(&"open".to_owned()).unwrap(),
                        2,
                    )
                    .unwrap();
                    body(runtime, &execution, snapshot, &fact, &source, relation_kind);
                });
        })
        .unwrap();
}

#[test]
fn source_currentness_keeps_a_stop_raised_inside_the_indexed_read() {
    with_indexed_fact(|runtime, execution, snapshot, fact, source, _| {
        let _observation =
            crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
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
        let mut admission =
            crate::domain_computation::primary_graph::InvalidationEditAdmission::new(
                worth_relational::facade::mvcc::CompanionPreflightBudget {
                    maximum_work_visits: 1_000_000,
                    maximum_preparation_bytes: 64 << 20,
                },
            );
        assert_eq!(fact.source_currentness_in(runtime, execution, snapshot, &mut admission), Err(worth_relational::facade::mvcc::CompanionPreflightStop::Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled)));
    });
}
