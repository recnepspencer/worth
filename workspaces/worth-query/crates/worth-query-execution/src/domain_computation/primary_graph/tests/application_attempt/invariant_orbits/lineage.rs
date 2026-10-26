//! Real Query commits and selected-product readers expose Native historical truth.
use super::{probe, program, schema::*, world, World};
use crate::domain_computation::primary_graph::*;
use crate::facade::primary_graph::{AggregateLineageKind, CommitId};
use worth_execution::{ExecutionAllocationPolicy as Allocation, ExecutionRequest};
use worth_relational::facade::identity::EntityId;
mod guards;

type Schema = OrbitSchema<true>;
type Name = WorthQueryAggregateIdentity<Schema, Next>;
type History = WorthQueryAggregateLineage<Schema, Next>;
type Denial = WorthQueryInvariantProjectionTraversalDenial;

fn install(edges: &[(u64, u64)]) -> World {
    world::install_orbit(
        world::Graph {
            nodes: edges.len(),
            edges: edges.to_vec(),
            active: false,
            blocked: vec![],
        },
        probe::Probe::new(|_, _, _| {}),
    )
}

fn selected(world: &World, execution: &ExecutionRequest<'_, '_>) -> WorthQueryProductBranchLease {
    world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap()
        .product()
        .retained_clone()
}

fn name(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    product: &WorthQueryProductBranchLease,
    key: u64,
) -> (EntityId, Name) {
    world
        .invariant
        .project_operation_on_product::<ReplaceNext, _>(
            execution,
            product,
            |reader| {
                let root = reader.resolve_entity(NodeKey::reference(), key).unwrap();
                let orbit = reader.orbit(Next::reference(), &root).unwrap();
                (
                    orbit.member(0).unwrap().entity_id(),
                    orbit.identity().clone(),
                )
            },
            Allocation::SystemAllocation,
        )
        .unwrap()
        .into_output()
}

fn inspect(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    product: &WorthQueryProductBranchLease,
    identity: &Name,
) -> Result<History, Denial> {
    world
        .invariant
        .project_operation_on_product::<ReplaceNext, _>(
            execution,
            product,
            |reader| reader.lineage(Next::reference(), identity),
            Allocation::SystemAllocation,
        )
        .unwrap()
        .into_output()
}

fn events(history: &History) -> Vec<(CommitId, AggregateLineageKind, Name, Name)> {
    history
        .events()
        .map(|event| (event.occurrence(), event.kind(), event.from(), event.to()))
        .collect()
}

#[test]
fn retired_name_follows_two_real_merges_while_old_bases_and_retained_bodies_stay_fixed() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let world = install(&[(0, 1), (1, 0), (2, 3), (3, 2), (4, 5), (5, 4)]);
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let original = selected(&world, &execution);
    let mut groups = [0, 2, 4].map(|seed| {
        let (anchor, identity) = name(&world, &execution, &original, seed);
        (anchor, seed, identity)
    });
    groups.sort_by_key(|(anchor, _, _)| *anchor);
    let [(_, a_seed, a), (_, b_seed, b), (_, c_seed, c)] = groups;
    let before = inspect(&world, &execution, &original, &c).unwrap();
    assert_eq!(before.identity(), &c);
    assert_eq!(before.latest_identity(), Some(c.clone()));
    assert_eq!(before.retired_at(), None);
    assert_eq!(before.events().len(), 0);

    let request = super::super::super::fixture::live_scope();
    let swap = program::swap_successors(&world, &request, b_seed, c_seed);
    assert!(matches!(
        super::commit(&world, &request, swap, 251),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let first = selected(&world, &execution);
    let middle = inspect(&world, &execution, &first, &c).unwrap();
    assert_eq!(middle.latest_identity(), Some(b.clone()));
    let retired = middle
        .retired_at()
        .expect("first actual merge retired the loser");
    assert_eq!(
        events(&middle),
        vec![(
            retired,
            AggregateLineageKind::MergedInto,
            c.clone(),
            b.clone()
        )]
    );

    let swap = program::swap_successors(&world, &request, a_seed, b_seed);
    assert!(matches!(
        super::commit(&world, &request, swap, 252),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let second = selected(&world, &execution);
    let after = inspect(&world, &execution, &second, &c).unwrap();
    assert_eq!(after.latest_identity(), Some(a.clone()));
    assert_eq!(after.retired_at(), Some(retired));
    assert_eq!(after.version(), world.committed_version());
    assert_ne!(after.version(), before.version());
    let steps = after
        .resolution_steps()
        .map(|event| {
            assert_eq!(event.kind(), AggregateLineageKind::MergedInto);
            (event.occurrence(), event.from(), event.to())
        })
        .collect::<Vec<_>>();
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0], (retired, c.clone(), b.clone()));
    assert_eq!((&steps[1].1, &steps[1].2), (&b, &a));
    assert_ne!(steps[0].0, steps[1].0);
    assert_eq!(
        events(&after),
        events(&middle),
        "incident history is not the transitive resolution path"
    );
    assert_eq!(middle.latest_identity(), Some(b));
    assert_eq!(before.latest_identity(), Some(c.clone()));
    assert_eq!(
        inspect(&world, &execution, &original, &c)
            .unwrap()
            .latest_identity(),
        Some(c)
    );
}

#[test]
fn split_history_keeps_the_current_name_and_refuses_a_future_name_at_an_older_basis() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let world = install(&[(0, 1), (1, 2), (2, 3), (3, 0)]);
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let original = selected(&world, &execution);
    let (_, old) = name(&world, &execution, &original, 0);
    let request = super::super::super::fixture::live_scope();
    let swap = program::swap_successors(&world, &request, 0, 2);
    assert!(matches!(
        super::commit(&world, &request, swap, 253),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let split = selected(&world, &execution);
    let (_, left) = name(&world, &execution, &split, 0);
    let (_, right) = name(&world, &execution, &split, 1);
    assert_ne!(left, right);
    let born = if left == old {
        right
    } else {
        assert_eq!(right, old);
        left
    };
    let keeper = inspect(&world, &execution, &split, &old).unwrap();
    assert_eq!(keeper.latest_identity(), Some(old.clone()));
    assert_eq!(keeper.retired_at(), None);
    assert_eq!(
        keeper.resolution_steps().len(),
        0,
        "split is not a merge continuation"
    );
    let incident = events(&keeper);
    assert_eq!(incident.len(), 1);
    assert_eq!(incident[0].1, AggregateLineageKind::SplitFrom);
    assert_eq!((&incident[0].2, &incident[0].3), (&born, &old));
    let child = inspect(&world, &execution, &split, &born).unwrap();
    assert_eq!(child.latest_identity(), Some(born.clone()));
    assert_eq!(events(&child), incident);
    assert_eq!(
        inspect(&world, &execution, &original, &old)
            .unwrap()
            .events()
            .len(),
        0
    );
    assert_eq!(
        inspect(&world, &execution, &original, &born)
            .unwrap_err()
            .kind(),
        WorthQueryInvariantProjectionTraversalDenialKind::AggregateIdentityUnavailable
    );
}
