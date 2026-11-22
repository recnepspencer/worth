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
        restore.subject = subject;
        restore.targets = vec![LineageId(4)];
        let detail = validate(&history).expect_err("a reused restored lineage refuses");
        assert!(detail.contains("lineage id 4 is reused"), "{detail}");
    }
}

#[test]
fn restores_bound_to_the_wrong_entity_node_or_subject_refuse() {
    let mut history = restore_history();
    history[1].allocating(2).targets = vec![LineageId(30)];
    history[1].allocating(3).targets = vec![LineageId(2)];
    history[1].allocating(30).targets = vec![LineageId(3)];
    let detail = validate(&history).expect_err("swapped allocations bind the wrong entities");
    assert!(
        detail.contains("does not name its exact entity"),
        "{detail}"
    );

    let mut history = restore_history();
    history[2].allocating(5).targets = vec![LineageId(50)];
    let detail = validate(&history).expect_err("a restore with no canonical node refuses");
    assert!(
        detail.contains("lineage id 50 has no canonical node"),
        "{detail}"
    );

    let mut history = restore_history();
    history[2].allocating(5).subject = LineageSubject::Relation;
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
        history[2].allocating(5).sources = sources;
        let detail = validate(&history).expect_err("malformed restore refuses");
        assert!(
            detail.contains("one source and one different target"),
            "{detail}"
        );
    }
}
