//! Zero-byte inline entries must not retain unowned memory ledger nodes.
use super::*;
use crate::tests::request;

#[test]
fn busy_zero_byte_entry_survives_the_charged_entry() {
    let lease = ExecutionResourceLease::serial(request(1, 32, 20)).unwrap();
    let charged = lease.reserve_entry(8).unwrap();
    let inline = lease.reserve_entry(0).unwrap();
    assert_eq!(lease.charged_memory_bytes(), 8);
    drop(charged);
    assert_eq!(lease.charged_memory_bytes(), 0);
    drop(inline);
    let replacement = lease.reserve_entry(32).unwrap();
    assert_eq!(lease.charged_memory_bytes(), 32);
    drop(replacement);
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn busy_zero_byte_entry_can_rebind_after_the_charged_entry_leaves() {
    let lease = ExecutionResourceLease::serial(request(1, 32, 20)).unwrap();
    let charged = lease.reserve_entry(8).unwrap();
    let mut inline = lease.reserve_entry(0).unwrap();
    drop(charged);
    lease.rebind_retained_memory(&mut inline, 16).unwrap();
    assert_eq!(lease.charged_memory_bytes(), 16);
    drop(inline);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
