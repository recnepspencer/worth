use super::*;

#[test]
fn exact_cleanup_restores_bounded_capacity_for_another_world_issued_reservation() {
    {
        let (first_product, first_binding, first_handle) =
        crate::domain_computation::primary_graph::tests::application_attempt::world_issued_unpublished_material(0xE1);
        let (second_product, second_binding, second_handle) =
        crate::domain_computation::primary_graph::tests::application_attempt::world_issued_unpublished_material(0xE2);
        let first_affinity = WorthQueryProductIdempotencyAffinity::from_observation(&first_product);
        let second_affinity =
            WorthQueryProductIdempotencyAffinity::from_observation(&second_product);
        let mut store = WorthQueryUnpublishedIdempotencyStore::new(Some(1));

        let (first_key, _) = store
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
}

#[test]
fn ordinary_store_keeps_80_world_issued_reservations_until_exact_cleanup() {
    let mut store = WorthQueryUnpublishedIdempotencyStore::new(None);
    let mut originals = Vec::new();
    for ordinal in 0..80 {
        let (product, binding, handle) = crate::domain_computation::primary_graph::tests::application_attempt::world_issued_unpublished_material(ordinal);
        let affinity = WorthQueryProductIdempotencyAffinity::from_observation(&product);
        let (key, _) = store.reserve(affinity, binding, handle.clone()).unwrap();
        originals.push((product, key, handle));
    }
    assert_eq!(store.retained_count(), originals.len());
    for (product, key, handle) in originals {
        store.release_exact(&key, &handle);
        drop((product, handle));
    }
    assert_eq!(store.retained_count(), 0);
}
