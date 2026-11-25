use super::*;

#[test]
fn pin_lease_expiry_preserves_protection_until_loans_and_children_drain() {
    let mut table = PinTableKernel::new();
    table.acquire_pin(1, 100, None).expect("parent pin");
    table.acquire_reader_loan(1).expect("reader loan");
    table.acquire_pin(2, 200, Some(1)).expect("child pin");

    assert!(table.is_root_protected(100));
    assert!(table.is_root_protected(200));

    assert_eq!(table.expire_lease(1), Ok(PinLifecycleState::Expiring));
    assert_eq!(
        table.acquire_reader_loan(1),
        Err(PinTransitionError::LeaseNotActive)
    );
    assert_eq!(table.expire_lease(1), Ok(PinLifecycleState::Expiring));
    assert!(table.is_root_protected(100));

    table.release_reader_loan(1).expect("release loan");
    assert_eq!(table.expire_lease(1), Ok(PinLifecycleState::Expiring));
    assert!(table.is_root_protected(100));

    assert_eq!(table.expire_lease(2), Ok(PinLifecycleState::Expiring));
    assert_eq!(table.expire_lease(2), Ok(PinLifecycleState::Expired));
    assert!(!table.is_root_protected(200));
    assert_eq!(table.expire_lease(1), Ok(PinLifecycleState::Expired));
    assert!(!table.is_root_protected(100));
}
