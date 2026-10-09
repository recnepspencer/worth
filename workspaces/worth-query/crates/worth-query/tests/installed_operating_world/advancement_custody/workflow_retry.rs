//! Ordinary replay and retry enter through their real host roots.
use super::super::operation_reexecution::intent;
use super::*;

#[test]
fn workflow_reexecution_opens_before_its_reader() {
    verify(
        "workflow reexecute",
        || {
            let workspace = workflow_workspace("workflow-reexecute-custody").unwrap();
            let operation = bind(&workspace)
                .admit_workflow_resources(execution_resource_request(), &workspace)
                .unwrap();
            (workspace, operation)
        },
        |operation, workspace| match operation.reexecute(intent(), workspace) {
            TransitionOutcome::Success(_) => None,
            TransitionOutcome::Denied(domain::WorthQueryWorkflowReexecutionStop::Start(denial)) => {
                match denial.kind() {
                    domain::WorthQueryWorkflowStartDenialKind::ExecutionRequest(cause) => {
                        Some(*cause)
                    }
                    other => panic!("unexpected reexecution denial: {other:?}"),
                }
            }
            _ => panic!("unexpected workflow reexecution posture"),
        },
    );
}

#[test]
fn workflow_stage_attempt_opens_before_its_reader() {
    verify(
        "workflow stage attempt execute",
        || {
            let (mut workspace, run) = super::workflow_roots::run_setup();
            let run = run
                .advance(
                    "start",
                    domain::WorthQueryWorkflowValue::NotRequired,
                    &mut workspace,
                )
                .unwrap();
            let attempt = run
                .prepare_stage_attempt(
                    "left",
                    domain::WorthQueryWorkflowIntentValue::Text("start".into()),
                )
                .unwrap();
            (workspace, attempt)
        },
        |attempt, workspace| match attempt.execute(workspace) {
            domain::WorthQueryWorkflowStageAttemptOutcome::Success(_) => None,
            domain::WorthQueryWorkflowStageAttemptOutcome::Failed(failure) => {
                match failure.denial().kind() {
                    domain::WorthQueryWorkflowAdvanceDenialKind::ExecutionRequest(cause) => {
                        Some(*cause)
                    }
                    other => panic!("unexpected stage attempt denial: {other:?}"),
                }
            }
            _ => panic!("unexpected stage attempt posture"),
        },
    );
}
