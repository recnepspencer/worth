//! The entity-allocation binding that complete checkpoint capture publishes
//! lineage nodes from binds `Restore` allocations exactly like `Create`.

use super::lineage_allocation::{entity_allocation_bindings, LineageAllocationRefusal};
use super::lineage_allocation_fixtures::{entity, receipt, restore_history};
use crate::identity::data::LineageId;
use crate::lineage::data::LineageSubject;

#[test]
fn restore_only_and_mixed_commits_bind_their_created_entities_in_ascending_order() {
    let [first, mixed, restore_only] = restore_history();
    assert_eq!(
        entity_allocation_bindings(&first.view()),
        Ok(vec![(LineageId(1), entity(1))])
    );
    assert_eq!(
        entity_allocation_bindings(&mixed.view()),
        Ok(vec![(LineageId(2), entity(2)), (LineageId(3), entity(3))]),
        "the relation allocation binds no entity; patch order is irrelevant"
    );
    assert_eq!(
        entity_allocation_bindings(&restore_only.view()),
        Ok(vec![(LineageId(5), entity(5))])
    );
}

#[test]
fn relation_restores_malformed_restores_and_foreign_commits_refuse() {
    let [_, mixed, mut restore_only] = restore_history();
    restore_only.allocating(5).subject = LineageSubject::Relation;
    assert_eq!(
        entity_allocation_bindings(&restore_only.view()),
        Err(LineageAllocationRefusal::EntityAllocationCountMismatch {
            allocations: 0,
            created_entities: 1,
        })
    );
    for sources in [vec![], vec![LineageId(1), LineageId(2)], vec![LineageId(3)]] {
        let mut malformed = mixed.clone();
        let restore = malformed.allocating(3);
        restore.sources = sources;
        let event_id = restore.event_id();
        assert_eq!(
            entity_allocation_bindings(&malformed.view()),
            Err(LineageAllocationRefusal::MalformedRestore { event_id })
        );
    }
    let mut foreign = mixed.clone();
    let restore = foreign.allocating(3);
    restore.commit = receipt(9);
    let event_id = restore.event_id();
    assert_eq!(
        entity_allocation_bindings(&foreign.view()),
        Err(LineageAllocationRefusal::ForeignCommit { event_id })
    );
}

/// A selection restore commit: it created nothing, rematerialized a named
/// `e6` (restored from lineage 5 onto lineage 6, naming `e6`) and an unnamed
/// `e7` (no lineage event).
fn selection_restore() -> super::lineage_allocation_fixtures::CommitLineage {
    let mut commit = super::lineage_allocation_fixtures::CommitLineage::new(4, Vec::new());
    commit.rematerialized_entities = vec![entity(7), entity(6)];
    commit.push(
        crate::lineage::data::LineageEventKind::Restore,
        LineageSubject::Entity,
        &[5],
        &[6],
    );
    commit.allocating(6).entity_id = Some(entity(6));
    commit
}

#[test]
fn explicit_bindings_anchor_to_rematerialized_entities_beside_unnamed_ones() {
    assert_eq!(
        entity_allocation_bindings(&selection_restore().view()),
        Ok(vec![(LineageId(6), entity(6))])
    );
}

#[test]
fn forged_duplicate_created_relation_and_non_allocating_bindings_refuse() {
    use crate::lineage::data::LineageEventKind::{Replace, Restore, Retire};
    let malformed = |commit: &super::lineage_allocation_fixtures::CommitLineage, event_id| {
        assert_eq!(
            entity_allocation_bindings(&commit.view()),
            Err(LineageAllocationRefusal::MalformedEntityBinding { event_id })
        );
    };
    // (a) an entity the commit never rematerialized.
    let mut forged = selection_restore();
    forged.allocating(6).entity_id = Some(entity(8));
    malformed(&forged, 40);
    // (b) the same entity bound twice in one commit.
    let mut duplicate = selection_restore();
    duplicate.push(Restore, LineageSubject::Entity, &[4], &[9]);
    duplicate.allocating(9).entity_id = Some(entity(6));
    malformed(&duplicate, 41);
    // (c) an entity the commit created, never a rematerialized one.
    let mut created = selection_restore();
    created.created_entities = vec![entity(6)];
    created.push(
        crate::lineage::data::LineageEventKind::Create,
        LineageSubject::Entity,
        &[],
        &[10],
    );
    malformed(&created, 40);
    // (d) a relation allocation.
    let mut relation = selection_restore();
    relation.allocating(6).subject = LineageSubject::Relation;
    malformed(&relation, 40);
    // (e) non-allocating events.
    for kind in [Retire, Replace] {
        let mut ending = selection_restore();
        let targets: &[u64] = if kind == Retire { &[] } else { &[11] };
        ending.push(kind, LineageSubject::Entity, &[6], targets);
        ending.events[1].entity_id = Some(entity(7));
        malformed(&ending, 41);
    }
}
