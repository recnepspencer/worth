//! Raw operation walks use actual admission and the existing pinned executor.
use super::{program::authorization::authorize, schema::*, world};
use crate::domain_computation::primary_graph::*;
use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy as Allocation, ExecutionRequest,
    SerialMemoryBudget, SerialRequest,
};
use worth_query_declaration::facade::application_schema::RelationDirection::{Forward, Reverse};

#[test]
fn operation_walks_preserve_pinned_results_and_exact_acquisitions() {
    let world = world::install(world::Graph {
        nodes: 6,
        active: false,
        blocked: vec![],
        edges: vec![(0, 0), (0, 1), (0, 2), (1, 3), (2, 3), (3, 0), (3, 4)],
    });
    let request = super::super::super::fixture::live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let (admission, _) = authorize(
                &world,
                &phase,
                &execution,
                &request,
                0,
                InspectWalk::reference(),
                None,
            );
            assert_eq!(
                admission.allowed_graph_contract().projection_work_budget(),
                None
            );
            let completed = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, root| reader.walk(Next::reference(), Forward, root).unwrap(),
                    Allocation::SystemAllocation,
                )
                .unwrap();
            let work = completed.work();
            assert_eq!(work.endpoint_records_read(), 5);
            assert_eq!(work.adjacency_lists_read(), 5);
            assert_eq!(work.adjacency_edges_inspected(), 7);
            assert_eq!(work.equality_lookups(), 0);
            let walk = completed.output();
            assert_eq!(
                walk.iter().map(|s| s.distance()).collect::<Vec<_>>(),
                [0, 1, 1, 2, 3]
            );
            assert_eq!(walk.relations().len(), 7);
            let actual: Vec<_> = walk.entities().map(|id| id.entity_id()).collect();
            let snapshot = world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let root = reader.resolve_entity(NodeKey::reference(), 0).unwrap();
                        reader.walk(Next::reference(), Forward, &root).unwrap()
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap();
            assert_eq!(
                snapshot
                    .output()
                    .entities()
                    .map(|id| id.entity_id())
                    .collect::<Vec<_>>(),
                actual
            );
            assert_eq!(
                snapshot.work().adjacency_lists_read(),
                work.adjacency_lists_read()
            );
            assert_eq!(
                snapshot.work().adjacency_edges_inspected(),
                work.adjacency_edges_inspected()
            );
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, root| {
                        let leaf = reader.resolve_entity(NodeKey::reference(), 4).unwrap();
                        let detached = reader.resolve_entity(NodeKey::reference(), 5).unwrap();
                        let roots = [root.clone(), leaf.clone(), root.clone()];
                        let closure = reader.closure(Next::reference(), Forward, &roots).unwrap();
                        assert_eq!(
                            closure.iter().map(|s| s.distance()).collect::<Vec<_>>(),
                            [0, 0, 1, 1, 2]
                        );
                        assert_eq!(closure.relations().len(), 7);
                        assert!(reader
                            .closure(Next::reference(), Forward, &[])
                            .unwrap()
                            .is_empty());
                        assert_eq!(
                            reader
                                .walk(Next::reference(), Reverse, &leaf)
                                .unwrap()
                                .len(),
                            5
                        );
                        assert!(reader.reaches(Next::reference(), root, &leaf).unwrap());
                        assert!(reader.reaches(Next::reference(), root, root).unwrap());
                        assert!(!reader.reaches(Next::reference(), root, &detached).unwrap());
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap();
        })
        .unwrap();
}

#[test]
fn operation_walks_keep_foreign_checks_and_original_cancellation() {
    let world = world::install(world::Graph::three_hops(false, vec![]));
    let foreign = world::install(world::Graph::three_hops(false, vec![]));
    let root = foreign
        .application
        .with_host_advancement(|phase| {
            foreign
                .invariant
                .project(
                    &phase,
                    |reader| reader.resolve_entity(NodeKey::reference(), 0).unwrap(),
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .unwrap();
    let request = super::super::super::fixture::live_scope();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let (admission, _) = authorize(
                &world,
                &phase,
                &execution,
                &request,
                0,
                InspectWalk::reference(),
                None,
            );
            let refused = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, local| {
                        assert_eq!(
                            reader
                                .walk(Next::reference(), Forward, &root)
                                .unwrap_err()
                                .kind(),
                            WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity
                        );
                        assert_eq!(
                            reader
                                .closure(Next::reference(), Forward, &[local.clone(), root.clone()])
                                .unwrap_err()
                                .kind(),
                            WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity
                        );
                        assert_eq!(
                            reader
                                .reaches(Next::reference(), local, &root)
                                .unwrap_err()
                                .kind(),
                            WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity
                        );
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap();
            assert_eq!(refused.work().endpoint_records_read(), 0);
            assert_eq!(refused.work().adjacency_lists_read(), 0);
            admission
        })
        .unwrap();
    // The original request is a root extent, not a sibling of the host extent
    // that authenticated and admitted the operation.
    for reaches in [false, true] {
        let cancel = CancellationSource::new();
        let original =
            SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
        let execution = ExecutionRequest::serial(&original);
        let denial = world
            .invariant
            .project_admitted_operation_with_request(
                &execution,
                &admission,
                |reader, local| {
                    cancel.cancel();
                    let error = if reaches {
                        reader.reaches(Next::reference(), local, local).map(|_| ())
                    } else {
                        reader.walk(Next::reference(), Forward, local).map(|_| ())
                    }
                    .unwrap_err();
                    assert!(matches!(
                        error.kind(),
                        WorthQueryInvariantProjectionTraversalDenialKind::ReadExecution(_)
                    ));
                    // Swallowing the original stop must still veto completion.
                },
                Allocation::SystemAllocation,
            )
            .err()
            .expect("cancelled raw read cannot mint a completed projection");
        assert_eq!(denial.kind(), WorthQueryOperationProjectionDenialKind::InvariantAdmission(
                WorthQueryInvariantProjectionDenialKind::ExecutionRequest(
                    worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled)));
        assert_eq!(denial.projection_work().unwrap().endpoint_records_read(), 0);
        assert_eq!(denial.projection_work().unwrap().adjacency_lists_read(), 0);
    }
}

#[test]
fn operation_walks_cannot_swallow_the_installed_limit() {
    let world = world::install(world::Graph {
        nodes: 32,
        active: false,
        blocked: vec![],
        edges: (0..31).map(|id| (id, id + 1)).collect(),
    });
    let request = super::super::super::fixture::live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let (limited, _) = authorize(
                &world,
                &phase,
                &execution,
                &request,
                0,
                ReplaceNext::reference(),
                None,
            );
            let denial = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &limited,
                    |reader, root| {
                        assert_eq!(
                            reader
                                .walk(Next::reference(), Forward, root)
                                .unwrap_err()
                                .kind(),
                            WorthQueryInvariantProjectionTraversalDenialKind::WorkBudgetExceeded
                        );
                    },
                    Allocation::SystemAllocation,
                )
                .err()
                .expect("partial traversal cannot seal");
            assert_eq!(
                denial.kind(),
                WorthQueryOperationProjectionDenialKind::WorkBudgetExceeded
            );
            let partial = denial.projection_work().unwrap().endpoint_records_read();
            assert!(
                partial > 0 && partial < 32,
                "the actual traversal stopped partway"
            );
            let (uncapped, _) = authorize(
                &world,
                &phase,
                &execution,
                &request,
                0,
                InspectWalk::reference(),
                None,
            );
            let complete = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &uncapped,
                    |reader, root| reader.walk(Next::reference(), Forward, root).unwrap(),
                    Allocation::SystemAllocation,
                )
                .unwrap();
            assert_eq!(complete.output().len(), 32);
            assert_eq!(complete.output().relations().len(), 31);
            assert_eq!(complete.work().endpoint_records_read(), 32);
            assert_eq!(complete.work().adjacency_lists_read(), 32);
        })
        .unwrap();
}
