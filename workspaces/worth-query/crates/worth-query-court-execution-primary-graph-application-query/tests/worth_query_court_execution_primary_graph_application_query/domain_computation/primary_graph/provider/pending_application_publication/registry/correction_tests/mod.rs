//! The correction index follows the existing reserved slot through cross-branch refusal and unwind.
use std::sync::Arc;
use std::sync::Mutex;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::pending_application_publication::registry::WorthQueryPendingApplicationIdempotency;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::pending_application_publication::registry::WorthQueryPendingApplicationPublicationRegistry;

#[test]
fn reserved_command_is_visible_across_branches_and_released_with_its_slot() {
    let product_incarnation = || {
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system())
        .application
        .current_world()
        .occurrence_for_test()
    };
    let first = product_incarnation();
    let second = product_incarnation();
    let binding = WorthQueryApplicationIdempotencyBinding::new_for_test([0xD1; 32], [0xD1; 32]);
    let registry = Arc::new(Mutex::new(
        WorthQueryPendingApplicationPublicationRegistry::new_for_test(2),
    ));
    let key = [0xD3; 32];
    let binding = binding.bind_correction_command(&key, &[0xD4; 32]);
    let reservation = WorthQueryPendingApplicationPublicationRegistry::reserve_for_test(
        &registry,
        first,
        Some(binding),
    )
    .unwrap();
    let slot = registry
        .lock()
        .unwrap()
        .correction_slot_for_test(&key)
        .expect("Reserved already owns the command key");
    assert!(Arc::ptr_eq(&slot, reservation.slot_for_test()));
    assert!(
        matches!(slot.inspect_idempotency_for_test(), Some(WorthQueryPendingApplicationIdempotency::Pending(recorded)) if recorded == binding)
    );
    assert!(
        WorthQueryPendingApplicationPublicationRegistry::reserve_for_test(
            &registry,
            second,
            Some(binding)
        )
        .is_err(),
        "a second target cannot reserve the same command while the original slot owns it"
    );
    drop(reservation);
    assert!(registry
        .lock()
        .unwrap()
        .correction_slot_for_test(&key)
        .is_none());
    let reservation = WorthQueryPendingApplicationPublicationRegistry::reserve_for_test(
        &registry,
        second,
        Some(binding),
    )
    .expect("unwind released both indexes");
    assert!(Arc::ptr_eq(
        &registry
            .lock()
            .unwrap()
            .correction_slot_for_test(&key)
            .unwrap(),
        reservation.slot_for_test()
    ));
    drop(reservation);
    let registry = registry.lock().unwrap();
    assert!(registry.slots.is_empty());
    assert!((*registry.corrections_for_test()).is_empty());
}
