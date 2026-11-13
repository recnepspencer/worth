//! Captured target input and original keys recover independently of output-demand custody.
use super::*;
use worth_query_host::facade::WorthQueryApplicationHomeLocation as Location;
#[test]
fn directory_captured_workflow_resumes_the_original_target_and_key_after_close() {
    let location = Location::Directory(super::super::fixture::location());
    let (home, authentication) = open(&location);
    let (definition, target) = definition::unsigned();
    let definition = publish_with(&home, definition);
    let (instance, original) = start(&home, definition);
    let identity = instance.entity_id();
    assert!(matches!(
        propose_captured(&home, instance, 1003, &target),
        WorkflowProposalOutcome::Published(_)
    ));
    drop(authentication);
    close(home);
    let (home, authentication) = open(&location);
    let instance = observation::instance(&home, &original);
    assert_eq!(instance.entity_id(), identity);
    assert!(matches!(
        propose_captured(&home, instance.clone(), 1003, &target),
        WorkflowProposalOutcome::RecoveredCommitted(_)
    ));
    let required = match advance(&home, instance.clone(), 1004) {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("the original captured target is ready: {other:?}"),
    };
    assert!(required
        .workflow_execution_requirements()
        .approval()
        .is_none());
    {
        let scope = request_scope();
        let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
        let result = home
            .runtime()
            .request(&principal, &scope)
            .on_branch(instance.branch())
            .mutate(input())
            .without_source()
            .idempotency(&1012u64)
            .for_workflow_operation(home.vocabulary(), &required)
            .unwrap()
            .advanced()
            .execute_in_program(
                home.runtime(),
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(
            matches!(
                result,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{result:?}"
        );
    }
    drop(authentication);
    close(home);
    let (home, authentication) = open(&location);
    let instance = observation::instance(&home, &original);
    assert!(
        matches!(advance(&home, instance, 1005), WorkflowProgressOutcome::Completed(done) if done.node_path() == "completed")
    );
    drop(authentication);
    close(home);
    let (home, authentication) = open(&location);
    match observation::read(&home, &original) {
        WorthQueryWorkflowInstanceRead::Historical { observation, .. } => {
            assert_eq!(
                observation.state(),
                WorthQueryWorkflowObservedState::Completed
            );
            assert_eq!(
                observation.recorded_paths(),
                &["propose", "apply", "completed"]
            );
        }
        _ => panic!("the original captured workflow is terminal after reopen"),
    }
    assert_eq!(
        document::readback::read_retention(
            home.runtime().runtime(),
            home.runtime().runtime().current_world()
        ),
        8
    );
    drop(authentication);
    close(home);
}
fn input() -> document::schema::SetRetentionInput {
    document::schema::SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 8,
    }
}
fn propose_captured(
    home: &Home,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
    target: &TargetNode,
) -> WorkflowProposalOutcome {
    let scope = request_scope();
    let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
    let request = home
        .runtime()
        .request(&principal, &scope)
        .on_branch(instance.branch());
    let original = request
        .mutate(input())
        .without_source()
        .idempotency_owned(1012u64)
        .unwrap_or_else(|_| panic!("original target description"));
    let inputs = WorthQueryWorkflowOperationInputs::new(home.runtime())
        .with_at::<document::programs::OccurrenceAction, _>(target, original)
        .unwrap_or_else(|_| panic!("installed target action"));
    let outcome = request
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&key)
        .prepare_workflow_proposal_with_inputs(home.vocabulary(), instance, &inputs)
        .unwrap()
        .execute();
    assert!(
        matches!(
            &outcome,
            WorkflowProposalOutcome::Published(_) | WorkflowProposalOutcome::RecoveredCommitted(_)
        ),
        "captured proposal: {outcome:?}"
    );
    outcome
}
