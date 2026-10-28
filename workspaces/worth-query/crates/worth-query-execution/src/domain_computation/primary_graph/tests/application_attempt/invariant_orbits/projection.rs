//! Ordinary projection reads retain actual naming, order and complete business scope.
use super::{probe, program, schema::*, world, World};
use crate::domain_computation::primary_graph::*;
use worth_execution::{ExecutionAllocationPolicy as Allocation, ExecutionRequest};
mod guards;

fn graph() -> world::Graph {
    let cycle = [0, 2, 4, 1, 3, 5];
    world::Graph {
        nodes: 8,
        active: false,
        blocked: vec![],
        edges: cycle
            .iter()
            .copied()
            .zip(cycle.into_iter().cycle().skip(1))
            .chain([(6, 7), (7, 6)])
            .collect(),
    }
}

#[test]
fn projection_orbits_preserve_successor_order_name_and_all_member_scope() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let world = world::install_orbit(graph(), probe::Probe::new(|_, _, _| {}));
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let selected = world
        .application
        .select_product_branch(
            &execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap();
    let completed = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let root = reader.resolve_entity(NodeKey::reference(), 3_u64).unwrap();
                        let before = reader.work();
                        let orbit = reader.orbit(Next::reference(), &root).unwrap();
                        let after = reader.work();
                        assert_eq!(
                            after.endpoint_records_read() - before.endpoint_records_read(),
                            6
                        );
                        assert_eq!(
                            after.adjacency_lists_read() - before.adjacency_lists_read(),
                            6
                        );
                        assert_eq!(
                            after.adjacency_edges_inspected() - before.adjacency_edges_inspected(),
                            6
                        );
                        let ids = orbit.members().map(|m| m.entity_id()).collect::<Vec<_>>();
                        assert_eq!(ids.len(), 6);
                        assert_eq!(ids[0], *ids.iter().min().unwrap());
                        assert_eq!(orbit.relations().len(), 6);
                        assert_eq!(orbit.member(5).unwrap().entity_id(), ids[5]);
                        assert!(orbit.member(6).is_none());
                        assert!(!orbit.is_empty());
                        assert_eq!(reader.work(), after, "iteration acquires no graph metadata");
                        assert_eq!(
                            reader.orbit_of(Next::reference(), &root).unwrap(),
                            *orbit.identity()
                        );
                        orbit
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let (orbit, snapshot, _) = completed.into_parts();
    let keys = orbit
        .members()
        .map(|member| {
            snapshot
                .field(&execution, &member, NodeKey::reference())
                .unwrap()
                .unwrap()
        })
        .collect::<Vec<u64>>();
    let cycle = [0, 2, 4, 1, 3, 5];
    let first = cycle.iter().position(|key| *key == keys[0]).unwrap();
    assert_eq!(
        keys,
        cycle[first..]
            .iter()
            .chain(&cycle[..first])
            .copied()
            .collect::<Vec<_>>()
    );
    let (_lease, scope, _, _) =
        snapshot.into_lease_and_realized_scope(selected.product().retained_clone());
    for member in orbit.members() {
        assert!(
            scope.contains(member.entity_id()),
            "every returned member has actual scope proof"
        );
    }
    let inspected = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_operation::<ReplaceNext, _>(
                    &phase,
                    |reader| {
                        let root = reader.resolve_entity(NodeKey::reference(), 1_u64).unwrap();
                        let same = reader.orbit(Next::reference(), &root).unwrap();
                        assert_eq!(same.identity(), orbit.identity());
                        assert_eq!(
                            reader.orbit_of(Next::reference(), &root).unwrap(),
                            *same.identity()
                        );
                        same.members()
                            .map(|member| reader.field(&member, NodeKey::reference()).unwrap())
                            .collect::<Vec<_>>()
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    assert_eq!(inspected.into_output(), keys);
}

fn inspect(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    product: &crate::basis::WorthQueryProductBranchLease,
    key: u64,
) -> (
    WorthQueryAggregateIdentity<OrbitSchema<true>, Next>,
    Vec<u64>,
) {
    world
        .invariant
        .project_operation_on_product::<ReplaceNext, _>(
            execution,
            product,
            |reader| {
                let root = reader.resolve_entity(NodeKey::reference(), key).unwrap();
                let orbit = reader.orbit(Next::reference(), &root).unwrap();
                let keys = orbit
                    .members()
                    .map(|member| reader.field(&member, NodeKey::reference()).unwrap())
                    .collect();
                (orbit.identity().clone(), keys)
            },
            Allocation::SystemAllocation,
        )
        .unwrap()
        .into_output()
}

#[test]
fn retained_selected_basis_keeps_its_orbit_after_an_actual_split() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let world = world::install_orbit(graph(), probe::Probe::new(|_, _, _| {}));
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let original = world
        .application
        .select_product_branch(
            &execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap();
    let before = inspect(&world, &execution, original.product(), 0);
    assert_eq!(before.1.len(), 6);
    let request = super::super::super::fixture::live_scope();
    let swap = program::swap_successors(&world, &request, 0, 1);
    assert!(matches!(
        super::commit(&world, &request, swap, 230),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let after = world
        .application
        .select_product_branch(
            &execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap();
    let left = inspect(&world, &execution, after.product(), 0);
    let right = inspect(&world, &execution, after.product(), 1);
    assert_eq!(left.1.len(), 3);
    assert_eq!(right.1.len(), 3);
    assert_ne!(left.0, right.0);
    assert_eq!(inspect(&world, &execution, original.product(), 0), before);
    let keeper = if [0, 3, 5].contains(&before.1[0]) {
        &left.0
    } else {
        &right.0
    };
    assert_eq!(
        keeper, &before.0,
        "the part with the least surviving old member keeps the name"
    );
    let mut left_keys = left.1;
    let mut right_keys = right.1;
    left_keys.sort();
    right_keys.sort();
    assert_eq!(left_keys, vec![0, 3, 5]);
    assert_eq!(right_keys, vec![1, 2, 4]);
}
