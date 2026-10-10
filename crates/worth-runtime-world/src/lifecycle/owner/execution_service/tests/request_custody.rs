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
