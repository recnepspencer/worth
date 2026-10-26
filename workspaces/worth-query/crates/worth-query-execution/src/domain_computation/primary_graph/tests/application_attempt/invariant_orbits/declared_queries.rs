//! Installed complete orbit answers, with the existing bootstrap and admission path.
pub(super) mod capability;
mod currentness;
pub(in crate::domain_computation::primary_graph) mod declarations;
mod disclosure;
mod metadata_only;
mod planning;
mod portable;
pub(in crate::domain_computation::primary_graph) mod read;
use super::{
    probe,
    schema::*,
    world::{self, Graph, World},
};
use crate::domain_computation::primary_graph::*;
use declarations::*;

pub(in crate::domain_computation::primary_graph) fn world() -> World {
    world::install_orbit(
        Graph {
            nodes: 5,
            edges: vec![(0, 2), (2, 1), (1, 0), (3, 4), (4, 3)],
            active: false,
            blocked: vec![],
        },
        probe::Probe::new(|_, _, _| {}),
    )
}
#[test]
fn complete_declared_orbit_keeps_anchor_order_identity_and_original_result_custody() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = world();
    let result = read::run(&world, Orbit::reference(), 1);
    let selected = result.orbit(orbit::<OrbitSchema<true>>()).unwrap();
    let keys: Vec<_> = selected.members().iter().map(|row| row.key).collect();
    let (expected, native_identity) = native_oracle(&world);
    assert_eq!(keys, expected);
    assert_eq!(selected.identity(), &native_identity);
    assert_eq!(selected.len(), 3);
    let identity = selected.identity().clone();
    let other: worth_query_declaration::facade::application_query::ApplicationQueryOrbitRef<
        OrbitSchema<true>,
        Orbit,
        OtherSlot,
        Next,
        Node,
    > = worth_query_declaration::facade::application_query::ApplicationQueryOrbitRef::new(
        "orbit",
        Next::reference(),
    );
    assert!(result.orbit(other).is_none());
    let admitted = result.into_admitted_disclosed();
    assert_eq!(
        admitted
            .orbit(orbit::<OrbitSchema<true>>())
            .unwrap()
            .identity(),
        &identity
    );
    let demand = admitted.into_output_demand_source();
    assert_eq!(
        demand
            .orbit(orbit::<OrbitSchema<true>>())
            .unwrap()
            .members()
            .iter()
            .map(|row| row.key)
            .collect::<Vec<_>>(),
        keys
    );
    let sibling = read::run(&world, Orbit::reference(), 2);
    assert_eq!(
        sibling
            .orbit(orbit::<OrbitSchema<true>>())
            .unwrap()
            .identity(),
        &identity
    );
}

fn native_oracle(
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
