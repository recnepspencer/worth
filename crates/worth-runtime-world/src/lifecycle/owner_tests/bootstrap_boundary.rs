use super::*;
use crate::branch::reference_test_fixture::{real_fixture, RealReferenceFixture};
use crate::facade::{RuntimeWorldCancellationSource, RuntimeWorldOwner};
fn empty_world() -> (RealReferenceFixture, RuntimeWorldOwner<(), (), (), (), ()>) {
    let mut fixture = real_fixture(8, 8);
    let (relational, signal, signal_publication, bridge, budgets, clock) = fixture
        .owner_inputs(
            bootstrap_budgets(),
            RuntimeWorldClock::from_source(FixedClock),
        )
        .into_parts();
    let owner = RuntimeWorldOwner::builder()
        .with_relational_services(relational)
        .with_signal_services(signal)
        .with_signal_definition_publication(signal_publication)
        .with_bridge_correspondence(bridge)
        .with_budgets(budgets)
        .with_clock(clock)
        .build()
        .unwrap();
    (fixture, owner)
}
#[test]
fn cancelled_bootstrap_is_no_contact_and_inspection_waits_for_complete_root() {
    let (fixture, owner) = empty_world();
    assert!(owner.inspection_port().history_snapshot().is_err());
    let cancellation = RuntimeWorldCancellationSource::new();
    cancellation.cancel();
    let result = owner
        .lifecycle_port()
        .bootstrap_root(
            fixture
                .bootstrap_intent()
                .with_cancellation(cancellation.token()),
        )
        .unwrap();
    assert!(
        matches!(result,RuntimeWorldBootstrapOutcome::NoEffect(value) if value.cause()==RuntimeWorldBootstrapNoEffectCause::Cancelled)
    );
    assert_eq!(
        owner
            .root
            .state
            .retention
            .cost_snapshot()
            .owner_acquisition_contacts(),
        0
    );
    assert_eq!(owner.root.state.history.len(), 0);
    assert!(matches!(
        owner
            .lifecycle_port()
            .bootstrap_root(fixture.bootstrap_intent())
            .unwrap(),
        RuntimeWorldBootstrapOutcome::Performed(_)
    ));
    assert_eq!(
        owner
            .inspection_port()
            .history_snapshot()
            .unwrap()
            .installed_commits(),
        1
    );
}
#[test]
fn close_admission_excludes_bootstrap_before_component_acquisition() {
    let (fixture, owner) = empty_world();
    let engine = owner.root.clone();
    let ledger = engine.state.operation.state.lock().unwrap();
    let closer = engine.clone();
    let close = std::thread::spawn(move || closer.close());
    super::admission_race::wait_until_close_is_admitting(&engine);
    let port = owner.lifecycle_port();
    let intent = fixture.bootstrap_intent();
    let bootstrap = std::thread::spawn(move || port.bootstrap_root(intent));
    drop(ledger);
    drop(close.join().unwrap().unwrap());
    match bootstrap.join().unwrap() {
        Err(_) => {}
        Ok(RuntimeWorldBootstrapOutcome::NoEffect(value)) => assert_eq!(
            value.cause(),
            RuntimeWorldBootstrapNoEffectCause::OwnerUnavailable
        ),
        Ok(other) => panic!("closed World admitted bootstrap: {other:?}"),
    }
    assert_eq!(
        engine
            .state
            .retention
            .cost_snapshot()
            .owner_acquisition_contacts(),
        0
    );
    assert_eq!(engine.state.history.len(), 0);
}
#[cfg(feature = "test-operation-control")]
#[test]
fn cancellation_and_owner_loss_during_pin_acquisition_never_expose_a_root() {
    use std::time::Duration;
    use worth_signal::facade::branch::SignalOwnerOperationBoundary;
    for lose_owner in [false, true] {
        let (fixture, owner) = empty_world();
        let mut owner = Some(owner);
        let engine = owner.as_ref().unwrap().root.clone();
        let inspection = owner.as_ref().unwrap().inspection_port();
        let lifecycle = owner.as_ref().unwrap().lifecycle_port();
        let cancellation = RuntimeWorldCancellationSource::new();
        let intent = fixture
            .bootstrap_intent()
            .with_cancellation(cancellation.token());
        let control = fixture.signal_operation_control();
        let admission = control.arm_pause_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
        let worker = std::thread::spawn(move || lifecycle.bootstrap_root(intent));
        assert!(admission.wait_until_reached(Duration::from_secs(5)));
        let pin = control.arm_pause_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
        admission.release();
        assert!(pin.wait_until_reached(Duration::from_secs(5)));
        assert_eq!(
            engine.state.retention.cost_snapshot().relational_contacts(),
            1
        );
        assert!(inspection.history_snapshot().is_err());
        assert_eq!(
            engine.close().unwrap_err(),
            RuntimeWorldCloseDenial::AlreadyClosing
        );
        if lose_owner {
            drop(owner.take());
        } else {
            cancellation.cancel();
        }
        pin.release();
        let expected = if lose_owner {
            RuntimeWorldBootstrapNoEffectCause::OwnerUnavailable
        } else {
            RuntimeWorldBootstrapNoEffectCause::Cancelled
        };
        assert!(
            matches!(worker.join().unwrap().unwrap(),RuntimeWorldBootstrapOutcome::NoEffect(value) if value.cause()==expected)
        );
        assert_eq!(engine.state.history.len(), 0);
        assert_eq!(engine.state.history.reserved_len(), 0);
        assert_eq!(
            engine.state.retention.active_component_obligation_count(),
            0
        );
        assert_eq!(engine.state.branches.branch_count(), 0);
        if let Some(owner) = owner {
            assert!(matches!(
                owner
                    .lifecycle_port()
                    .bootstrap_root(fixture.bootstrap_intent())
                    .unwrap(),
                RuntimeWorldBootstrapOutcome::Performed(_)
            ));
        }
    }
}
