use super::*;
use crate::lifecycle::RuntimeWorldInstant;
use crate::publication::RuntimeWorldPublicationOutcome;

struct PanickingClock;
impl RuntimeWorldClockSource for PanickingClock {
    fn now(&self) -> RuntimeWorldInstant {
        panic!("clock fails at exact-cell cutoff")
    }
}
#[test]
fn final_clock_panic_keeps_owner_effects_without_product_movement() {
    let (fixture, owner, expected) = setup();
    let cancellation = RuntimeWorldCancellationSource::new();
    let intent =
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate("clock-panic"),
            );
    let prepared = owner
        .prepare_publication(
            expected.clone(),
            intent,
            &cancellation.token(),
            Some(RuntimeWorldInstant::from_ticks(10)),
        )
        .unwrap();
    let settlement = settled(execute_without_signal(&owner, prepared));
    let successor = settlement.successor_basis().unwrap().clone();
    let ready = settlement.ready(successor).unwrap();
    let cell = owner.state.branches.root_cell().unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ready.publish_controlled(
            &cell,
            &cancellation.token(),
            &RuntimeWorldClock::from_source(PanickingClock),
            #[cfg(feature = "test-operation-control")]
            &owner.state.operation_control,
        )))
        .is_err()
    );
    assert_eq!(cell.atomic_snapshot(), *expected.snapshot());
    let handle = owner
        .recovery_handles()
        .pop()
        .expect("effect remains recoverable");
    assert_eq!(
        owner
            .inspect_recovery(&handle)
            .unwrap()
            .progress()
            .relational_posture(),
        RelationalAttemptProgressPosture::Settled
    );
}

struct CancellingClock(Arc<RuntimeWorldCancellationSource>);
impl RuntimeWorldClockSource for CancellingClock {
    fn now(&self) -> RuntimeWorldInstant {
        self.0.cancel();
        RuntimeWorldInstant::from_ticks(0)
    }
}
#[test]
fn cancellation_during_clock_check_prevents_product_movement() {
    let (fixture, owner, expected) = setup();
    let source = Arc::new(RuntimeWorldCancellationSource::new());
    let intent =
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate("late-cancellation"),
            );
    let prepared = owner
        .prepare_publication(
            expected.clone(),
            intent,
            &source.token(),
            Some(RuntimeWorldInstant::from_ticks(10)),
        )
        .unwrap();
    let settlement = settled(execute_without_signal(&owner, prepared));
    let successor = settlement.successor_basis().unwrap().clone();
    let ready = settlement.ready(successor).unwrap();
    let cell = owner.state.branches.root_cell().unwrap();
    let record = match ready.publish_controlled(
        &cell,
        &source.token(),
        &RuntimeWorldClock::from_source(CancellingClock(source)),
        #[cfg(feature = "test-operation-control")]
        &owner.state.operation_control,
    ) {
        RuntimeWorldPublicationOutcome::ProductUnpublished(record) => record,
        other => panic!("cancellation before final atomic cutoff: {other:?}"),
    };
    assert_eq!(
        record.cause(),
        ProductUnpublishedCause::CancellationAfterEffect
    );
    assert_eq!(cell.atomic_snapshot(), *expected.snapshot());
    assert_eq!(
        record.progress().relational_posture(),
        RelationalAttemptProgressPosture::Settled
    );
}
