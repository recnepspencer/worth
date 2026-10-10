use super::{execute_without_signal, prepare_relational, settled, setup};
use crate::publication::{NoEffectCause, OwnerExecutionOutcome};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};

#[test]
fn carried_request_cancellation_refuses_a_prepared_relational_publication() {
    let (fixture, owner, expected) = setup();
    let prepared = prepare_relational(&fixture, &owner, &expected, "request-cancelled");
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
    let (fixture, owner, expected) = setup();
    let prepared = prepare_relational(&fixture, &owner, &expected, "request-deadline");
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        CancellationToken::new(),
        Some(std::time::Instant::now() - std::time::Duration::from_secs(1)),
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
    let (fixture, owner, expected) = setup();
    let prepared = prepare_relational(&fixture, &owner, &expected, "request-admitted");
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
    use worth_execution::{ExecutionScan, ExecutionWorkCeiling, MapKernelFailure, ScanOutcome};
    use worth_foundational::PartitionIdentity;

    let (fixture, owner, expected) = setup();
    let prepared = prepare_relational(&fixture, &owner, &expected, "request-spent-exactly");
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, 1_u64)]).unwrap();
    let (performed, report) = request
        .run(ExecutionWorkCeiling::new(1), |_| {
            let charged = scan.run(None, 0_u64, 0, 0, 0, 0, |sum, item, work| {
                work.checkpoint(1)?;
                Ok::<_, MapKernelFailure<()>>((sum + item, ()))
            });
            assert_eq!(charged.report().charged_work(), 1);
            assert!(matches!(charged, ScanOutcome::Complete { state: 1, .. }));
            settled(execute_without_signal(request, &owner, prepared))
        })
        .unwrap();
    assert_eq!(report.charged_work(), 1);
    assert_eq!(performed.progress().owner_effect_count(), 1);
}
