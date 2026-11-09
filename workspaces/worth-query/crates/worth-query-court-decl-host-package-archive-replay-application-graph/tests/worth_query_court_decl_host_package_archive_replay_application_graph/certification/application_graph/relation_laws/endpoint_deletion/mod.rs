pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::relation_laws::endpoint_deletion::*;

#[test]
fn deleting_a_live_endpoint_precedes_the_cardinality_consequence_and_preserves_its_subject() {
    let mut world = install_with_feeds::<DeletionProtected>(1, &[], 1, &[(0, 0)]);
    let before = read::feeds::committed(&world);
    let refusal = denied(world.edit_feeds(1, 0, FeedsChange::DeleteLeaf { key: 0 }));
    assert_eq!(
        refusal.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert!(refusal.invariant_execution_failure().is_some());
    let Some(WorthQueryRelationLawDenial::EndpointDeletion {
        contract_id,
        entity_id,
        remaining_relation_endpoint_count,
        mode,
        cascade_delete_policy,
        ..
    }) = refusal.relation_law()
    else {
        panic!("original endpoint deletion refusal required: {refusal:?}")
    };
    assert_eq!(contract_id.as_str(), "Feeds:deletion");
    assert_eq!(*entity_id, before.leaves[&0]);
    assert_eq!(remaining_relation_endpoint_count, 1);
    assert_eq!(
        mode,
        EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations
    );
    assert_eq!(cascade_delete_policy, None);
    assert_eq!(read::feeds::committed(&world), before);

    world.close();
}

#[test]
fn explicitly_unlinking_and_deleting_the_target_refuses_as_source_cardinality() {
    let mut world = install_with_feeds::<DeletionProtected>(1, &[], 1, &[(0, 0)]);
    let before = read::feeds::committed(&world);
    let refusal = denied(world.edit_feeds(1, 0, FeedsChange::UnlinkAndDeleteLeaf { key: 0 }));
    let Some(WorthQueryRelationLawDenial::Cardinality {
        contract_id,
        member,
        boundary,
        count,
        limit,
        ..
    }) = refusal.relation_law()
    else {
        panic!("explicit unlink must retain its cardinality refusal: {refusal:?}")
    };
    assert_eq!(contract_id.as_str(), "Feeds:cardinality");
    assert_eq!(member, &EntityReference::Existing(before.sources[&0]));
    assert_eq!(boundary, RelationCardinalityBoundary::Source);
    assert_eq!(count, 0);
    assert_eq!(limit, 1);
    assert_eq!(read::feeds::committed(&world), before);
    world.close();
}

#[test]
fn deleting_a_target_without_live_edges_commits_with_exact_source_cardinality() {
    let mut world = install_with_feeds::<DeletionProtected>(1, &[], 2, &[(0, 0)]);
    let before = read::feeds::committed(&world);
    committed(world.edit_feeds(1, 0, FeedsChange::DeleteLeaf { key: 1 }));
    let after = read::feeds::committed(&world);
    assert_eq!(after.sources, before.sources);
    assert_eq!(after.edges, before.edges);
    assert_eq!(after.leaves.len(), 1);
    assert_eq!(after.leaves[&0], before.leaves[&0]);
    assert!(!after.leaves.contains_key(&1));
    world.close();
}
