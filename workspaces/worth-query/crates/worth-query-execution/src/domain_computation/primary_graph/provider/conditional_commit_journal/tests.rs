use super::*;
use worth_relational::facade::identity::{EntityId, PartitionId};

fn entity(slot: u32) -> RecordRef {
    RecordRef::Entity(EntityId::new(PartitionId(1), u64::from(slot), 1))
}

fn commit(id: u64, branch: u64) -> RelationalCommitReceipt {
    RelationalCommitReceipt {
        commit_id: CommitId(id),
        version_id: worth_relational::facade::identity::VersionId(id),
        branch_id: branch_id(branch),
        parents: Vec::new(),
    }
}

fn branch_id(id: u64) -> BranchId {
    BranchId(format!("branch-{id}"))
}
#[test]
fn same_kind_unrelated_records_do_not_enter_exact_record_route() {
    let mut journal = WorthQueryConditionalCommitJournal::default();
    journal.replace_routes([entity(7)], false);
    journal.record(&commit(1, 1), [entity(8)]);
    journal.record(&commit(2, 1), [entity(9)]);

    let batch = journal
        .after_records(&branch_id(1), Some(CommitId(2)), 0, 8, [entity(7)], false)
        .unwrap();
    assert!(batch.commits.is_empty());
    assert_eq!(batch.cursor, 2);
}

#[test]
fn exact_record_route_retains_only_matching_commits() {
    let mut journal = WorthQueryConditionalCommitJournal::default();
    journal.replace_routes([entity(7)], false);
    journal.record(&commit(1, 1), [entity(7)]);
    journal.record(&commit(2, 1), [entity(8)]);

    assert_eq!(
        journal
            .after_records(&branch_id(1), Some(CommitId(2)), 0, 8, [entity(7)], false)
            .unwrap()
            .commits,
        vec![(1, CommitId(1))]
    );
}

#[test]
fn whole_graph_route_admits_every_committed_record() {
    let mut journal = WorthQueryConditionalCommitJournal::default();
    journal.replace_routes(std::iter::empty(), true);
    journal.record(&commit(1, 1), [entity(8)]);
    journal.record(&commit(2, 1), [entity(9)]);

    let batch = journal
        .after_records(
            &branch_id(1),
            Some(CommitId(2)),
            0,
            8,
            std::iter::empty(),
            true,
        )
        .unwrap();
    assert_eq!(batch.commits, vec![(1, CommitId(1)), (2, CommitId(2))]);
    assert!(batch.caught_up_to_latest);
}

#[test]
fn exact_route_reports_only_its_own_real_overrun() {
    let mut journal = WorthQueryConditionalCommitJournal::default();
    journal.replace_routes([entity(7)], false);
    for commit in 1..=(MAX_RETAINED_CONDITIONAL_COMMITS as u64 + 1) {
        journal.record(&self::commit(commit, 1), [entity(7)]);
    }
    assert!(journal
        .after_records(
            &branch_id(1),
            Some(CommitId(MAX_RETAINED_CONDITIONAL_COMMITS as u64 + 1)),
            0,
            1,
            [entity(7)],
            false,
        )
        .is_err());
}

#[test]
fn unrelated_commits_beyond_capacity_neither_scan_nor_overrun_exact_route() {
    let mut journal = WorthQueryConditionalCommitJournal::default();
    journal.replace_routes([entity(7)], false);
    for commit in 1..=(MAX_RETAINED_CONDITIONAL_COMMITS as u64 + 1) {
        journal.record(&self::commit(commit, 1), [entity(8)]);
    }
    let batch = journal
        .after_records(
            &branch_id(1),
            Some(CommitId(MAX_RETAINED_CONDITIONAL_COMMITS as u64 + 1)),
            0,
            1,
            [entity(7)],
            false,
        )
        .unwrap();
    assert!(batch.commits.is_empty());
    assert_eq!(batch.cursor, MAX_RETAINED_CONDITIONAL_COMMITS as u64 + 1);
    assert!(journal.exact_routes[&entity(7)]
        .branch(&branch_id(1))
        .is_none_or(|route| route.entries.is_empty()));
}

#[test]
fn route_replacement_bounds_record_identity_inventory() {
    let mut journal = WorthQueryConditionalCommitJournal::default();
    journal.replace_routes([entity(7), entity(8)], false);
    journal.replace_routes([entity(9)], false);
    assert_eq!(
        journal.exact_routes.keys().cloned().collect::<Vec<_>>(),
        vec![entity(9)]
    );
}

#[test]
fn exact_route_excludes_same_record_commits_from_sibling_branches() {
    let mut journal = WorthQueryConditionalCommitJournal::default();
    journal.replace_routes([entity(7)], false);
    journal.record(&commit(1, 1), [entity(7)]);
    journal.record(&commit(2, 2), [entity(7)]);

    assert_eq!(
        journal
            .after_records(&branch_id(2), Some(CommitId(2)), 0, 8, [entity(7)], false)
            .unwrap()
            .commits,
        vec![(2, CommitId(2))]
    );
}

#[test]
fn capacity_overrun_is_scoped_to_the_selected_branch() {
    let mut journal = WorthQueryConditionalCommitJournal::default();
    journal.replace_routes([entity(7)], false);
    for id in 1..=(MAX_RETAINED_CONDITIONAL_COMMITS as u64 + 1) {
        journal.record(&commit(id, 1), [entity(7)]);
    }

    let sibling = journal
        .after_records(
            &branch_id(2),
            Some(CommitId(MAX_RETAINED_CONDITIONAL_COMMITS as u64 + 1)),
            0,
            1,
            [entity(7)],
            false,
        )
        .unwrap();
    assert!(sibling.commits.is_empty());
    assert_eq!(sibling.cursor, MAX_RETAINED_CONDITIONAL_COMMITS as u64 + 1);
}

#[test]
fn retained_product_ceiling_excludes_later_branch_commits() {
    let mut journal = WorthQueryConditionalCommitJournal::default();
    journal.replace_routes([entity(7)], false);
    journal.record(&commit(1, 1), [entity(7)]);
    journal.record(&commit(2, 1), [entity(7)]);

    let retained = journal
        .after_records(&branch_id(1), Some(CommitId(1)), 0, 8, [entity(7)], false)
        .unwrap();
    assert_eq!(retained.commits, vec![(1, CommitId(1))]);
    assert_eq!(retained.cursor, 1);
    assert!(!retained.caught_up_to_latest);

    let successor = journal
        .after_records(
            &branch_id(1),
            Some(CommitId(2)),
            retained.cursor,
            8,
            [entity(7)],
            false,
        )
        .unwrap();
    assert_eq!(successor.commits, vec![(2, CommitId(2))]);
    assert_eq!(successor.cursor, 2);
    assert!(successor.caught_up_to_latest);
}

#[test]
fn bootstrap_narrowing_rejects_a_stale_frontier_and_preserves_the_raced_commit() {
    let mut journal = WorthQueryConditionalCommitJournal::default();
    journal.replace_routes([entity(7)], false);
    journal.replace_bootstrap_routes(["operation".to_string()]);
    journal.record(&commit(1, 1), [entity(8)]);

    let caught_up = journal
        .after_records_with_bootstrap(
            &branch_id(1),
            Some(CommitId(1)),
            0,
            8,
            [entity(7)],
            false,
            Some("operation"),
        )
        .unwrap();
    assert_eq!(caught_up.commits, vec![(1, CommitId(1))]);
    assert!(caught_up.caught_up_to_latest);

    // This commit models publication after the catch-up read and before the
    // lifecycle asks the journal to narrow the bootstrap route.
    journal.record(&commit(2, 1), [entity(8)]);
    assert!(!journal.narrow_bootstrap_route_if_current("operation", caught_up.cursor));

    let raced = journal
        .after_records_with_bootstrap(
            &branch_id(1),
            Some(CommitId(2)),
            caught_up.cursor,
            8,
            [entity(7)],
            false,
            Some("operation"),
        )
        .unwrap();
    assert_eq!(raced.commits, vec![(2, CommitId(2))]);
    assert!(journal.narrow_bootstrap_route_if_current("operation", raced.cursor));

    // A concurrent route inventory publication cannot resurrect a completed
    // bootstrap route from its stale operation snapshot.
    journal.replace_bootstrap_routes(["operation".to_string()]);
    assert!(!journal.bootstrap_routes.contains_key("operation"));
}

#[test]
fn historical_ceiling_detects_an_older_drop_after_a_later_drop() {
    let mut journal = WorthQueryConditionalCommitJournal::default();
    journal.replace_routes([entity(7)], false);
    for id in 1..=(MAX_RETAINED_CONDITIONAL_COMMITS as u64 + 2) {
        journal.record(&commit(id, 1), [entity(7)]);
    }

    assert!(journal
        .after_records(&branch_id(1), Some(CommitId(1)), 0, 8, [entity(7)], false)
        .is_err());
}
