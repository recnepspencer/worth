use super::*;

#[test]
fn settled_owner_recovery_adopts_without_replaying_relational_work() {
    use crate::lifecycle::ports::{RuntimeWorldOwnerExecutionService, RuntimeWorldRecoveryService};
    let (fixture, owner, expected) = setup_with_retention_capacity(16, 16);
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);
    fixture.fail_next_relational_durable_append();
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = prepare_relational(
        &execution,
        &fixture,
        owner.as_ref(),
        expected.clone(),
        "settled-adoption",
    );
    let retained = match RuntimeWorldOwnerExecutionService::execute_without_signal(
        owner.as_ref(),
        execution,
        prepared,
        &cancellation.token(),
    ) {
        OwnerExecutionOutcome::ProductUnpublished(retained) => retained,
        other => panic!("the injected settlement failure must retain: {other:?}"),
    };
    let handle = retained.recovery_handle();
    RuntimeWorldRecoveryService::continue_effects(owner.as_ref(), retained)
        .expect("the exact Relational owner result settles");
    let settled = RuntimeWorldRecoveryService::inspect_effects(owner.as_ref(), &handle)
        .expect("the settled record remains in World custody");
    let prepared = RuntimeWorldRecoveryService::prepare_settled_relational_adoption(
        owner.as_ref(),
        &settled,
        &cancellation.token(),
        None,
    )
    .expect("World admits the settled owner result against its original product head");
    drop(settled);
    let outcome = RuntimeWorldOwnerExecutionService::execute_without_signal(
        owner.as_ref(),
        execution,
        prepared,
        &cancellation.token(),
    );
    let performed = match owner.finish_publication(outcome, &cancellation.token()) {
        RuntimeWorldPublicationOutcome::Performed(performed) => performed,
        other => panic!("the settled owner result must publish without replay: {other:?}"),
    };
    assert_eq!(performed.old_product_head(), expected.snapshot());
    assert_eq!(
        performed.cost_counters().relational_owner_contacts(),
        0,
        "adoption cannot contact or replay the Relational owner"
    );
    assert!(performed
        .component_results()
        .relational_commit_result()
        .is_some());
    drop(performed);
    RuntimeWorldRecoveryService::release_effects(owner.as_ref(), &handle, 0)
        .expect("the superseded recovery record releases after product movement");
}
