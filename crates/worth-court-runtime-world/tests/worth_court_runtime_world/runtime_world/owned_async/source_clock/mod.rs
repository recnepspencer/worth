// Named coordinates advance the exact resource cell, before separate admission.
use std::num::NonZeroU64;
use worth_runtime_world::facade::RuntimeWorldOwnedAsyncRequestAdmissionDenial;
use {
    crate::owned_async::ordinary_context::fixture::admit_managed,
    crate::owned_async::ordinary_context::fixture::selected_fork,
    crate::owned_async::ordinary_context::fixture::AsyncSourceCourt,
};

#[test]
fn source_clock_rejects_foreign_lease_and_retains_accepted_clock_when_request_admission_refuses() {
    let court = crate::owned_async::ordinary_context::fixture::AsyncSourceCourt::process_local(
        std::num::NonZeroU64::MIN,
    );
    let selected = selected_fork(&court);
    let foreign = crate::owned_async::ordinary_context::fixture::AsyncSourceCourt::process_local(
        std::num::NonZeroU64::MIN,
    );
    let foreign_selected = selected_fork(&foreign);
    assert!(matches!(
        court.world.advance_owned_async_source_clock(
            &court.bridge,
            &court.declaration,
            &selected,
            &foreign.lease(&foreign_selected),
            100
        ),
        Err(worth_runtime_world::facade::RuntimeWorldOwnedAsyncRequestAdmissionDenial::RelationalSourceMismatch)
    ));

    let admitted_clock = court
        .world
        .advance_owned_async_source_clock(
            &court.bridge,
            &court.declaration,
            &selected,
            &court.lease(&selected),
            10,
        )
        .unwrap();
    assert_eq!(admitted_clock.promoted_wake_count(), 0);
    let mut original = admit_managed(&court, &selected);
    assert_eq!(
        original
            .request()
            .unwrap()
            .request()
            .in_flight_identity()
            .in_flight()
            .timeout_due_tick()
            .unwrap()
            .get(),
        15,
        "actual resource timeout uses its own cell's clock"
    );
    let progressed = court
        .world
        .advance_owned_async_source_clock(
            &court.bridge,
            &court.declaration,
            &selected,
            &court.lease(&selected),
            20,
        )
        .unwrap();
    assert_eq!(progressed.validated_advance().next_tick().get(), 20);
    assert_eq!(progressed.promoted_wake_count(), 0);
    assert!(progressed.ready_selection_deferred());
    assert!(
        court
            .world
            .admit_managed_owned_async_request(
                &court.bridge,
                &court.declaration,
                &selected,
                &court.lease(&selected)
            )
            .is_err(),
        "clock admission is not a free request slot"
    );
    let timeout = court
        .bridge
        .advance_managed_owned_async_request_to_timeout(&mut original, 20)
        .unwrap();
    assert!(timeout.report().timed_out_request().is_some());
    if original.wake_cleanup_pending() {
        court
            .bridge
            .finish_managed_owned_async_wake_cleanup(&mut original)
            .unwrap();
    }
    court
        .bridge
        .release_process_local_managed_request(original, None)
        .unwrap_or_else(|cause| panic!("actual timed-out owner release: {:?}", cause.cause()));
    let next = admit_managed(&court, &selected);
    assert_eq!(
        next.request()
            .unwrap()
            .request()
            .in_flight_identity()
            .in_flight()
            .timeout_due_tick()
            .unwrap()
            .get(),
        25,
        "a later admission cannot roll accepted clock progress back"
    );
    court
        .bridge
        .cancel_managed_owned_async_request(&next)
        .unwrap();
    court
        .bridge
        .release_process_local_managed_request(next, None)
        .unwrap_or_else(|cause| panic!("actual successor release: {:?}", cause.cause()));
}
