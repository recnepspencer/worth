//! Minimum counts distinct actual targets across the full proposed mutation.
use super::{feeding::FeedsChange, installation::*, outcome::*, read, schema::*};
use worth_query_host::facade::{
    application_invariants::{EntityId, EntityReference},
    primary_graph::{WorthQueryApplicationCommitDenial, WorthQueryRelationLawDenial},
};
fn minimum(denial: &WorthQueryApplicationCommitDenial, source_id: EntityId, count: usize) {
    let Some(WorthQueryRelationLawDenial::ConnectivityMinimum {
        contract_id,
        source,
        reachable_target_count,
        minimum_reachable_targets,
        ..
    }) = denial.relation_law()
    else {
        panic!("original minimum refusal: {denial:?}")
    };
    assert_eq!(contract_id.as_str(), "Feeds:reaches_at_least");
    assert_eq!(source, &EntityReference::Existing(source_id));
    assert_eq!(reachable_target_count, count);
    assert_eq!(minimum_reachable_targets, 2);
}

#[test]
fn one_created_leaf_is_one_target_and_two_distinct_leaves_are_its_lawful_twin() {
    let mut world = install_with_feeds::<LeafMinimumTwo>(1, &[], 2, &[(0, 0), (0, 1)]);
    let before = read::feeds::committed(&world);
    let refusal = denied(world.edit_feeds(
        1,
        0,
        FeedsChange::Replace {
            first_leaf: 2,
            count: 1,
        },
    ));
    minimum(&refusal, before.sources[&0], 1);
    assert_eq!(read::feeds::identity(&world, 2), None);
    assert_eq!(read::feeds::committed(&world), before);
    committed(world.edit_feeds(
        2,
        0,
        FeedsChange::Replace {
            first_leaf: 2,
            count: 2,
        },
    ));
    let after = read::feeds::committed(&world);
    assert_eq!(after.sources, before.sources);
    assert_eq!(
        after.edges.into_iter().collect::<Vec<_>>(),
        vec![((0, 2), 1), ((0, 3), 1)]
    );
    assert_ne!(after.leaves[&2], after.leaves[&3]);
    world.close();
}

#[test]
fn deleting_a_target_rechecks_distinct_visible_targets_and_source_removal_removes_its_obligation() {
    let mut world = install_with_feeds::<LeafMinimumTwo>(1, &[], 2, &[(0, 0), (0, 1)]);
    let before = read::feeds::committed(&world);
    let refusal = denied(world.edit_feeds(10, 0, FeedsChange::DeleteLeaf { key: 1 }));
    minimum(&refusal, before.sources[&0], 1);
    assert_eq!(read::feeds::committed(&world), before);
    committed(world.edit_feeds(11, 0, FeedsChange::DeleteSource));
    assert_eq!(read::identity(&world, 0), None);
    assert_eq!(read::feeds::identity(&world, 0), Some(before.leaves[&0]));
    assert_eq!(read::feeds::identity(&world, 1), Some(before.leaves[&1]));
    world.close();

    let mut twin = install_with_feeds::<LeafMinimumTwo>(1, &[], 3, &[(0, 0), (0, 1), (0, 2)]);
    let before = read::feeds::committed(&twin);
    committed(twin.edit_feeds(12, 0, FeedsChange::DeleteLeaf { key: 1 }));
    assert_eq!(read::feeds::identity(&twin, 1), None);
    assert_eq!(read::feeds::identity(&twin, 0), Some(before.leaves[&0]));
    assert_eq!(read::feeds::identity(&twin, 2), Some(before.leaves[&2]));
    // A target deleted in the previous commit cannot satisfy the next minimum.
    let refusal = denied(twin.edit_feeds(13, 0, FeedsChange::DeleteLeaf { key: 0 }));
    minimum(&refusal, before.sources[&0], 1);
    assert_eq!(read::feeds::identity(&twin, 0), Some(before.leaves[&0]));

    twin.close();
}

#[test]
fn declared_unreachable_kinds_do_not_count_and_bootstrap_preserves_original_publication_fields() {
    use super::program::GraphProgram;
    use worth_query_host::facade::{
        application_installation::WorthQueryInMemoryApplicationDenial,
        declaration::{
            application_program::ApplicationProgramAuthoring, application_schema::ApplicationSchema,
        },
    };
    let schema = GraphSchema::<LeafAndPrincipalMinimumTwo>::declaration().unwrap();
    let program =
        ApplicationProgramAuthoring::<_, GraphProgram<LeafAndPrincipalMinimumTwo>>::begin()
            .with_schema(&schema)
            .validated_program()
            .unwrap();
    // A real Principal exists for authentication, but Feeds reaches only one Leaf.
    let refusal = match install_program(program, 1, &[], 1, &[(0, 0)]) {
        Err(WorthQueryInMemoryApplicationDenial::Publication(refusal)) => refusal,
        Err(other) => panic!("{other:?}"),
        Ok(world) => {
            world.close();
            panic!("unreachable principal cannot satisfy minimum2")
        }
    };
    let Some(WorthQueryRelationLawDenial::ConnectivityMinimum {
        contract_id,
        reachable_target_count,
        minimum_reachable_targets,
        ..
    }) = refusal.relation_law()
    else {
        panic!("{refusal:?}")
    };
    assert_eq!(contract_id.as_str(), "Feeds:reaches_at_least");
    assert_eq!(reachable_target_count, 1);
    assert_eq!(minimum_reachable_targets, 2);
    let twin = install_with_feeds::<LeafAndPrincipalMinimumTwo>(1, &[], 2, &[(0, 0), (0, 1)]);
    assert_eq!(read::feeds::committed(&twin).edges.len(), 2);
    twin.close();
}
