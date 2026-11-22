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
