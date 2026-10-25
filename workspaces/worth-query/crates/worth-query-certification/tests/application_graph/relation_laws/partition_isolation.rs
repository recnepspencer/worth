//! Real SDK-created partitions, judged only by the native endpoint law.
use super::{installation::*, outcome::*, read, schema::*};
use worth_query_host::facade::{
    application_invariants::EntityReference,
    primary_graph::{WorthQueryApplicationCommitDenial, WorthQueryRelationLawDenial},
};

fn endpoints(denial: &WorthQueryApplicationCommitDenial) -> (&EntityReference, &EntityReference) {
    let Some(WorthQueryRelationLawDenial::PartitionIsolation {
        contract_id,
        source,
        target,
        ..
    }) = denial.relation_law()
    else {
        panic!("original partition law: {denial:?}")
    };
    assert_eq!(contract_id.as_str(), "Precedes:same_partition_endpoints");
    assert_ne!(source.partition_id(), target.partition_id());
    (source, target)
}

#[test]
fn new_target_uses_original_created_identity_and_contextual_twin_commits() {
    let mut world = install::<Isolated>(1, &[]);
    let before = read::committed(&world);
    let refusal = denied(world.create_node(0, false, true));
    let (source, target) = endpoints(&refusal);
    assert_eq!(source, &EntityReference::Existing(before.identities[&0]));
    assert!(matches!(target, EntityReference::Created(_)), "{target:?}");
    assert_eq!(world.nodes, 1);
    assert_eq!(
        read::identity(&world, 1),
        None,
        "refused target must actually be absent"
    );
    let after = read::committed(&world);
    assert_eq!(after.identities, before.identities);
    assert_eq!(after.multiplicity, before.multiplicity);
    committed(world.create_node(0, true, true));
    let after = read::committed(&world);
    assert_eq!(
        after.identities[&0].partition_id,
        after.identities[&1].partition_id
    );
    assert_eq!(
        after.multiplicity.into_iter().collect::<Vec<_>>(),
        vec![((0, 1), 1)]
    );
    world.close();
}

#[test]
fn existing_endpoints_from_separate_creations_refuse_with_the_original_ids() {
    let mut world = install::<Isolated>(1, &[]);
    committed(world.create_node(0, false, false));
    committed(world.create_node(1, true, false));
    let before = read::committed(&world);
    assert_ne!(
        before.identities[&0].partition_id,
        before.identities[&1].partition_id
    );
    assert_eq!(
        before.identities[&1].partition_id,
        before.identities[&2].partition_id
    );
    let refusal = denied(world.edit(10, &[(1, 2), (0, 1)]));
    let (source, target) = endpoints(&refusal);
    assert_eq!(source, &EntityReference::Existing(before.identities[&0]));
    assert_eq!(target, &EntityReference::Existing(before.identities[&1]));
    assert_eq!(read::committed(&world).multiplicity, before.multiplicity);
    committed(world.edit(11, &[(1, 2), (2, 1)]));
    assert_eq!(read::committed(&world).multiplicity.len(), 2);
    world.close();
}

#[test]
fn explicit_cross_context_endpoints_remain_legal_without_the_new_law() {
    let mut world = install::<CrossPartition>(1, &[]);
    committed(world.create_node(0, false, true));
    let graph = read::committed(&world);
    assert_ne!(
        graph.identities[&0].partition_id,
        graph.identities[&1].partition_id
    );
    assert_eq!(
        graph.multiplicity.into_iter().collect::<Vec<_>>(),
        vec![((0, 1), 1)]
    );
    world.close();
}

#[test]
fn seeded_edits_match_independent_groups_of_genuinely_created_entities() {
    let mut world = install::<Isolated>(1, &[]);
    // Independent model: [0, 1], [2, 3], [4, 5] are separate creation contexts.
    committed(world.create_node(0, true, false));
    committed(world.create_node(0, false, false));
    committed(world.create_node(2, true, false));
    committed(world.create_node(0, false, false));
    committed(world.create_node(4, true, false));
    let identities = read::committed(&world).identities;
    for a in 0..6 {
        for b in 0..6 {
            assert_eq!(
                identities[&a].partition_id == identities[&b].partition_id,
                a / 2 == b / 2
            );
        }
    }
    let mut expected = std::collections::BTreeMap::new();
    let mut seed = 0x9181_5041_u64;
    let (mut accepts, mut refuses) = (0, 0);
    for serial in 20..92 {
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            (seed >> 32) % 6
        };
        let (a, b) = (next(), next());
        let outcome = world.edit(serial, &[(a, b)]);
        if a / 2 == b / 2 {
            committed(outcome);
            *expected.entry((a, b)).or_insert(0) += 1;
            accepts += 1;
        } else {
            let denial = denied(outcome);
            let (source, target) = endpoints(&denial);
            assert_eq!(source, &EntityReference::Existing(identities[&a]));
            assert_eq!(target, &EntityReference::Existing(identities[&b]));
            refuses += 1;
        }
        let actual = read::committed(&world);
        assert_eq!(actual.identities, identities);
        assert_eq!(actual.multiplicity, expected, "edit {serial}: {a}->{b}");
    }
    assert!(accepts > 0 && refuses > 0);
    world.close();
}
