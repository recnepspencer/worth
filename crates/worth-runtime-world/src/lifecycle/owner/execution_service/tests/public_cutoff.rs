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
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let cancellation = RuntimeWorldCancellationSource::new();
    let intent =
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate(&execution, "clock-panic"),
            );
    let prepared = owner
        .prepare_publication(
            expected.clone(),
            intent,
            &cancellation.token(),
            Some(RuntimeWorldInstant::from_ticks(10)),
        )
        .unwrap();
    let settlement = settled(execute_without_signal(execution, &owner, prepared));
    let successor = settlement.successor_basis().unwrap().clone();
    let ready = settlement.ready(successor).unwrap();
    let ready = ready
        .capture_memory_original(
            &execution,
            &owner.state.history,
            &owner.state.relational_services().basis_port(),
        )
        .unwrap();
    let cell = owner.state.branches.root_cell().unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ready.publish_controlled(
            &cell,
            &cancellation.token(),
            &RuntimeWorldClock::from_source(PanickingClock),
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
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = Arc::new(RuntimeWorldCancellationSource::new());
    let intent =
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate(&execution, "late-cancellation"),
            );
    let prepared = owner
        .prepare_publication(
            expected.clone(),
            intent,
            &source.token(),
            Some(RuntimeWorldInstant::from_ticks(10)),
        )
        .unwrap();
    let settlement = settled(execute_without_signal(execution, &owner, prepared));
    let successor = settlement.successor_basis().unwrap().clone();
    let ready = settlement.ready(successor).unwrap();
    let ready = ready
        .capture_memory_original(
            &execution,
            &owner.state.history,
            &owner.state.relational_services().basis_port(),
        )
        .unwrap();
    let cell = owner.state.branches.root_cell().unwrap();
    let record = match ready.publish_controlled(
        &cell,
        &source.token(),
        &RuntimeWorldClock::from_source(CancellingClock(source)),
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

#[test]
fn cutover_callback_world_cancellation_and_deadline_stop_before_native_effect() {
    use crate::publication::NoEffectCause;
    use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};

    let native_head = |fixture: &RealReferenceFixture| {
        let rt = fixture.relational_runtime();
        rt.observe_branch(&rt.main_branch_identity())
            .unwrap()
            .1
            .descriptor()
            .clone()
    };
    let seed_with_clock = |clock: RuntimeWorldClock| {
        let (fixture, owner, expected) = setup_with_clock(clock);
        let ready = ready_relational_competitor(&fixture, &owner, &expected, "cutover-seed");
        let seeded = publish_ready_competing_head(&owner, ready, &expected);
        (fixture, owner, seeded)
    };
    let host = crate::test_execution_policy::serial_request();
    let setup_exec = ExecutionRequest::serial(&host);

    // 1. Cancel the World token inside the cutover callback before calling perform().
    let clock = MutableClock::new(0);
    let (fixture, owner, expected) =
        seed_with_clock(RuntimeWorldClock::from_source(clock.clone()));
    let before_native = native_head(&fixture);
    let cancel = RuntimeWorldCancellationSource::new();
    let mut intent =
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate(&setup_exec, "cutover-cancel"),
            );
    intent.require_current_read(expected.clone()).unwrap();
    let prepared = RuntimeWorldPreparationService::prepare_publication(
        owner.as_ref(),
        expected.clone(),
        intent,
        &cancel.token(),
        Some(RuntimeWorldInstant::from_ticks(10)),
    )
    .unwrap();
    let mem = SerialMemoryBudget::from_policy(&owner.state.execution.request_policy());
    let req = SerialRequest::from_memory(mem.clone(), CancellationToken::new(), None);
    let mut callback_ran = false;
    let outcome = RuntimeWorldOwnerExecutionService::execute_without_signal_with_cutover(
        owner.as_ref(),
        ExecutionRequest::serial(&req),
        prepared,
        &cancel.token(),
        Some(&mut |cutover| {
            callback_ran = true;
            cancel.cancel();
            cutover.perform();
        }),
    );
    assert!(callback_ran);
    assert!(matches!(
        outcome,
        OwnerExecutionOutcome::NoEffect(refused)
            if refused.cause() == NoEffectCause::CancelledBeforeEffect
    ));
    assert_eq!(native_head(&fixture), before_native);
    assert!(owner.recovery_handles().is_empty());
    assert_eq!(mem.charged_memory_bytes(), 0);

    // 2. Advance the World clock past deadline inside the cutover callback before perform().
    let clock = MutableClock::new(0);
    let (fixture, owner, expected) =
        seed_with_clock(RuntimeWorldClock::from_source(clock.clone()));
    let before_native = native_head(&fixture);
    let cancel = RuntimeWorldCancellationSource::new();
    let mut intent =
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate(&setup_exec, "cutover-deadline"),
            );
    intent.require_current_read(expected.clone()).unwrap();
    let prepared = RuntimeWorldPreparationService::prepare_publication(
        owner.as_ref(),
        expected,
        intent,
        &cancel.token(),
        Some(RuntimeWorldInstant::from_ticks(10)),
    )
    .unwrap();
    let mem = SerialMemoryBudget::from_policy(&owner.state.execution.request_policy());
    let req = SerialRequest::from_memory(mem.clone(), CancellationToken::new(), None);
    let mut callback_ran = false;
    let outcome = RuntimeWorldOwnerExecutionService::execute_without_signal_with_cutover(
        owner.as_ref(),
        ExecutionRequest::serial(&req),
        prepared,
        &cancel.token(),
        Some(&mut |cutover| {
            callback_ran = true;
            clock.set(10);
            cutover.perform();
        }),
    );
    assert!(callback_ran);
    assert!(matches!(
        outcome,
        OwnerExecutionOutcome::NoEffect(refused)
            if refused.cause() == NoEffectCause::DeadlineBeforeEffect
    ));
    assert_eq!(native_head(&fixture), before_native);
    assert!(owner.recovery_handles().is_empty());
    assert_eq!(mem.charged_memory_bytes(), 0);
}
