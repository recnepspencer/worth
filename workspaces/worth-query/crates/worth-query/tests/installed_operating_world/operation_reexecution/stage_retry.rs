use super::*;

#[test]
fn retry_requires_installed_idempotence_and_never_reuses_attempt_identity() {
    let mut workspace = workflow_workspace("idempotent-stage-retry").unwrap();
    let run = bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap();
    let first = run
        .prepare_stage_attempt(
            "left",
            domain::WorthQueryWorkflowIntentValue::Text("fail-dependency".into()),
        )
        .unwrap();
    let first_identity = first.identity().to_owned();
    let failure = match first.execute(
        &mut workspace,
        ExecutionRequest::serial(&workflow_request()),
    ) {
        domain::WorthQueryWorkflowStageAttemptOutcome::Retryable(failure) => failure,
        _ => panic!("effect-free declared executor failure was not retryable"),
    };
    assert_eq!(failure.failed_attempt_identity(), first_identity);
    assert!(failure.denial().executed_effects().is_empty());
    let second = failure.retry();
    assert_ne!(second.identity(), first_identity);
    assert!(matches!(
        second.execute(
            &mut workspace,
            ExecutionRequest::serial(&workflow_request())
        ),
        domain::WorthQueryWorkflowStageAttemptOutcome::Retryable(_)
    ));
}
