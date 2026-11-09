pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::relation_laws::endpoint_deletion::*;

#[test]
fn deleting_a_live_endpoint_preserves_its_typed_cause_and_unlinking_is_its_lawful_twin() {
    let mut world = install_with_feeds::<DeletionProtected>(1, &[], 1, &[(0, 0)]);
    let before = read::feeds::committed(&world);
    let refusal = denied(world.edit_feeds(1, 0, FeedsChange::DeleteLeaf { key: 0 }));
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

    committed(world.edit_feeds(2, 0, FeedsChange::UnlinkAndDeleteLeaf { key: 0 }));
    let after = read::feeds::committed(&world);
    assert_eq!(after.sources, before.sources);
    assert!(after.leaves.is_empty());
    assert!(after.edges.is_empty());
    world.close();
}
