//! Real installed snapshot callbacks use the pinned Native complete walk.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::{
    schema::*, world,
};
use crate::facade::primary_graph::*;
use worth_execution::{
    ExecutionAllocationPolicy as Allocation, ExecutionRequest, ExecutionWorkCeiling,
};
use worth_query_declaration::facade::application_schema::RelationDirection;

#[test]
fn snapshot_walks_mint_local_identities_without_rereading_metadata() {
    let world = world::install(world::Graph {
        nodes: 6,
        active: false,
        blocked: vec![],
        edges: vec![(0, 0), (0, 1), (0, 2), (1, 3), (2, 3), (3, 0), (3, 4)],
    });
    let completed = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let root = reader.resolve_entity(NodeKey::reference(), 0_u64).unwrap();
                        let before = reader.work();
                        let walk = reader
                            .walk(Next::reference(), RelationDirection::Forward, &root)
                            .unwrap();
                        assert_eq!(
                            reader.work().endpoint_records_read() - before.endpoint_records_read(),
                            5
                        );
                        assert_eq!(
                            reader.work().adjacency_lists_read() - before.adjacency_lists_read(),
                            5
                        );
                        assert_eq!(
                            reader.work().adjacency_edges_inspected()
                                - before.adjacency_edges_inspected(),
                            7
                        );
                        let after = reader.work();
                        assert_eq!(
                            walk.iter().map(|step| step.distance()).collect::<Vec<_>>(),
                            vec![0, 1, 1, 2, 3]
                        );
                        assert_eq!(walk.relations().len(), 7);
                        assert_eq!(
                            reader.work(),
                            after,
                            "typed iteration performs no graph reads"
                        );
                        let last = walk.entities().last().unwrap();
                        let reverse = reader
                            .walk(Next::reference(), RelationDirection::Reverse, &last)
                            .unwrap();
                        assert_eq!(reverse.len(), 5);
                        assert!(reader.reaches(Next::reference(), &root, &last).unwrap());
                        assert!(reader.reaches(Next::reference(), &root, &root).unwrap());
                        let empty = reader
                            .closure(Next::reference(), RelationDirection::Forward, &[])
                            .unwrap();
                        assert!(empty.is_empty());
                        let closure = reader
                            .closure(
                                Next::reference(),
                                RelationDirection::Forward,
                                &[last, root.clone(), root],
                            )
                            .unwrap();
                        assert_eq!(closure.iter().filter(|s| s.distance() == 0).count(), 2);
                        walk
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let (walk, snapshot, _) = completed.into_parts();
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let keys: Vec<_> = walk
        .entities()
        .map(|id| {
            snapshot
                .field(&execution, &id, NodeKey::reference())
                .unwrap()
                .unwrap()
        })
        .collect();
    assert_eq!(keys, vec![0, 1, 2, 3, 4]);
}

fn basis(
    world: &world::World,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_relational::facade::branch::AdmittedRelationalBranchBasis {
    world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap()
        .product()
        .publication_binding()
        .observation()
        .basis()
        .relational_basis()
        .clone()
}

#[test]
fn snapshot_walk_refuses_foreign_roots_and_a_swallowed_partial_stop() {
    let world = world::install(world::Graph::three_hops(false, vec![]));
    let foreign = world::install(world::Graph::three_hops(false, vec![]));
    let root = foreign
        .application
        .with_host_advancement(|phase| {
            foreign
                .invariant
                .project(
                    &phase,
                    |r| r.resolve_entity(NodeKey::reference(), 0_u64).unwrap(),
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let before = reader.work();
                        let refusal = reader
                            .walk(Next::reference(), RelationDirection::Forward, &root)
                            .unwrap_err();
                        assert_eq!(
                            refusal.kind(),
                            WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity
                        );
                        assert_eq!(reader.work(), before);
                        let local = reader.resolve_entity(NodeKey::reference(), 0_u64).unwrap();
                        assert_eq!(
                            reader
                                .walk(Next::reference(), RelationDirection::Forward, &local)
                                .unwrap()
                                .len(),
                            4
                        );
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let selected = basis(&world, &execution);
    let denial = world
        .invariant
        .project_with_work_limit(
            &execution,
            Some(80),
            selected,
            |reader| {
                let local = reader.resolve_entity(NodeKey::reference(), 0_u64).unwrap();
                let error = reader
                    .walk(Next::reference(), RelationDirection::Forward, &local)
                    .unwrap_err();
                assert_eq!(
                    error.kind(),
                    WorthQueryInvariantProjectionTraversalDenialKind::WorkBudgetExceeded
                );
                // Intentionally swallow the read error; completion must still refuse.
            },
            None,
            Allocation::SystemAllocation,
        )
        .err()
        .expect("partial walk cannot seal output");
    assert!(denial.projection_work().unwrap().endpoint_records_read() > 0);

    for reachability in [false, true] {
        use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
        let cancellation = CancellationSource::new();
        let host =
            SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
        let execution = ExecutionRequest::serial(&host);
        let selected = basis(&world, &execution);
        let before = std::cell::Cell::new(None);
        let denial = world
            .invariant
            .project_with_work_limit(
                &execution,
                None,
                selected,
                |reader| {
                    let local = reader.resolve_entity(NodeKey::reference(), 0_u64).unwrap();
                    before.set(Some(reader.work()));
                    cancellation.cancel();
                    let refusal = if reachability {
                        reader
                            .reaches(Next::reference(), &local, &local)
                            .map(|_| ())
                    } else {
                        reader
                            .walk(Next::reference(), RelationDirection::Forward, &local)
                            .map(|_| ())
                    }
                    .unwrap_err();
                    assert!(matches!(
                        refusal.kind(),
                        WorthQueryInvariantProjectionTraversalDenialKind::ReadExecution(_)
                    ));
                    // Swallow the original stop too; no result may seal from this callback.
                },
                None,
                Allocation::SystemAllocation,
            )
            .err()
            .expect("original cancellation vetoes output");
        assert_eq!(
            denial.kind(),
            WorthQueryInvariantProjectionDenialKind::ExecutionRequest(
                worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled
            )
        );
        assert_eq!(
            denial.projection_work(),
            before.get(),
            "no graph read ran after cancellation"
        );
    }
}

#[test]
fn snapshot_walk_keeps_scope_payment_after_result_drop_and_spends_cumulative_serial_work() {
    let world = world::install(world::Graph::three_hops(false, vec![]));
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let selected = basis(&world, &execution);
    let memory = host.memory().unwrap();
    let before = memory.charged_memory_bytes();
    let completed = world
        .invariant
        .project_with_work_limit(
            &execution,
            None,
            selected.clone(),
            |reader| {
                let root = reader.resolve_entity(NodeKey::reference(), 0_u64).unwrap();
                reader
                    .walk(Next::reference(), RelationDirection::Forward, &root)
                    .unwrap()
            },
            None,
            Allocation::SystemAllocation,
        )
        .unwrap();
    let (walk, snapshot, _) = completed.into_parts();
    let with_both = memory.charged_memory_bytes();
    drop(walk);
    let with_scope = memory.charged_memory_bytes();
    assert!(with_scope < with_both, "result releases its own payer");
    assert!(with_scope > before, "snapshot still funds reached entities");
    drop(snapshot);
    assert_eq!(memory.charged_memory_bytes(), before);

    let run = |count, ceiling| {
        execution.run(ExecutionWorkCeiling::new(ceiling), |_| {
            world.invariant.project_with_work_limit(
                &execution,
                None,
                selected.clone(),
                |reader| {
                    let root = reader.resolve_entity(NodeKey::reference(), 0_u64).unwrap();
                    let roots = vec![root; count];
                    reader
                        .closure(Next::reference(), RelationDirection::Forward, &roots)
                        .map(|w| w.len())
                },
                None,
                Allocation::SystemAllocation,
            )
        })
    };
    assert_eq!(*run(2000, Some(11000)).unwrap().0.unwrap().output(), Ok(4));
    let refused = run(4096, Some(11000))
        .unwrap()
        .0
        .err()
        .expect("lowering and Native traversal share one remaining ceiling");
    assert_eq!(
        refused.kind(),
        WorthQueryInvariantProjectionDenialKind::ExecutionRequest(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted
        )
    );
    assert_eq!(*run(4096, None).unwrap().0.unwrap().output(), Ok(4));
}
