//! Genuine merge/split history isolates identity truth from membership and lineage.
use super::super::{probe, program, schema::*, world, World};
use crate::domain_computation::primary_graph::{
    aggregate_observation::AggregateObservation,
    application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease, *,
};
use std::sync::Arc;
use worth_execution::{ExecutionAllocationPolicy as Allocation, ExecutionRequest};

pub(in crate::domain_computation::primary_graph) fn install() -> World {
    world::install_orbit(
        world::Graph {
            nodes: 8,
            active: false,
            blocked: vec![],
            edges: vec![
                (0, 1),
                (1, 2),
                (2, 0),
                (3, 4),
                (4, 5),
                (5, 3),
                (6, 7),
                (7, 6),
            ],
        },
        probe::Probe::new(|_, _, _| {}),
    )
}
pub(super) struct Read {
    pub(super) body: Arc<AggregateObservation>,
    lease: WorthQueryApplicationSnapshotLease,
}
pub(super) fn read(world: &World, key: u64, execution: &ExecutionRequest<'_, '_>) -> Read {
    let selected = world
        .application
        .select_product_branch(
            execution,
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
                        let root = reader.resolve_entity(NodeKey::reference(), key).unwrap();
                        reader
                            .orbit(Next::reference(), &root)
                            .unwrap()
                            .aggregate()
                            .clone()
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let (body, snapshot, _) = completed.into_parts();
    let (lease, _, _, _) =
        snapshot.into_lease_and_realized_scope(selected.product().retained_clone());
    Read { body, lease }
}
impl Read {
    pub(super) fn still_equal(
        &self,
        old: &AggregateObservation,
        execution: &ExecutionRequest<'_, '_>,
    ) -> bool {
        self.lease.handle().with_runtime(execution, |runtime| {
            old.remains_equal(runtime, execution, self.lease.snapshot(), None)
                .unwrap()
        })
    }
}
#[test]
fn aggregate_identity_changes_are_distinct_from_membership_and_incidental_history() {
    let _isolation = super::super::super::super::fixture::isolated_request_owner();
    let world = install();
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let originals = [read(&world, 0, &execution), read(&world, 3, &execution)];
    let winner =
        usize::from(originals[0].body.native().anchor() > originals[1].body.native().anchor());
    let request = super::super::super::super::fixture::live_scope();
    for key in [231, 232] {
        let change = program::swap_successors(&world, &request, 0, 3);
        assert!(matches!(
            super::super::commit(&world, &request, change, key),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
    }
    for (index, key) in [0, 3].into_iter().enumerate() {
        let current = read(&world, key, &execution);
        let old = &originals[index].body;
        assert_eq!(
            old.native().members().collect::<Vec<_>>(),
            current.body.native().members().collect::<Vec<_>>()
        );
        assert_eq!(
            old.native().definition(),
            current.body.native().definition()
        );
        assert!(old.native().lineage().next().is_none());
        assert!(
            current.body.native().lineage().next().is_some(),
            "real commits changed stored history"
        );
        let same = index == winner;
        assert_eq!(
            old.native().identity() == current.body.native().identity(),
            same
        );
        assert_eq!(
            old.same_truth_with_poll(&current.body, || Ok::<_, ()>(()))
                .unwrap(),
            same
        );
        assert_eq!(old == &current.body, same);
        assert_eq!(
            current.still_equal(old, &execution),
            same,
            "only aggregate truth is compared; no walk can mask missing ID comparison"
        );
    }
}
