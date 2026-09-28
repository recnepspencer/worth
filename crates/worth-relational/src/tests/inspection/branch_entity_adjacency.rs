//! A view's per-entity adjacency reads its own branch root: a fork that wrote
//! after its parent still sees only its own relations, and the read is bounded
//! exactly like the version-addressed one.

use super::*;

const RELATION_KIND: crate::facade::identity::KindId = crate::facade::identity::KindId(2);

#[test]
fn a_fork_view_reads_its_own_adjacency_after_its_parent_writes_again() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let kept_target = create_entity(&runtime, "kept-target");
    let other_target = create_entity(&runtime, "other-target");
    let kept = create_relation(&runtime, source, kept_target, "kept");
    let feature = BranchId("feature".to_owned());
    runtime
        .history_authority()
        .fork_branch_from(feature.clone(), &BranchId("main".to_owned()))
        .expect("the fork publishes");
    delete_relation_on_branch(&runtime, kept, BranchId("main".to_owned()));
    let later = create_relation(&runtime, source, other_target, "later");
    // The fork writes last, so its own version is newer than both parent writes.
    let forked = crate::tests::support::create_relation_in_partition_on_branch(
        &runtime,
        source,
        other_target,
        "forked",
        "forked",
        crate::facade::identity::PartitionId::main(),
        feature.clone(),
    );

    let fork = branch_view(&runtime, &feature);
    let outgoing = fork
        .bounded_outgoing_relations_of_kind(source, RELATION_KIND, usize::MAX)
        .expect("an unbounded read completes");
    let work_units = outgoing.work_units();
    assert_eq!(relation_ids(outgoing), vec![kept, forked]);
    let incoming = fork
        .bounded_incoming_relations_of_kind(other_target, RELATION_KIND, usize::MAX)
        .expect("an unbounded read completes");
    assert_eq!(relation_ids(incoming), vec![forked]);

    let main = branch_view(&runtime, &BranchId("main".to_owned()));
    let outgoing = main
        .bounded_outgoing_relations_of_kind(source, RELATION_KIND, usize::MAX)
        .expect("an unbounded read completes");
    assert_eq!(relation_ids(outgoing), vec![later]);

    let exact = fork
        .bounded_outgoing_relations_of_kind(source, RELATION_KIND, work_units)
        .expect("the read fits its own work");
    assert_eq!(exact.work_units(), work_units);
    let refused = fork
        .bounded_outgoing_relations_of_kind(source, RELATION_KIND, work_units - 1)
        .expect_err("one unit short refuses before completing the fanout");
    assert_eq!(refused.consumed_work_units(), work_units - 1);
}

fn relation_ids(
    read: crate::facade::runtime::BoundedAdjacencyTruthRead,
) -> Vec<crate::facade::identity::RelationId> {
    read.into_records()
        .into_iter()
        .map(|record| record.relation_id)
        .collect()
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
