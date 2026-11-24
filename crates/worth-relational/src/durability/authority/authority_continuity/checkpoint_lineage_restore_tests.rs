//! Checkpoint lineage validation accounts for `Restore` allocations exactly
//! as for `Create`: each binds one created entity to one canonical node, and
//! no allocated lineage of either subject repeats.

use super::checkpoint_lineage::validate_entity_allocation_nodes;
use crate::durability::authority::lineage_allocation_fixtures::{
    entity, restore_history, restore_history_nodes, CommitLineage,
};
use crate::durability::data::RecoveryFailureClass;
use crate::identity::data::LineageId;
use crate::lineage::data::LineageSubject;
use worth_relational_contracts::internal_port::lineage::data::events::LineageEventRecordFieldsPort as _;

fn validate(commits: &[CommitLineage]) -> Result<(), String> {
    validate_entity_allocation_nodes(
        &restore_history_nodes(),
        commits.iter().map(CommitLineage::view),
    )
    .map_err(|error| {
        assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
        error.detail
    })
}

#[test]
fn restore_only_and_mixed_create_restore_commits_validate_with_exact_nodes() {
    let history = restore_history();
    validate(&history).expect("every restored entity binds its canonical node");
}

#[test]
fn reused_restore_lineages_of_either_subject_refuse() {
    for subject in [LineageSubject::Entity, LineageSubject::Relation] {
        let mut history = restore_history();
        let restore = history[2].allocating(5);
        (*restore.owner_subject_mut()) = subject;
        (*restore.owner_targets_mut()) = vec![LineageId(4)];
        let detail = validate(&history).expect_err("a reused restored lineage refuses");
        assert!(detail.contains("lineage id 4 is reused"), "{detail}");
    }
}

#[test]
fn restores_bound_to_the_wrong_entity_node_or_subject_refuse() {
    let mut history = restore_history();
    (*history[1].allocating(2).owner_targets_mut()) = vec![LineageId(30)];
    (*history[1].allocating(3).owner_targets_mut()) = vec![LineageId(2)];
    (*history[1].allocating(30).owner_targets_mut()) = vec![LineageId(3)];
    let detail = validate(&history).expect_err("swapped allocations bind the wrong entities");
    assert!(
        detail.contains("does not name its exact entity"),
        "{detail}"
    );

    let mut history = restore_history();
    (*history[2].allocating(5).owner_targets_mut()) = vec![LineageId(50)];
    let detail = validate(&history).expect_err("a restore with no canonical node refuses");
    assert!(
        detail.contains("lineage id 50 has no canonical node"),
        "{detail}"
    );

    let mut history = restore_history();
    (*history[2].allocating(5).owner_subject_mut()) = LineageSubject::Relation;
    let detail = validate(&history).expect_err("a relation restore cannot bind an entity");
    assert!(
        detail.contains("binds 0 entity lineage allocations to 1"),
        "{detail}"
    );

    let mut history = restore_history();
    history[2].created_entities = vec![entity(6)];
    let detail = validate(&history).expect_err("the restored node names another entity");
    assert!(
        detail.contains("does not name its exact entity"),
        "{detail}"
    );
}

#[test]
fn malformed_restore_shapes_refuse_before_binding() {
    for sources in [vec![], vec![LineageId(1), LineageId(2)], vec![LineageId(5)]] {
        let mut history = restore_history();
        (*history[2].allocating(5).owner_sources_mut()) = sources;
        let detail = validate(&history).expect_err("malformed restore refuses");
        assert!(
            detail.contains("one source and one different target"),
            "{detail}"
        );
    }
}

#[test]
fn a_selection_restore_binds_its_named_node_and_a_forged_binding_refuses() {
    use crate::lineage::data::LineageEventKind;
    let selection = |named: u64| {
        let mut commit = CommitLineage::new(4, Vec::new());
        commit.rematerialized_entities = vec![entity(6), entity(7)];
        commit.push(
            LineageEventKind::Restore,
            LineageSubject::Entity,
            &[5],
            &[6],
        );
        (*commit.allocating(6).owner_entity_id_mut()) = Some(entity(named));
        commit
    };
    let mut nodes = restore_history_nodes();
    nodes.insert(LineageId(6), entity(6));
    let validate_with = |commit: CommitLineage| {
        let mut history = restore_history().to_vec();
        history.push(commit);
        validate_entity_allocation_nodes(&nodes, history.iter().map(CommitLineage::view))
    };
    validate_with(selection(6)).expect("the restored entity binds its canonical node");
    let forged = validate_with(selection(8)).expect_err("a forged binding refuses");
    assert_eq!(forged.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(forged.detail.contains("may not bind"), "{}", forged.detail);
}
