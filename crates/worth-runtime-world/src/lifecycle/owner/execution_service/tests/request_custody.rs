use super::{execute_without_signal, prepare_relational, settled, setup};
use crate::publication::{NoEffectCause, OwnerExecutionOutcome};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};

fn relational_head(
    fixture: &crate::branch::reference_test_fixture::RealReferenceFixture,
) -> worth_relational::facade::branch::RelationalBranchBasisDescriptor {
    let rt = fixture.relational_runtime();
    rt.observe_branch(&rt.main_branch_identity())
        .unwrap()
        .1
        .descriptor()
        .clone()
}

#[test]
fn carried_request_cancellation_refuses_a_prepared_relational_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (fixture, owner, expected) = setup();
    let prepared = prepare_relational(execution, &fixture, &owner, &expected, "request-cancelled");
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        cancellation.token(),
        None,
    );
    let outcome = execute_without_signal(ExecutionRequest::serial(&request), &owner, prepared);
    let OwnerExecutionOutcome::NoEffect(refused) = outcome else {
        panic!("cancelled request reached publication: {outcome:?}");
    };
    assert_eq!(
        refused.cause(),
        NoEffectCause::ExecutionRequest(WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled
        ))
    );
}

#[test]
fn carried_request_deadline_stays_distinct_from_cancellation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (fixture, owner, expected) = setup();
    let prepared = prepare_relational(execution, &fixture, &owner, &expected, "request-deadline");
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        CancellationToken::new(),
        Some(
            worth_execution::MonotonicClock::system()
                .deadline_at(std::time::Instant::now() - std::time::Duration::from_secs(1)),
        ),
    );
    let outcome = execute_without_signal(ExecutionRequest::serial(&request), &owner, prepared);
    let OwnerExecutionOutcome::NoEffect(refused) = outcome else {
        panic!("expired request reached publication: {outcome:?}");
    };
    assert_eq!(
        refused.cause(),
        NoEffectCause::ExecutionRequest(WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::DeadlineElapsed
        ))
    );
}

#[test]
fn the_same_request_door_completes_prepared_publication_when_admitted() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (fixture, owner, expected) = setup();
    let prepared = prepare_relational(execution, &fixture, &owner, &expected, "request-admitted");
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        CancellationToken::new(),
        None,
    );
    let performed = settled(execute_without_signal(
        ExecutionRequest::serial(&request),
        &owner,
        prepared,
    ));
    assert_eq!(performed.progress().owner_effect_count(), 1);
}

#[test]
fn fully_spent_scope_still_publishes_when_no_charge_was_refused() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_execution::{ExecutionScan, ExecutionWorkCeiling, MapKernelFailure, ScanOutcome};
    use worth_foundational::PartitionIdentity;

    let (measure_fixture, measure_owner, measure_expected) = setup();
    let measure_prepared = prepare_relational(
        execution,
        &measure_fixture,
        &measure_owner,
        &measure_expected,
        "request-spent-measure",
    );
    let measure_serial = SerialRequest::from_memory(
        SerialMemoryBudget::from_policy(&measure_owner.state.execution.request_policy()),
        CancellationToken::new(),
        None,
    );
    let measure_request = ExecutionRequest::serial(&measure_serial);
    let (_, measure_report) = measure_request
        .run(ExecutionWorkCeiling::new(None), |_| {
            settled(execute_without_signal(
                measure_request,
                &measure_owner,
                measure_prepared,
            ))
        })
        .unwrap();
    let original_work = measure_report.charged_work();
    assert!(original_work > 0);

    let (fixture, owner, expected) = setup();
    let prepared = prepare_relational(
        execution,
        &fixture,
        &owner,
        &expected,
        "request-spent-exactly",
    );
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, 1_u64)]).unwrap();
    let ceiling = original_work.checked_add(1).unwrap();
    let (performed, report) = request
        .run(ExecutionWorkCeiling::new(Some(ceiling)), |_| {
            let charged = scan.run(None, 0_u64, 0, 0, 0, 0, |sum, item, work| {
                work.checkpoint(1)?;
                Ok::<_, MapKernelFailure<()>>((sum + item, ()))
            });
            assert_eq!(charged.report().charged_work(), 1);
            assert!(matches!(charged, ScanOutcome::Complete { state: 1, .. }));
            settled(execute_without_signal(request, &owner, prepared))
        })
        .unwrap();
    assert_eq!(report.charged_work(), ceiling);
    assert_eq!(performed.progress().owner_effect_count(), 1);
}

#[test]
fn memory_original_admission_one_unit_short_stops_before_owner_effects() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (measure_fixture, measure_owner, measure_expected) = setup();
    let measure_prepared = prepare_relational(
        execution,
        &measure_fixture,
        &measure_owner,
        &measure_expected,
        "original-short-measure",
    );
    let measure_serial = SerialRequest::from_memory(
        SerialMemoryBudget::from_policy(&measure_owner.state.execution.request_policy()),
        CancellationToken::new(),
        None,
    );
    let measure_request = ExecutionRequest::serial(&measure_serial);
    let (_, measure_report) = measure_request
        .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
            settled(execute_without_signal(
                measure_request,
                &measure_owner,
                measure_prepared,
            ))
        })
        .unwrap();
    let original_work = measure_report.charged_work();
    assert!(original_work > 0);

    let (fixture, owner, expected) = setup();
    let before_native = relational_head(&fixture);
    let prepared = prepare_relational(
        execution,
        &fixture,
        &owner,
        &expected,
        "original-short-stop",
    );
    let memory = SerialMemoryBudget::from_policy(&owner.state.execution.request_policy());
    let serial = SerialRequest::from_memory(memory.clone(), CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&serial);
    let outcome = request
        .run(
            worth_execution::ExecutionWorkCeiling::new(Some(original_work - 1)),
            |_| execute_without_signal(request, &owner, prepared),
        )
        .unwrap()
        .0;
    let OwnerExecutionOutcome::NoEffect(refused) = outcome else {
        panic!("one-unit-short original admission must stop before effects: {outcome:?}");
    };
    assert_eq!(
        refused.cause(),
        NoEffectCause::ExecutionRequest(WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::WorkCeiling
        ))
    );
    assert!(owner.recovery_handles().is_empty());
    assert_eq!(memory.charged_memory_bytes(), 0);
    assert_eq!(relational_head(&fixture), before_native);
}

#[test]
fn world_cancellation_and_deadline_after_first_gate_stop_before_native_effect() {
    use crate::lifecycle::{
        RuntimeWorldClock, RuntimeWorldClockSource, RuntimeWorldInstant,
        RuntimeWorldOwnerExecutionService, RuntimeWorldPreparationService,
    };
    use crate::publication::{CompositePublicationIntent, RuntimeWorldCancellationSource};
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    };
    use worth_relational::facade::mvcc::RelationalTransactionIntent;

    struct StepClock(Arc<AtomicBool>, AtomicUsize);
    impl RuntimeWorldClockSource for StepClock {
        fn now(&self) -> RuntimeWorldInstant {
            if !self.0.load(Ordering::Acquire) {
                return RuntimeWorldInstant::from_ticks(0);
            }
            // Armed call 0 is admit_owner_execution; armed call 1 is
            // recheck_pre_effect_cutoff immediately after original admission.
            if self.1.fetch_add(1, Ordering::AcqRel) == 0 {
                RuntimeWorldInstant::from_ticks(0)
            } else {
                RuntimeWorldInstant::from_ticks(10)
            }
        }
    }
    let armed = Arc::new(AtomicBool::new(false));
    let (fixture, owner, expected) = super::setup_with_clock(RuntimeWorldClock::from_source(
        StepClock(Arc::clone(&armed), AtomicUsize::new(0)),
    ));
    let before_native = relational_head(&fixture);
    let host = crate::test_execution_policy::serial_request();
    let setup_exec = ExecutionRequest::serial(&host);
    let cancel = RuntimeWorldCancellationSource::new();
    let prepared = RuntimeWorldPreparationService::prepare_publication(
        owner.as_ref(),
        expected.clone(),
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate(&setup_exec, "deadline-after-admission"),
            ),
        &cancel.token(),
        Some(RuntimeWorldInstant::from_ticks(5)),
    )
    .unwrap();
    armed.store(true, Ordering::Release);
    let mem = SerialMemoryBudget::from_policy(&owner.state.execution.request_policy());
    let req = SerialRequest::from_memory(mem.clone(), CancellationToken::new(), None);
    let outcome = RuntimeWorldOwnerExecutionService::execute_without_signal(
        owner.as_ref(),
        ExecutionRequest::serial(&req),
        prepared,
        &cancel.token(),
    );
    assert!(matches!(
        outcome,
        OwnerExecutionOutcome::NoEffect(refused) if refused.cause() == NoEffectCause::DeadlineBeforeEffect
    ));
    assert!(owner.recovery_handles().is_empty());
    assert_eq!(mem.charged_memory_bytes(), 0);
    assert_eq!(relational_head(&fixture), before_native);

    struct CancelOnArmedNow(Arc<AtomicBool>, Arc<RuntimeWorldCancellationSource>);
    impl RuntimeWorldClockSource for CancelOnArmedNow {
        fn now(&self) -> RuntimeWorldInstant {
            // Fires during admit_owner_execution's deadline check (after its
            // cancellation check already passed), before recheck_pre_effect_cutoff.
            if self.0.load(Ordering::Acquire) {
                self.1.cancel();
            }
            RuntimeWorldInstant::from_ticks(0)
        }
    }
    let armed = Arc::new(AtomicBool::new(false));
    let cancel = Arc::new(RuntimeWorldCancellationSource::new());
    let (fixture, owner, expected) = super::setup_with_clock(RuntimeWorldClock::from_source(
        CancelOnArmedNow(Arc::clone(&armed), Arc::clone(&cancel)),
    ));
    let before_native = relational_head(&fixture);
    let prepared = RuntimeWorldPreparationService::prepare_publication(
        owner.as_ref(),
        expected,
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate(&setup_exec, "cancel-after-admission"),
            ),
        &cancel.token(),
        Some(RuntimeWorldInstant::from_ticks(5)),
    )
    .unwrap();
    armed.store(true, Ordering::Release);
    let mem = SerialMemoryBudget::from_policy(&owner.state.execution.request_policy());
    let req = SerialRequest::from_memory(mem.clone(), CancellationToken::new(), None);
    let outcome = RuntimeWorldOwnerExecutionService::execute_without_signal(
        owner.as_ref(),
        ExecutionRequest::serial(&req),
        prepared,
        &cancel.token(),
    );
    assert!(matches!(
        outcome,
        OwnerExecutionOutcome::NoEffect(refused) if refused.cause() == NoEffectCause::CancelledBeforeEffect
    ));
    assert!(owner.recovery_handles().is_empty());
    assert_eq!(mem.charged_memory_bytes(), 0);
    assert_eq!(relational_head(&fixture), before_native);
}

#[test]
fn post_admission_no_effect_refunds_registry_growth_and_leaves_no_phantom_slot() {
    use crate::lifecycle::RuntimeWorldOwnerExecutionService;
    use crate::publication::RuntimeWorldCancellationSource;
    use worth_execution::ExecutionWorkCeiling;

    let (fixture, owner, expected) = setup();
    let setup_host = crate::test_execution_policy::serial_request();
    let setup_exec = ExecutionRequest::serial(&setup_host);
    let prepared = prepare_relational(&setup_exec, &fixture, &owner, &expected, "declined-cutover");
    let memory = SerialMemoryBudget::from_policy(&owner.state.execution.request_policy());
    let serial = SerialRequest::from_memory(memory.clone(), CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&serial);
    let cancel = RuntimeWorldCancellationSource::new();
    let outcome = RuntimeWorldOwnerExecutionService::execute_without_signal_with_cutover(
        owner.as_ref(),
        request,
        prepared,
        &cancel.token(),
        Some(&mut |_cutover| ()),
    );
    assert!(matches!(
        outcome,
        OwnerExecutionOutcome::NoEffect(refused)
            if matches!(refused.cause(), NoEffectCause::CurrentReadDependency(_))
    ));
    assert_eq!(
        memory.charged_memory_bytes(),
        0,
        "declined post-admission attempt must refund provisional registry growth"
    );
    // Zero-work checkpoint snapshot proves records.slots.len() == 0 (no phantom slot).
    let (snapshot, report) = request
        .run(ExecutionWorkCeiling::new(Some(0)), |_| {
            owner.state.history.snapshot_memory_originals(&request)
        })
        .unwrap();
    assert_eq!(snapshot.unwrap().len(), 0);
    assert_eq!(report.charged_work(), 0);
    assert!(owner.state.history.reserve_memory_readmission().is_ok());
}
