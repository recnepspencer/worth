//! Output settlement delivers from the same request that ran its producer.
use super::*;

worth_execution::needs_os_threads! {
/// Master's `a2a9ce5d99` removed the per-computation work ceiling. Settlement
/// runs the producer, refreshes over its own write and delivers, with no
/// caller work bound.
#[test]
fn delivery_settles_from_the_same_request_that_ran_its_producer() {
    let _guard = checkpoint_recovery_test_guard();
    let _restore = Restore(place(Placement::Serial), bound(None));
    place(Placement::Leased(NonZeroUsize::new(4).unwrap()));
    let application = application_fixture::application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut interest = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram<false, PRODUCER_WORK, RUNS>, RegionConnection>(
            &application,
        )
        .unwrap();
    arm_own_write(Some(OwnWrite {
        number: 0,
        bits: 2.5_f64.to_bits(),
        read: OwnWriteRead::Blind,
    }));
    reports();
    room().clear();
    interest
        .settle(&request)
        .expect("settlement delivers with no caller work bound");
    // The entry changes once (1.0 -> 2.5): the existing oracle model requires
    // an original decision and one refreshed decision, not an unbounded retry.
    assert_eq!(room().len(), 2);
    assert!(room().iter().all(|run| run.outcome.is_ok()));
    let requests = reports();
    assert_eq!(requests.len(), 1, "producer and delivery share one request");
    room().clear();
    interest
        .settle(&request)
        .expect("completed interest remains settled");
    assert!(
        room().is_empty(),
        "settlement cannot duplicate the producer effect"
    );
}
}
