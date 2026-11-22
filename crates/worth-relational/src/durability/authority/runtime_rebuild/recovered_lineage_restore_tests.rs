//! Recovery treats `Restore` as a lineage allocation: the recovered identity
//! ledger refuses reused or non-advancing restored lineage ids of either
//! subject, the replay frontier of a `Restore`-only commit is its restored
//! lineage, and restored lineages count against allocator capacity.

use super::recovered_counter_capacity::{allocations_exhaust_allocator, first_allocated_lineage};
use super::recovered_lineage_artifacts::RecoveredLineageIdentityLedger;
use crate::durability::authority::lineage_allocation_fixtures::{restore_history, CommitLineage};
use crate::durability::data::RecoveryFailureClass;
use crate::identity::data::LineageId;
use crate::lineage::data::LineageSubject;

fn admit_all(history: &[CommitLineage]) -> Result<(), String> {
    let mut ledger = RecoveredLineageIdentityLedger::default();
    for commit in history {
        ledger
            .admit_commit_events(
                &commit.receipt,
                &commit.receipt.branch_id,
                &commit.events,
                RecoveryFailureClass::CorruptCanonicalArtifact,
            )
            .map_err(|error| {
                assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact);
                error.detail
            })?;
    }
    Ok(())
}

#[test]
fn restore_only_and_mixed_commits_are_admitted_with_fresh_ascending_lineages() {
    admit_all(&restore_history()).expect("restored lineages are fresh and ascending");
}

#[test]
fn reused_restored_lineages_of_either_subject_refuse() {
    for subject in [LineageSubject::Entity, LineageSubject::Relation] {
        let mut history = restore_history();
        let restore = history[2].allocating(5);
        restore.subject = subject;
        restore.targets = vec![LineageId(3)];
        let detail = admit_all(&history).expect_err("reused restored lineage");
        assert!(detail.contains("lineage id 3 is reused"), "{detail}");
    }
}

#[test]
fn non_advancing_and_malformed_restored_lineages_refuse() {
    let mut history = restore_history();
    history[1].allocating(2).targets = vec![LineageId(30)];
    history[1].allocating(3).targets = vec![LineageId(2)];
    let detail = admit_all(&history).expect_err("a restore below its commit's create");
    assert!(
        detail.contains("lineage id 2 does not advance after 30"),
        "{detail}"
    );

    let mut history = restore_history();
    history[2].allocating(5).sources = vec![LineageId(5)];
    let detail = admit_all(&history).expect_err("a self-restore is malformed");
    assert!(
        detail.contains("one source and one different target"),
        "{detail}"
    );
}

#[test]
fn restore_only_commits_set_the_replay_frontier_and_count_against_capacity() {
    let [_, mixed, mut restore_only] = restore_history();
    assert_eq!(
        first_allocated_lineage(&restore_only.events),
        Some(LineageId(5))
    );
    assert_eq!(first_allocated_lineage(&mixed.events), Some(LineageId(2)));
    assert!(!allocations_exhaust_allocator(&restore_only.events));
    restore_only.allocating(5).targets = vec![LineageId(u64::MAX - 1)];
    assert!(allocations_exhaust_allocator(&restore_only.events));
}
