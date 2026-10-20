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
