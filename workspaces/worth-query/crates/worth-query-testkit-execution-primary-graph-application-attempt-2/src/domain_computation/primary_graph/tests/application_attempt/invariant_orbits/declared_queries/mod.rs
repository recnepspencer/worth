//! Installed complete orbit answers, with the existing bootstrap and admission path.

use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::KeySlot;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::Next;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::NodeRow;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitModel;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitSchema;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::Parameters;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::builder;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::definition;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::key;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::orbit;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::whole;

use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::Graph;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::World;
use worth_execution::ExecutionRequest;
use worth_query_execution::facade::application_invariants::WorthQueryAggregateIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotResult;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
use worth_query_execution::facade::domain_computation::WorthQueryObservedSource;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::component_selection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::facade;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::root;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::*;
pub fn world() -> World {
    world::install_orbit(
        Graph {
            nodes: 5,
            edges: vec![(0, 2), (2, 1), (1, 0), (3, 4), (4, 3)],
            active: false,
            blocked: vec![],
        },
        crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe::new(|_, _, _| {}),
    )
}
pub fn native_oracle(
    world: &World,
) -> (
    Vec<u64>,
    WorthQueryAggregateIdentity<OrbitSchema<true>, Next>,
) {
    let completed = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let ids = [0_u64, 2, 1].map(|key| {
                            (
                                key,
                                reader.resolve_entity(NodeKey::reference(), key).unwrap(),
                            )
                        });
                        let least = ids
                            .iter()
                            .min_by_key(|(_, member)| member.entity_id())
                            .unwrap()
                            .0;
                        let cycle = [0_u64, 2, 1];
                        let offset = cycle.iter().position(|key| *key == least).unwrap();
                        let expected = cycle[offset..]
                            .iter()
                            .chain(&cycle[..offset])
                            .copied()
                            .collect();
                        let root = reader.resolve_entity(NodeKey::reference(), 1).unwrap();
                        (
                            expected,
                            reader
                                .orbit(Next::reference(), &root)
                                .unwrap()
                                .identity()
                                .clone(),
                        )
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    completed.into_parts().0
}

#[path = "capability/mod.rs"]
pub mod capability;
#[path = "components/mod.rs"]
pub mod components;
#[path = "declarations/mod.rs"]
pub mod declarations;
#[path = "disclosure/mod.rs"]
pub mod disclosure;
#[path = "metadata_only/mod.rs"]
pub mod metadata_only;
#[path = "read/mod.rs"]
pub mod read;
