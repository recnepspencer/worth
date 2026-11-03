//! Genuine merge/split history isolates identity truth from membership and lineage.

use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::Next;

use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::World;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::aggregate_observation::AggregateObservation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
use worth_execution::ExecutionRequest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::aggregate_observation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::root;
use std::sync::Arc;
use worth_execution::ExecutionAllocationPolicy as Allocation;
pub fn install() -> World {
    crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::install_orbit(
        crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::Graph {
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
        crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe::new(|_, _, _| {}),
    )
}
pub struct Read {
    pub body: Arc<AggregateObservation>,
    pub lease: WorthQueryApplicationSnapshotLease,
}
pub fn read(world: &World, key: u64, execution: &ExecutionRequest<'_, '_>) -> Read {
    let selected = world
        .application
        .select_product_branch_for_test(
            execution,
            world.application.product_runtime().default_branch_for_test(),
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
        snapshot.into_lease_and_realized_scope_for_test(selected.product().retained_clone_for_test());
    Read { body, lease }
}
impl Read {
    pub fn still_equal(
        &self,
        old: &AggregateObservation,
        execution: &ExecutionRequest<'_, '_>,
    ) -> bool {
        self.lease.handle_for_test().with_runtime(execution, |runtime| {
            old.remains_equal_for_test(runtime, execution, self.lease.snapshot_for_test(), None)
                .unwrap()
        })
    }
}
