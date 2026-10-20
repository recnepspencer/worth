//! Absent host policy leaves the same counted backing grants alive.
use super::*;
use std::sync::atomic::Ordering;

#[test]
fn ordinary_registry_large_backing_preserves_counted_custody_and_final_refund() {
    let state = DemandRegistryState::default();
    let records = 65 * 1024 * 1024;
    let required = 5 * 1024 * 1024;
    let record_ticket = state.reserve_record_capacity(records).unwrap();
    let required_ticket = state.reserve_required_custody_capacity(required).unwrap();
    let mut record_backing = Vec::<u8>::new();
    record_backing.try_reserve_exact(records).unwrap();
    record_backing.resize(records, 4);
    let mut required_backing = Vec::<u8>::new();
    required_backing.try_reserve_exact(required).unwrap();
    required_backing.resize(required, 5);
    let original = Arc::new((
        record_backing,
        required_backing,
        record_ticket,
        required_ticket,
    ));
    let clone = Arc::clone(&original);
    drop(original);
    assert_eq!(state.record_retained_bytes.load(Ordering::Acquire), records);
    assert_eq!(
        state
            .required_custody_retained_bytes
            .load(Ordering::Acquire),
        required
    );
    assert_eq!(clone.0[0], 4);
    assert_eq!(clone.1[0], 5);
    drop(clone);
    assert_eq!(state.record_retained_bytes.load(Ordering::Acquire), 0);
    assert_eq!(
        state
            .required_custody_retained_bytes
            .load(Ordering::Acquire),
        0
    );
    assert_eq!(state.obligation_reserved_bytes, 0);
}
