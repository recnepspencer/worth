//! The compiled consumption arm refuses a second arm and clears on drop and unwind.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryRecoveredProgramSourceDeliveryDenialForTest as ControlDenial;

#[test]
fn fixed_missing_carrier_guard_unused_conflict_drop_and_unwind() {
    let world = installation::install(&crate::installed_schema());
    let runtime = world.application.runtime();
    fn require_send_sync<T: Send + Sync>(_: &T) {}
    require_send_sync(runtime);
    let guard = runtime
        .consume_next_recovered_program_source_delivery_for_test()
        .unwrap();
    assert!(guard.captured_delivery_for_test().is_none());
    assert_eq!(
        runtime
            .consume_next_recovered_program_source_delivery_for_test()
            .err()
            .unwrap(),
        ControlDenial::AlreadyArmed
    );
    drop(guard);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let guard = runtime
            .consume_next_recovered_program_source_delivery_for_test()
            .unwrap();
        assert!(guard.captured_delivery_for_test().is_none());
        panic!("unwind the actual owned arm");
    }));
    assert!(outcome.is_err());
    let guard = runtime
        .consume_next_recovered_program_source_delivery_for_test()
        .unwrap();
    assert!(guard.captured_delivery_for_test().is_none());
    drop(guard);
}
