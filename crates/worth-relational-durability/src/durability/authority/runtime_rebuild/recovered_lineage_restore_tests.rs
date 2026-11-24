//! Recovery treats `Restore` as a lineage allocation: the recovered identity
//! ledger refuses reused or non-advancing restored lineage ids of either
//! subject, the replay frontier of a `Restore`-only commit is its restored
//! lineage, and every `Create` or `Restore` of either subject counts against
//! allocator capacity and the recovered lineage-id frontier.

use super::recovered_counter_capacity::{
    allocations_exhaust_allocator, first_allocated_lineage, recovered_lineage_frontier,
};
use super::recovered_lineage_artifacts::RecoveredLineageIdentityLedger;
use crate::durability::authority::lineage_allocation_fixtures::{restore_history, CommitLineage};
use contracts::durability::data::recovery_errors::RecoveryFailureClass;
use contracts::identity::data::primitives::LineageId;
use contracts::lineage::data::events::{LineageEventRecordFieldsPort as _, LineageSubject};
use worth_relational_contracts::internal_port as contracts;

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
        (*restore.owner_subject_mut()) = subject;
        (*restore.owner_targets_mut()) = vec![LineageId(3)];
        let detail = admit_all(&history).expect_err("reused restored lineage");
        assert!(detail.contains("lineage id 3 is reused"), "{detail}");
    }
}

#[test]
fn non_advancing_and_malformed_restored_lineages_refuse() {
    let mut history = restore_history();
    (*history[1].allocating(2).owner_targets_mut()) = vec![LineageId(30)];
    (*history[1].allocating(3).owner_targets_mut()) = vec![LineageId(2)];
    let detail = admit_all(&history).expect_err("a restore below its commit's create");
    assert!(
        detail.contains("lineage id 2 does not advance after 30"),
        "{detail}"
    );

    let mut history = restore_history();
    (*history[2].allocating(5).owner_sources_mut()) = vec![LineageId(5)];
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
    (*restore_only.allocating(5).owner_targets_mut()) = vec![LineageId(u64::MAX - 1)];
    assert!(allocations_exhaust_allocator(&restore_only.events));
}

#[test]
fn relation_allocations_above_every_entity_node_advance_the_recovered_frontier() {
    let [first, mixed, mut restore_only] = restore_history();
    let entity_node_max = Some(3);
    assert_eq!(
        recovered_lineage_frontier(entity_node_max, first.events.iter().chain(&mixed.events)),
        Ok(5),
        "a relation Create at lineage 4 outranks entity node 3"
    );
    let restore = restore_only.allocating(5);
    (*restore.owner_subject_mut()) = LineageSubject::Relation;
    (*restore.owner_targets_mut()) = vec![LineageId(7)];
    assert_eq!(
        recovered_lineage_frontier(entity_node_max, &restore_only.events),
        Ok(8),
        "a relation Restore at lineage 7 outranks every entity node"
    );
    let relation_only = [mixed.events[2].clone()];
    assert_eq!(relation_only[0].subject(), LineageSubject::Relation);
    assert_eq!(recovered_lineage_frontier(None, &relation_only), Ok(5));
    assert_eq!(recovered_lineage_frontier(Some(9), &relation_only), Ok(10));
}

#[test]
fn an_exhausting_relation_allocation_refuses_the_recovered_frontier() {
    let [_, mixed, _] = restore_history();
    let mut relation_only = [mixed.events[2].clone()];
    (*relation_only[0].owner_targets_mut()) = vec![LineageId(u64::MAX - 1)];
    let error = recovered_lineage_frontier(None, &relation_only)
        .expect_err("no safe lineage id follows the relation allocation");
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error
        .detail
        .contains("lineage id has no safe next transition"));
}
