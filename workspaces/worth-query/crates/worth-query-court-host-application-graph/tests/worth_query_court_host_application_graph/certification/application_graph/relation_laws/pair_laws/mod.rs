pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::relation_laws::pair_laws::*;

#[test]
fn paired_insertion_and_single_or_paired_deletion_judge_surviving_edges() {
    let world = install::<Symmetric>(3, &[]);
    let before = read::committed(&world);
    symmetry(&denied(world.edit(1, &[(0, 1)])), &before, &[(0, 1)]);
    assert_eq!(read::committed(&world).multiplicity, before.multiplicity);
    committed(world.edit(2, &[(0, 1), (1, 0)]));
    let paired = read::committed(&world);
    symmetry(
        &denied(world.replace_edges(3, &[], &[(0, 1)])),
        &paired,
        &[(1, 0)],
    );
    assert_eq!(read::committed(&world).multiplicity, paired.multiplicity);
    committed(world.replace_edges(4, &[], &[(0, 1), (1, 0)]));
    assert!(read::committed(&world).multiplicity.is_empty());
    world.close();
}

#[test]
fn an_unrelated_pair_at_the_new_target_does_not_cause_a_false_symmetry_refusal() {
    let world = install::<Symmetric>(3, &[(0, 1), (1, 0), (1, 2), (2, 1)]);
    committed(world.edit(1, &[(0, 1)]));
    assert_eq!(
        read::committed(&world).multiplicity,
        BTreeMap::from([((0, 1), 2), ((1, 0), 1), ((1, 2), 1), ((2, 1), 1)])
    );
    world.close();
}

#[test]
fn a_balanced_replacement_removes_the_old_pair_and_installs_the_new_pair_atomically() {
    let world = install::<Symmetric>(3, &[(0, 1), (1, 0)]);
    let before = read::committed(&world);
    symmetry(
        &denied(world.replace_edges(1, &[(0, 2)], &[(0, 1)])),
        &before,
        &[(1, 0), (0, 2)],
    );
    assert_eq!(read::committed(&world).multiplicity, before.multiplicity);
    committed(world.replace_edges(2, &[(0, 2), (2, 0)], &[(0, 1), (1, 0)]));
    assert_eq!(
        read::committed(&world).multiplicity,
        BTreeMap::from([((0, 2), 1), ((2, 0), 1)])
    );
    world.close();
}

#[test]
fn a_permitted_self_edge_is_its_own_inverse_and_directed_uniqueness_composes() {
    let world = install::<SymmetricDirectedUnique>(2, &[]);
    committed(world.edit(1, &[(0, 0)]));
    committed(world.edit(2, &[(0, 1), (1, 0)]));
    let before = read::committed(&world);
    uniqueness(
        &denied(world.edit(3, &[(0, 1)])),
        &before,
        (0, 1),
        UniquenessScope::DirectedSemanticEdge,
    );
    assert_eq!(read::committed(&world).multiplicity, before.multiplicity);
    committed(world.replace_edges(4, &[], &[(0, 0)]));
    assert_eq!(
        read::committed(&world).multiplicity,
        BTreeMap::from([((0, 1), 1), ((1, 0), 1)])
    );
    world.close();
}

#[test]
fn directed_uniqueness_refuses_duplicates_in_existing_and_proposed_edges() {
    let world = install::<DirectedUnique>(3, &[(0, 1)]);
    let before = read::committed(&world);
    uniqueness(
        &denied(world.edit(1, &[(0, 1)])),
        &before,
        (0, 1),
        UniquenessScope::DirectedSemanticEdge,
    );
    uniqueness(
        &denied(world.edit(2, &[(0, 1), (1, 2)])),
        &before,
        (0, 1),
        UniquenessScope::DirectedSemanticEdge,
    );
    assert_eq!(read::committed(&world).multiplicity, before.multiplicity);
    committed(world.edit(3, &[(1, 0), (1, 2)]));
    let before_replace = read::committed(&world).multiplicity;
    committed(world.replace_edges(4, &[(0, 1)], &[(0, 1)]));
    assert_eq!(read::committed(&world).multiplicity, before_replace);
    world.close();
}

#[test]
fn normalized_uniqueness_counts_inverse_records_together() {
    let world = install::<NormalizedUnique>(3, &[(0, 1)]);
    let before = read::committed(&world);
    uniqueness(
        &denied(world.edit(1, &[(1, 0)])),
        &before,
        (0, 1),
        UniquenessScope::NormalizedSymmetricEdge,
    );
    uniqueness(
        &denied(world.edit(2, &[(1, 2), (2, 1)])),
        &before,
        (1, 2),
        UniquenessScope::NormalizedSymmetricEdge,
    );
    assert_eq!(read::committed(&world).multiplicity, before.multiplicity);
    committed(world.edit(3, &[(1, 2)]));
    committed(world.replace_edges(4, &[(1, 0)], &[(0, 1)]));
    assert_eq!(
        read::committed(&world).multiplicity,
        BTreeMap::from([((1, 0), 1), ((1, 2), 1)])
    );
    world.close();
    let twin = install::<Unrestricted>(2, &[(0, 1)]);
    committed(twin.edit(1, &[(0, 1), (1, 0)]));
    assert_eq!(
        read::committed(&twin).multiplicity,
        BTreeMap::from([((0, 1), 2), ((1, 0), 1)]),
        "readout must not deduplicate physical records"
    );
    twin.close();
}
