//! The correction index follows the existing reserved slot through cross-branch refusal and unwind.
use super::*;

#[test]
fn reserved_command_is_visible_across_branches_and_released_with_its_slot() {
    let product_incarnation = || {
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        )
        .application
        .current_world()
        .occurrence()
    };
    let first = product_incarnation();
    let second = product_incarnation();
    let binding = WorthQueryApplicationIdempotencyBinding::new([0xD1; 32], [0xD1; 32]);
    let registry = Arc::new(Mutex::new(
        WorthQueryPendingApplicationPublicationRegistry::new(2),
    ));
    let key = [0xD3; 32];
    let binding = binding.bind_correction_command(&key, &[0xD4; 32]);
    let reservation =
        WorthQueryPendingApplicationPublicationRegistry::reserve(&registry, first, Some(binding))
            .unwrap();
    let slot = registry
        .lock()
        .unwrap()
        .correction_slot(&key)
        .expect("Reserved already owns the command key");
    assert!(Arc::ptr_eq(&slot, &reservation.slot));
    assert!(
        matches!(slot.inspect_idempotency(), Some(WorthQueryPendingApplicationIdempotency::Pending(recorded)) if recorded == binding)
    );
    assert!(
        WorthQueryPendingApplicationPublicationRegistry::reserve(&registry, second, Some(binding))
            .is_err(),
        "a second target cannot reserve the same command while the original slot owns it"
    );
    drop(reservation);
    assert!(registry.lock().unwrap().correction_slot(&key).is_none());
    let reservation =
        WorthQueryPendingApplicationPublicationRegistry::reserve(&registry, second, Some(binding))
            .expect("unwind released both indexes");
    assert!(Arc::ptr_eq(
        &registry.lock().unwrap().correction_slot(&key).unwrap(),
        &reservation.slot
    ));
    drop(reservation);
    let registry = registry.lock().unwrap();
    assert!(registry.slots.is_empty());
    assert!(registry.corrections.is_empty());
}
