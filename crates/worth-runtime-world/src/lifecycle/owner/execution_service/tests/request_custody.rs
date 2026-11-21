use super::{execute_without_signal, prepare_relational, settled, setup};
use crate::publication::{NoEffectCause, OwnerExecutionOutcome};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};

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
    let prepared = prepare_relational(
        execution,
        &fixture,
        &owner,
        &expected,
        "original-short-stop",
    );
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        CancellationToken::new(),
        None,
    );
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
}
