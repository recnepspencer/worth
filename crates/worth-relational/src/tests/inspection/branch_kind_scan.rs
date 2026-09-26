//! A view's bounded kind scans read its own branch root: a fork sees its own
//! writes and never its parent's later ones.

use super::*;

const ENTITY_KIND: crate::facade::identity::KindId = crate::facade::identity::KindId(1);
const RELATION_KIND: crate::facade::identity::KindId = crate::facade::identity::KindId(2);

#[test]
fn a_fork_view_scans_its_own_writes_and_not_its_parents_later_ones() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let kept_target = create_entity(&runtime, "kept-target");
    let removed_target = create_entity(&runtime, "removed-target");
    let kept = create_relation(&runtime, source, kept_target, "kept");
    let removed = create_relation(&runtime, source, removed_target, "removed");
    let feature = BranchId("feature".to_owned());
    runtime
        .history_authority()
        .fork_branch_from(feature.clone(), &BranchId("main".to_owned()))
        .expect("the fork publishes");
    let forked = crate::tests::support::create_relation_in_partition_on_branch(
        &runtime,
        removed_target,
        kept_target,
        "forked",
        "forked",
        crate::facade::identity::PartitionId::main(),
        feature.clone(),
    );
    let forked_entity = crate::tests::support::create_entity_in_partition_on_branch(
        &runtime,
        "forked-entity",
        crate::facade::identity::PartitionId::main(),
        feature.clone(),
    );
    delete_relation_on_branch(&runtime, removed, feature.clone());
    let later = create_relation(&runtime, kept_target, removed_target, "later");
    let later_entity = create_entity(&runtime, "later-entity");

    let fork = branch_view_relations(&runtime, &feature, 4);
    assert_eq!(fork, vec![kept, forked]);
    let main = branch_view_relations(&runtime, &BranchId("main".to_owned()), 6);
    assert_eq!(main, vec![kept, removed, later]);

    let view = branch_view(&runtime, &feature);
    let entities = view
        .bounded_entities_of_kind(ENTITY_KIND, 8)
        .expect("four live fork entities cost eight units")
        .into_records()
        .into_iter()
        .map(|record| record.entity_id)
        .collect::<Vec<_>>();
    assert_eq!(
        entities,
        vec![source, kept_target, removed_target, forked_entity]
    );
    assert!(!entities.contains(&later_entity));
    let refused = view
        .bounded_relations_of_kind(RELATION_KIND, 3)
        .expect_err("one unit short refuses before completing the fork's inventory");
    let crate::facade::runtime::RelationKindTruthReadDenial::WorkLimitExceeded(refused) = refused
    else {
        panic!("the fork's inventory may fail only on its declared work bound");
    };
    assert_eq!(refused.consumed_work_units(), 3);
}

#[test]
fn a_historical_branch_view_scans_live_only_at_the_current_version() {
    let runtime = runtime_with_test_schema();
    let parent = create_entity(&runtime, "parent");
    let feature = BranchId("feature".to_owned());
    runtime
        .history_authority()
        .fork_branch_from(feature.clone(), &BranchId("main".to_owned()))
        .expect("the fork publishes");
    let later = create_entity(&runtime, "later");
    let forked = crate::tests::support::create_entity_in_partition_on_branch(
        &runtime,
        "forked",
        crate::facade::identity::PartitionId::main(),
        feature.clone(),
    );

    // The fork wrote last: its head is the current version, read live.
    let (fork, fork_slot_scans) = historical_entities(&runtime, &feature);
    assert_eq!(fork, vec![parent, forked]);
    assert_eq!(fork_slot_scans, 0, "a live read examines no retained slots");
    // Main's head is older: it is read at its version, never live.
    let (main, main_slot_scans) = historical_entities(&runtime, &BranchId("main".to_owned()));
    assert_eq!(main, vec![parent, later]);
    assert!(main_slot_scans > 0, "an older head is read at its version");
}

/// The branch's entities through the retained historical basis of its head,
/// with the retained entity slots the read examined.
fn historical_entities(
    runtime: &crate::runtime::RelationalRuntime,
    branch: &BranchId,
) -> (Vec<crate::facade::identity::EntityId>, usize) {
    let head = runtime
        .history()
        .branch_head(branch)
        .expect("the branch has a head");
    let view = runtime
        .read_truth()
        .try_project_historical_version(head.version_id)
        .expect("the branch head is retained");
    let before = runtime.performance_access().counters();
    let entities = view
        .bounded_entities_of_kind(ENTITY_KIND, 4)
        .expect("two entities cost four units")
        .into_records()
        .into_iter()
        .map(|record| record.entity_id)
        .collect();
    let after = runtime.performance_access().counters();
    (
        entities,
        after.visibility_entity_slot_scans - before.visibility_entity_slot_scans,
    )
}

fn branch_view<'runtime>(
    runtime: &'runtime crate::runtime::RelationalRuntime,
    branch: &BranchId,
) -> crate::facade::runtime::VisibilityProjectionView<'runtime> {
    let head = runtime
        .history()
        .branch_head(branch)
        .expect("the branch has a head");
    runtime
        .read_truth()
        .project_branch_head(branch, head.version_id)
        .expect("the branch head is admitted")
        .expect("the branch head is retained")
}

fn branch_view_relations(
    runtime: &crate::runtime::RelationalRuntime,
    branch: &BranchId,
    maximum_work_units: usize,
) -> Vec<crate::facade::identity::RelationId> {
    let read = branch_view(runtime, branch)
        .bounded_relations_of_kind(RELATION_KIND, maximum_work_units)
        .expect("the branch's live relations fit the budget");
    assert_eq!(read.work_units(), maximum_work_units);
    read.into_records()
        .into_iter()
        .map(|record| record.relation_id)
        .collect()
}
