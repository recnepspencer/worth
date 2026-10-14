//! The compiled consumption arm owns its receiver, actual capture and synchronous scope.
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

#[test]
fn fixed_missing_carrier_guard_receiver_isolation_and_one_shot() {
    let foreign = crate::installed_schema();
    let world = installation::install(&foreign);
    let other = installation::install(&foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let other_adapter = authentication::admit(other.application.installed_schema());
    let other_principal = authentication::block_on(other_adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let other_request = other.application.request(&other_principal, &scope);
    let guard = world
        .application
        .runtime()
        .consume_next_recovered_program_source_delivery_for_test()
        .unwrap();
    let other_guard = other
        .application
        .runtime()
        .consume_next_recovered_program_source_delivery_for_test()
        .unwrap();
    assert_eq!(
        other
            .application
            .runtime()
            .consume_next_recovered_program_source_delivery_for_test()
            .err()
            .unwrap(),
        ControlDenial::AlreadyArmed
    );
    let intent = input(2);
    reset_planar_source_adjustment_contacts(&intent);
    let other_operation = recover(
        partial(&other, &other_request, intent.clone(), 10_174),
        &other_request,
    );
    assert!(
        guard.captured_delivery_for_test().is_none(),
        "another real receiver cannot consume this arm"
    );
    assert!(other_guard
        .captured_delivery_for_test()
        .unwrap()
        .has_original_delivery());
    let other_terminal = match other_operation.advance(&other_request) {
        Advance::Terminal {
            operation,
            denial: Denial::RecoveryCarrierUnavailable,
        } => operation,
        _ => panic!("the independently armed receiver receives its exhausted delivery"),
    };
    other_terminal
        .close(&other_request)
        .unwrap_or_else(|(_, cause)| panic!("independent receiver cleanup: {cause:?}"));
    drop(other_guard);
    let operation = recover(partial(&world, &request, intent.clone(), 10_174), &request);
    assert!(guard
        .captured_delivery_for_test()
        .unwrap()
        .has_original_delivery());
    let terminal = match operation.advance(&request) {
        Advance::Terminal {
            operation,
            denial: Denial::RecoveryCarrierUnavailable,
        } => operation,
        _ => panic!("the matching receiver alone receives exhausted delivery"),
    };
    terminal
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("native pending obligations close: {cause:?}"));
    assert_eq!(
        planar_source_adjustment_contacts(&intent),
        (2, 2),
        "two authentic runtimes each executed the same real input once"
    );
    let first_capture = guard.captured_delivery_for_test().unwrap();
    let next = input(3);
    reset_planar_source_adjustment_contacts(&next);
    let operation = recover(partial(&world, &request, next.clone(), 10_175), &request);
    settle(operation, &request)
        .close(&request)
        .unwrap_or_else(|(_, cause)| {
            panic!("a consumed arm does not consume another source: {cause:?}")
        });
    assert_eq!(planar_source_adjustment_contacts(&next), (1, 1));
    assert!(first_capture.has_original_delivery());
    assert!(
        guard
            .captured_delivery_for_test()
            .unwrap()
            .has_original_delivery(),
        "the first captured owner was not overwritten"
    );
    drop(guard);
    let unused = world
        .application
        .runtime()
        .consume_next_recovered_program_source_delivery_for_test()
        .unwrap();
    assert!(unused.captured_delivery_for_test().is_none());
    drop(unused);
    let last = input(4);
    reset_planar_source_adjustment_contacts(&last);
    let captured = world
        .application
        .runtime()
        .consume_next_recovered_program_source_delivery_for_test()
        .unwrap();
    let operation = recover(partial(&world, &request, last.clone(), 10_176), &request);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let captured = captured;
        assert!(captured
            .captured_delivery_for_test()
            .unwrap()
            .has_original_delivery());
        panic!("unwind releases an actual captured native carrier");
    }));
    assert!(outcome.is_err());
    let unused = world
        .application
        .runtime()
        .consume_next_recovered_program_source_delivery_for_test()
        .unwrap();
    drop(unused);
    let terminal = match operation.advance(&request) {
        Advance::Terminal {
            operation,
            denial: Denial::RecoveryCarrierUnavailable,
        } => operation,
        _ => panic!("unwind cannot reconstruct the already consumed original parts"),
    };
    assert!(terminal.recovered_progress().is_some());
    assert_eq!(
        world
            .application
            .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .len(),
        1,
        "guard unwind did not discard pending native cleanup"
    );
    terminal
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("surviving pending cleanup closes: {cause:?}"));
    assert_eq!(planar_source_adjustment_contacts(&last), (1, 1));
}
