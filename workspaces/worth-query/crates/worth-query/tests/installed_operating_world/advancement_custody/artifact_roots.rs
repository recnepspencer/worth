//! Artifact advancement has two executing spellings, each with one request.
use super::super::installed_operation_fixture::{
    artifact_lease_workspace, artifact_move_workspace, bind_artifact_workflow,
};
use super::*;

#[test]
fn moved_artifact_advancement_opens_before_its_reader() {
    verify(
        "artifact move advance",
        || {
            let (mut workspace, _probe) = artifact_move_workspace("artifact-move-custody").unwrap();
            let run = bind_artifact_workflow(&workspace)
                .admit_workflow_resources(execution_resource_request(), &workspace)
                .unwrap()
                .start_workflow(&mut workspace)
                .unwrap()
                .advance(
                    "produce",
                    domain::WorthQueryWorkflowValue::Text("produce".into()),
                    &mut workspace,
                )
                .unwrap();
            (workspace, run)
        },
        |run, workspace| match run.advance_with_artifact("consume", "produce", workspace) {
            TransitionOutcome::Success(_) => None,
            TransitionOutcome::Denied(denial) => match denial.kind() {
                domain::WorthQueryWorkflowAdvanceDenialKind::ExecutionRequest(cause) => {
                    Some(*cause)
                }
                other => panic!("unexpected artifact denial: {other:?}"),
            },
            _ => panic!("unexpected artifact move posture"),
        },
    );
}

#[test]
fn leased_artifact_advancement_opens_before_its_reader() {
    verify(
        "artifact lease advance",
        || {
            let (mut workspace, _probe) =
                artifact_lease_workspace("artifact-lease-custody").unwrap();
            let run = bind_artifact_workflow(&workspace)
                .admit_workflow_resources(execution_resource_request(), &workspace)
                .unwrap()
                .start_workflow(&mut workspace)
                .unwrap()
                .advance(
                    "produce",
                    domain::WorthQueryWorkflowValue::Text("produce".into()),
                    &mut workspace,
                )
                .unwrap();
            (workspace, run)
        },
        |run, workspace| match run.advance_with_artifact_lease(
            "observe-a",
            "produce",
            "observer-a",
            workspace,
        ) {
            TransitionOutcome::Success(_) => None,
            TransitionOutcome::Denied(denial) => match denial.kind() {
                domain::WorthQueryWorkflowAdvanceDenialKind::ExecutionRequest(cause) => {
                    Some(*cause)
                }
                other => panic!("unexpected leased artifact denial: {other:?}"),
            },
            _ => panic!("unexpected artifact lease posture"),
        },
    );
}
