//! Distinct reachability through cycles and repeated physical paths.
use super::{creation::CreateNodeInput, installation::*, outcome::*, read, schema::*};
use worth_query_host::facade::{
    application_invariants::EntityReference, primary_graph::WorthQueryRelationLawDenial,
};
const CYCLE: &[(u64, u64)] = &[(0, 1), (1, 2), (2, 0)];

#[test]
fn cycles_count_other_entities_once_and_neither_self_nor_duplicate_edges_substitute_for_targets() {
    let world = install::<MinimumTwo>(3, CYCLE);
    let before = read::committed(&world);
    // Extra physical records and a self-edge do not replace reachability through node1.
    committed(world.edit(1, &[(0, 1), (0, 0), (2, 1)]));
    let before_removal = read::committed(&world);
    let refusal = denied(world.replace_edges(2, &[], &[(0, 1)]));
    let Some(WorthQueryRelationLawDenial::ConnectivityMinimum {
        contract_id,
        source,
        reachable_target_count,
        minimum_reachable_targets,
        ..
    }) = refusal.relation_law()
    else {
        panic!("{refusal:?}")
    };
    assert_eq!(contract_id.as_str(), "Precedes:reaches_at_least");
    assert_eq!(source, &EntityReference::Existing(before.identities[&0]));
    assert_eq!(reachable_target_count, 0);
    assert_eq!(minimum_reachable_targets, 2);
    assert_eq!(
        read::committed(&world).multiplicity,
        before_removal.multiplicity
    );
    // Preserve one outgoing route while replacing the old direct one as a whole candidate.
    committed(world.replace_edges(3, &[(0, 2), (2, 1), (1, 0)], &[(0, 1)]));
    assert!(read::committed(&world).edges.contains(&(0, 2)));
    world.close();
}

#[test]
fn isolated_created_sources_refuse_and_a_return_edge_makes_the_same_creation_lawful() {
    let mut world = install::<MinimumTwo>(3, CYCLE);
    let before = read::committed(&world);
    let refusal = denied(world.create_node(0, true, true));
    let Some(WorthQueryRelationLawDenial::ConnectivityMinimum {
        source,
        reachable_target_count,
        minimum_reachable_targets,
        ..
    }) = refusal.relation_law()
    else {
        panic!("{refusal:?}")
    };
    let EntityReference::Created(created) = source else {
        panic!("unpublished source must retain its original creation reference: {source:?}")
    };
    assert_eq!(created.client_key.canonical_text(), "node-3");
    assert_eq!(reachable_target_count, 0);
    assert_eq!(minimum_reachable_targets, 2);
    assert_eq!(read::identity(&world, 3), None);
    assert_eq!(read::committed(&world).multiplicity, before.multiplicity);
    committed(world.create_with(CreateNodeInput {
        key: 3,
        context: 0,
        in_context: true,
        link: true,
        backlink: true,
        self_cycle: false,
    }));
    let after = read::committed(&world);
    assert!(after.edges.contains(&(0, 3)) && after.edges.contains(&(3, 0)));
    assert_eq!(read::identity(&world, 3), Some(after.identities[&3]));
    world.close();
}

#[test]
fn repeated_physical_paths_do_not_turn_one_distinct_reachable_target_into_two() {
    let world = install::<MinimumTwo>(3, CYCLE);
    committed(world.edit(20, &[(0, 1), (1, 0)]));
    committed(world.edit(21, &[(1, 0)]));
    let before = read::committed(&world);
    assert_eq!(before.multiplicity[&(0, 1)], 2);
    assert_eq!(before.multiplicity[&(1, 0)], 2);
    let refusal = denied(world.replace_edges(22, &[], &[(1, 2)]));
    let Some(WorthQueryRelationLawDenial::ConnectivityMinimum {
        source,
        reachable_target_count,
        minimum_reachable_targets,
        ..
    }) = refusal.relation_law()
    else {
        panic!("{refusal:?}")
    };
    assert!([before.identities[&0], before.identities[&1]]
        .iter()
        .any(|identity| source == &EntityReference::Existing(*identity)));
    assert_eq!(reachable_target_count, 1);
    assert_eq!(minimum_reachable_targets, 2);
    assert_eq!(read::committed(&world).multiplicity, before.multiplicity);
    committed(world.replace_edges(23, &[(0, 2)], &[(1, 2)]));
    world.close();
}
