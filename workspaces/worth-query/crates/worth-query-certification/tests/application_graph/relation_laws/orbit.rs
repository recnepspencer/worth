//! The stored permutation law is judged by the ordinary native commit path.
use super::{creation::CreateNodeInput, installation::*, outcome::*, read, schema::*};
use worth_query_host::facade::{
    application_invariants::{EntityReference, RelationCardinalityBoundary as Side},
    primary_graph::WorthQueryRelationLawDenial,
};

const CYCLE: &[(u64, u64)] = &[(0, 1), (1, 2), (2, 0)];

#[test]
fn bootstrap_requires_a_complete_permutation_before_publication() {
    use super::program::GraphProgram;
    use worth_query_host::facade::{
        application_installation::WorthQueryInMemoryApplicationDenial,
        declaration::{
            application_program::ApplicationProgramAuthoring, application_schema::ApplicationSchema,
        },
    };
    let schema = GraphSchema::<Orbit>::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<_, GraphProgram<Orbit>>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    let refusal = match install_program(program, 1, &[], 0, &[]) {
        Err(WorthQueryInMemoryApplicationDenial::Publication(refusal)) => refusal,
        Err(other) => panic!("native bootstrap publication refusal required: {other:?}"),
        Ok(world) => {
            world.close();
            panic!("an isolated member cannot bootstrap an orbit")
        }
    };
    let Some(WorthQueryRelationLawDenial::Cardinality {
        contract_id,
        count,
        limit,
        ..
    }) = refusal.relation_law()
    else {
        panic!("original bootstrap cardinality cause required: {refusal:?}")
    };
    assert_eq!(contract_id.as_str(), "Precedes:orbit");
    assert_eq!((count, limit), (0, 1));
    let twin = install::<Orbit>(1, &[(0, 0)]);
    assert_eq!(read::committed(&twin).edges, [(0, 0)].into());
    twin.close();
}

#[test]
fn open_chains_and_excess_physical_edges_name_the_original_member() {
    let world = install::<Orbit>(3, CYCLE);
    let before = read::committed(&world);
    // Remove a successor, duplicate a physical edge, then redirect only one end.
    for (serial, additions, removals, count, expected) in [
        (
            1,
            vec![],
            vec![(2, 0)],
            0,
            vec![(2, Side::Source), (0, Side::Target)],
        ),
        (
            2,
            vec![(0, 1)],
            vec![],
            2,
            vec![(0, Side::Source), (1, Side::Target)],
        ),
        (3, vec![(0, 2)], vec![(0, 1)], 2, vec![(2, Side::Target)]),
        (4, vec![(2, 1)], vec![(0, 1)], 2, vec![(2, Side::Source)]),
    ] {
        let refusal = denied(world.replace_edges(serial, &additions, &removals));
        let Some(WorthQueryRelationLawDenial::Cardinality {
            contract_id,
            member: EntityReference::Existing(member),
            boundary,
            count: actual,
            limit,
            ..
        }) = refusal.relation_law()
        else {
            panic!("original native cardinality member required: {refusal:?}")
        };
        assert_eq!(contract_id.as_str(), "Precedes:orbit");
        assert_eq!((actual, limit), (count, 1));
        assert!(expected
            .iter()
            .any(|&(key, side)| { *member == before.identities[&key] && boundary == side }));
        assert_eq!(read::committed(&world).multiplicity, before.multiplicity);
    }
    // The lawful twin replaces both successors in one candidate, splitting the cycle.
    committed(world.replace_edges(5, &[(0, 0), (2, 1)], &[(0, 1), (2, 0)]));
    assert_eq!(
        read::committed(&world).edges,
        [(0, 0), (1, 2), (2, 1)].into()
    );
    // Merging the cycles remains a permutation. No aggregate identity is asserted here.
    committed(world.replace_edges(6, &[(0, 1), (2, 0)], &[(0, 0), (2, 1)]));
    assert_eq!(read::committed(&world).multiplicity, before.multiplicity);
    world.close();
}

#[test]
fn every_created_member_requires_edges_and_a_self_cycle_is_a_lawful_creation() {
    let mut world = install::<Orbit>(1, &[(0, 0)]);
    let before = read::committed(&world);
    let refusal = denied(world.create_node(0, true, false));
    let Some(WorthQueryRelationLawDenial::Cardinality {
        contract_id,
        member: EntityReference::Created(_),
        count,
        limit,
        ..
    }) = refusal.relation_law()
    else {
        panic!("isolated created member must reach native cardinality: {refusal:?}")
    };
    assert_eq!(contract_id.as_str(), "Precedes:orbit");
    // This proposal creates exactly one member. The original typed reference
    // may carry a normalized key; its display is not a raw-key recovery API.
    assert_eq!((count, limit), (0, 1));
    assert_eq!(read::identity(&world, 1), None);
    assert_eq!(read::committed(&world).multiplicity, before.multiplicity);
    committed(world.create_with(CreateNodeInput {
        key: 1,
        context: 0,
        in_context: true,
        link: false,
        backlink: false,
        self_cycle: true,
    }));
    assert!(read::identity(&world, 1).is_some());
    assert_eq!(read::committed(&world).edges, [(0, 0), (1, 1)].into());
    world.close();
}

#[test]
fn orbit_preserves_a_stricter_independent_self_edge_policy() {
    let world = install::<NoSelfOrbit>(3, CYCLE);
    let before = read::committed(&world);
    let refusal = denied(world.replace_edges(1, &[(0, 0), (2, 1)], &[(0, 1), (2, 0)]));
    let Some(WorthQueryRelationLawDenial::SelfEdge {
        contract_id,
        source: EntityReference::Existing(source),
        target: EntityReference::Existing(target),
        self_edge,
        ..
    }) = refusal.relation_law()
    else {
        panic!("independent native endpoint policy required: {refusal:?}")
    };
    assert_eq!(contract_id.as_str(), "Precedes:endpoints");
    assert_eq!(
        (*source, *target),
        (before.identities[&0], before.identities[&0])
    );
    assert!(self_edge);
    assert_eq!(read::committed(&world).multiplicity, before.multiplicity);
    committed(world.replace_edges(2, &[(0, 2), (2, 1), (1, 0)], CYCLE));
    assert_eq!(
        read::committed(&world).edges,
        [(0, 2), (2, 1), (1, 0)].into()
    );
    world.close();
}
