use super::*;

#[test]
fn exact_cleanup_restores_bounded_capacity_for_another_world_issued_reservation() {
    let (first_product, first_binding, first_handle) =
        crate::domain_computation::primary_graph::tests::application_attempt::world_issued_unpublished_material(0xE1);
    let (second_product, second_binding, second_handle) =
        crate::domain_computation::primary_graph::tests::application_attempt::world_issued_unpublished_material(0xE2);
    let first_affinity = WorthQueryProductIdempotencyAffinity::from_observation(&first_product);
    let second_affinity = WorthQueryProductIdempotencyAffinity::from_observation(&second_product);
    let mut store = WorthQueryUnpublishedIdempotencyStore::new(1);

    let first_key = store
        .reserve(first_affinity, first_binding, first_handle.clone())
        .expect("one real unpublished reservation fits");
    assert!(
        store
            .reserve(
                second_affinity.clone(),
                second_binding,
                second_handle.clone(),
            )
            .is_err(),
        "the configured bound denies before a second reservation is installed",
    );
    assert_eq!(store.retained_count(), 1);

    store.release_exact(&first_key, &first_handle);
    store
        .reserve(second_affinity, second_binding, second_handle)
        .expect("exact cleanup restores the one available slot");
    assert_eq!(store.retained_count(), 1);
}
