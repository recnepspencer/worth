//! Resource custody may change its ledger, never ordered query meaning.
use super::*;
use std::collections::BTreeMap;
use worth_execution::SerialMemoryBudget;
use worth_relational::facade::identity::PartitionId;

fn key(relation: &str, memory: Option<ExecutionMemoryReservation>) -> StructuralWalkKey {
    StructuralWalkKey(Arc::new(Parameters {
        relation: Arc::from(relation),
        entity_kind: KindId::new(1),
        relation_kind: KindId::new(2),
        direction: RelationDirection::Forward,
        roots: vec![EntityId::new(PartitionId::main(), 7, 1)],
        _memory: memory,
    }))
}

#[test]
fn ordered_lookup_and_ticket_lifetime_are_independent_of_ledger_mutation() {
    let budget = SerialMemoryBudget::new(Some(256));
    let held = key("links", Some(budget.reserve(64).unwrap()));
    let lookup = key("links", None);
    let mut records = BTreeMap::from([(held, "selected")]);
    assert_eq!(budget.available_memory_bytes(), Some(192));
    assert_eq!(records.get(&lookup), Some(&"selected"));

    let competing = budget.reserve(128).unwrap();
    assert_eq!(budget.available_memory_bytes(), Some(64));
    assert_eq!(records.get(&lookup), Some(&"selected"));
    assert_eq!(records.insert(lookup.clone(), "replaced"), Some("selected"));
    assert_eq!(records.len(), 1);
    assert!(!records.contains_key(&key("other-links", None)));
    // Replacement of an equal key retains the original key and its custody.
    assert_eq!(budget.available_memory_bytes(), Some(64));
    drop(competing);
    assert_eq!(budget.available_memory_bytes(), Some(192));
    assert_eq!(records.remove(&lookup), Some("replaced"));
    assert_eq!(budget.available_memory_bytes(), Some(256));
}
