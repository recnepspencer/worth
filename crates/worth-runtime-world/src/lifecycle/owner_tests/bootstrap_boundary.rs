use super::*;
use crate::branch::reference_test_fixture::{real_fixture, RealReferenceFixture};
use crate::facade::{RuntimeWorldCancellationSource, RuntimeWorldOwner};
fn empty_world() -> (RealReferenceFixture, RuntimeWorldOwner<(), (), (), (), ()>) {
    let mut fixture = real_fixture(8, 8);
    let inputs = fixture.owner_inputs(
        bootstrap_budgets(),
        RuntimeWorldClock::from_source(FixedClock),
    );
    let owner = RuntimeWorldOwner::builder()
        .with_execution_policy(inputs.execution.request_policy())
        .with_relational_services(inputs.relational)
        .with_signal_services(inputs.signal)
        .with_signal_definition_publication(inputs.signal_definition_publication)
        .with_bridge_correspondence(inputs.bridge)
        .with_budgets(inputs.budgets)
        .with_clock(inputs.clock)
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
fn close_refuses_busy_admission_then_excludes_bootstrap() {
    let (fixture, owner) = empty_world();
    let engine = owner.root.clone();
    let policy = crate::test_execution_policy::serial_request();
    let request = worth_execution::ExecutionRequest::serial(&policy);
    let ledger = engine.state.operation.state.lock().unwrap();
    assert_eq!(
        engine.close(&request).unwrap_err(),
        RuntimeWorldCloseDenial::Busy
    );
    drop(ledger);
    drop(engine.close(&request).unwrap());
    assert!(owner
        .lifecycle_port()
        .bootstrap_root(fixture.bootstrap_intent())
        .is_err());
}

#[test]
fn checkpoint_origin_retirement_requires_closed_world_and_an_actual_home_binding() {
    use crate::facade::WorldCheckpointOriginRetirementDenial as Denial;
    let (_fixture, owner) = empty_world();
    assert_eq!(
        owner.retire_checkpoint_origin().unwrap_err(),
        Denial::OwnerNotClosed
    );
    let close = owner.root.state.close.lock().unwrap();
    assert_eq!(owner.retire_checkpoint_origin().unwrap_err(), Denial::Busy);
    drop(close);
    let policy = crate::test_execution_policy::serial_request();
    let request = worth_execution::ExecutionRequest::serial(&policy);
    drop(owner.root.close(&request).unwrap());
    assert_eq!(
        owner.retire_checkpoint_origin().unwrap_err(),
        Denial::NotHomeBound
    );
}
#[test]
fn cancellation_and_owner_loss_during_pin_acquisition_never_expose_a_root() {
    use std::time::Duration;
    use worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest;
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
        let admission = control
            .arm_pause_once_for_test(SignalOwnerOperationBoundaryForTest::BranchRegistryLookup);
        let worker = std::thread::spawn(move || lifecycle.bootstrap_root(intent));
        assert!(admission.wait_until_reached(Duration::from_secs(5)));
        let pin = control
            .arm_pause_once_for_test(SignalOwnerOperationBoundaryForTest::BranchRegistryLookup);
        admission.release();
        assert!(pin.wait_until_reached(Duration::from_secs(5)));
        assert_eq!(
            engine.state.retention.cost_snapshot().relational_contacts(),
            1
        );
        assert!(inspection.history_snapshot().is_err());
        assert_eq!(
            engine
                .close(&worth_execution::ExecutionRequest::serial(
                    &crate::test_execution_policy::serial_request()
                ))
                .unwrap_err(),
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
