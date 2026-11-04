//! Canonical publication must retain genuine text custody across dictionary updates.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn repeated_canonical_import_keeps_the_original_managed_text_grant() {
    let budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    let text = StoredSymbolText::copy_from_str(&request, "original actor").unwrap();
    let snapshot = SymbolTableSnapshot {
        entries: vec![(Symbol(1), text)],
    };
    let table = RuntimeSymbolTable::default();
    table.import_canonical_snapshot(&snapshot).unwrap();
    let retained = budget.charged_memory_bytes();
    assert!(retained > 0);
    table.import_canonical_snapshot(&snapshot).unwrap();
    drop(snapshot);
    assert_eq!(budget.charged_memory_bytes(), retained);
    assert_eq!(table.resolve(Symbol(1)), Some("original actor".to_owned()));
    drop(table);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn canonical_import_replaces_the_ticket_when_another_request_funds_the_same_text() {
    let first_budget = SerialMemoryBudget::new(Some(65_536));
    let next_budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    let first = SerialRequest::from_memory(first_budget.clone(), cancellation.token(), None);
    let next = SerialRequest::from_memory(next_budget.clone(), cancellation.token(), None);
    let table = RuntimeSymbolTable::default();
    let original = SymbolTableSnapshot {
        entries: vec![(
            Symbol(1),
            StoredSymbolText::copy_from_str(&ExecutionRequest::serial(&first), "same actor")
                .unwrap(),
        )],
    };
    table.import_canonical_snapshot(&original).unwrap();
    drop(original);
    assert!(first_budget.charged_memory_bytes() > 0);
    let replacement = SymbolTableSnapshot {
        entries: vec![(
            Symbol(1),
            StoredSymbolText::copy_from_str(&ExecutionRequest::serial(&next), "same actor")
                .unwrap(),
        )],
    };
    table.import_canonical_snapshot(&replacement).unwrap();
    drop(replacement);
    assert_eq!(first_budget.charged_memory_bytes(), 0);
    assert!(next_budget.charged_memory_bytes() > 0);
    assert_eq!(table.resolve(Symbol(1)), Some("same actor".to_owned()));
    drop(table);
    assert_eq!(next_budget.charged_memory_bytes(), 0);
}

#[test]
fn canonical_import_upgrades_a_system_configuration_key_to_the_admitted_text() {
    let table = RuntimeSymbolTable::default();
    table
        .import_canonical_snapshot(&SymbolTableSnapshot {
            entries: vec![(Symbol(1), "actor".into())],
        })
        .unwrap();
    let budget = SerialMemoryBudget::new(Some(65_536));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let admitted = SymbolTableSnapshot {
        entries: vec![(
            Symbol(1),
            StoredSymbolText::copy_from_str(&ExecutionRequest::serial(&host), "actor").unwrap(),
        )],
    };
    table.import_canonical_snapshot(&admitted).unwrap();
    drop(admitted);
    assert!(table.read_state(|state| state
        .configuration
        .get_key_value("actor")
        .unwrap()
        .0
        .is_managed()));
    assert!(budget.charged_memory_bytes() > 0);
    drop(table);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
