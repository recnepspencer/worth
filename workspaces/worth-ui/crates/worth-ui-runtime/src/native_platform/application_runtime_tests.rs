use super::{UiNativeApplicationReadinessOwnerCount, UiNativeApplicationReadinessOwnerCountDenial};

#[test]
fn public_runtime_owner_count_preserves_five_slots_beneath_host_internal_capacity() {
    assert_eq!(UiNativeApplicationReadinessOwnerCount::none().get(), 0);
    assert_eq!(
        UiNativeApplicationReadinessOwnerCount::new(5)
            .expect("five application readiness owners fit")
            .get(),
        5
    );
    assert_eq!(
        UiNativeApplicationReadinessOwnerCount::new(6),
        Err(UiNativeApplicationReadinessOwnerCountDenial::CapacityExceeded)
    );
    assert_eq!(
        worth_ui_host_native::UiNativeApplicationReadinessOwnerCount::MAXIMUM,
        6
    );
}
