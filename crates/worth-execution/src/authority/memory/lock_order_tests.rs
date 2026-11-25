use super::*;

#[test]
fn serial_lock_order_follows_issue_order_when_addresses_reverse_it() {
    let mut a = SerialMemoryBudget::new(Some(10));
    let mut b = SerialMemoryBudget::new(Some(10));
    let first = LedgerIssueOrder::issue();
    let second = LedgerIssueOrder::issue();
    // Allocation and issue can interleave on different threads. Assign fresh
    // owner-issued positions before exposing these private fixture ledgers,
    // ensuring the first issued ledger has the higher address on every run.
    let a_is_higher = Arc::as_ptr(&a.ledgers[0]) > Arc::as_ptr(&b.ledgers[0]);
    let ledger_a = &mut Arc::get_mut(&mut a.ledgers).unwrap()[0];
    Arc::get_mut(ledger_a).unwrap().issue_order = if a_is_higher {
        first.clone()
    } else {
        second.clone()
    };
    let ledger_b = &mut Arc::get_mut(&mut b.ledgers).unwrap()[0];
    Arc::get_mut(ledger_b).unwrap().issue_order = if a_is_higher { second } else { first.clone() };
    let joined = a.within_parent(&b);
    assert_eq!(joined.lock_order()[0].1.issue_order, first);
    let opposite = b.within_parent(&a);
    assert_eq!(opposite.lock_order()[0].1.issue_order, first);
}

#[test]
fn rebind_between_overlapping_lineages_counts_held_bytes_on_shared_parent() {
    let parent = SerialMemoryBudget::new(Some(1000));
    let _parent_hold = parent.reserve(600).unwrap();
    let child_a = SerialMemoryBudget::new(Some(500)).within_parent(&parent);
    let child_b = SerialMemoryBudget::new(Some(450)).within_parent(&parent);
    child_a.resize(0, 300).unwrap();
    assert_eq!(parent.charged_memory_bytes(), 900);
    assert_eq!(child_a.charged_memory_bytes(), 300);

    // Refusal on child_b (500 > 450) leaves every ledger untouched.
    assert!(SerialMemoryBudget::rebind_between(Some(&child_a), Some(&child_b), 300, 500).is_err());
    assert_eq!(parent.charged_memory_bytes(), 900);
    assert_eq!(child_a.charged_memory_bytes(), 300);
    assert_eq!(child_b.charged_memory_bytes(), 0);

    // Rebinding 300 -> 350 counts held 300 as room on shared parent (900 - 300 + 350 = 950 <= 1000).
    SerialMemoryBudget::rebind_between(Some(&child_a), Some(&child_b), 300, 350).unwrap();
    assert_eq!(parent.charged_memory_bytes(), 950);
    assert_eq!(child_a.charged_memory_bytes(), 0);
    assert_eq!(child_b.charged_memory_bytes(), 350);

    SerialMemoryBudget::rebind_between(Some(&child_b), None, 350, 0).unwrap();
    assert_eq!(parent.charged_memory_bytes(), 600);
    assert_eq!(child_b.charged_memory_bytes(), 0);
}
