//! Actual retained-root ownership and cancellation through original Native ports.
use super::occurrences::{fixture, prepare};
use super::*;
use worth_relational::facade::mvcc::{
    RelationalCancellationSource, RelationalInterruptionBoundary, RelationalOperationInterruption,
    RelationalRetainedEntitySelectionPreparationError as Error,
};

#[test]
fn pending_selection_holds_the_historical_root_until_original_candidate_discard() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = fixture().build_runtime();
    let old = create(execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let current = update(execution, &runtime, [(model, "5")]);
    let candidate = prepare(execution, &runtime, &source, model);
    release_test_commit_snapshot(&runtime, &old);
    release_test_commit_snapshot(&runtime, &current);
    drop(source);
    let pending = runtime.observe_retained_storage().unwrap();
    assert!(pending.held_roots > 0);
    runtime
        .owner_component_services()
        .preparation_port()
        .discard_prepared_candidate(candidate)
        .unwrap();
    let released = runtime.observe_retained_storage().unwrap();
    assert!(
        released.held_roots < pending.held_roots,
        "{pending:?} -> {released:?}"
    );
    assert!(released.reclaimable_roots > pending.reclaimable_roots);
    assert_eq!(
        released.current_root_authoritative_bytes,
        pending.current_root_authoritative_bytes
    );
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id(),
        Some(current.commit.commit_id)
    );
}

#[test]
fn cancellation_before_preparation_and_after_candidate_keeps_the_original_current_head() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = fixture().build_runtime();
    let old = create(execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let current = update(execution, &runtime, [(model, "5")]);
    let target = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let services = runtime.owner_component_services();
    let cancellation = RelationalCancellationSource::new();
    let transaction = runtime
        .begin_branch_transaction_with_control(
            &target,
            RelationalTransactionIntent::ordinary(),
            cancellation.token().into(),
        )
        .unwrap();
    let selection = services
        .preparation_port()
        .admit_retained_entity_selection(transaction, &source, model)
        .unwrap();
    cancellation.cancel();
    let Err(Error::Preparation(TransactionCommitError::Interrupted { interruption, .. })) =
        services
            .preparation_port()
            .prepare_retained_entity_selection(execution, selection)
    else {
        panic!("cancelled input must stop before retained payload preparation")
    };
    assert_eq!(
        interruption.interruption(),
        RelationalOperationInterruption::Cancelled
    );
    assert_eq!(
        interruption.boundary(),
        RelationalInterruptionBoundary::ProposalValidation
    );
    let cancellation = RelationalCancellationSource::new();
    let transaction = runtime
        .begin_branch_transaction_with_control(
            &target,
            RelationalTransactionIntent::ordinary(),
            cancellation.token().into(),
        )
        .unwrap();
    let selection = services
        .preparation_port()
        .admit_retained_entity_selection(transaction, &source, model)
        .unwrap();
    let candidate = services
        .preparation_port()
        .prepare_retained_entity_selection(execution, selection)
        .unwrap();
    cancellation.cancel();
    let RelationalPublicationOutcome::Interrupted(interruption) = services
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("cancelled candidate must use the original publication interruption")
    };
    assert_eq!(
        interruption.interruption(),
        RelationalOperationInterruption::Cancelled
    );
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap()
            .reference(),
        target.reference()
    );
    assert_eq!(
        values(execution, &runtime, &current, [model, model]),
        ["5", "5"].map(string_aspect_value)
    );
    drop(source);
    release_test_commit_snapshot(&runtime, &old);
    release_test_commit_snapshot(&runtime, &current);
    assert_eq!(runtime.observe_retained_storage().unwrap().held_roots, 0);
}
