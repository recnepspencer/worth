use crate::workflow_request::workflow_request;
use worth_query::facade::runtime::ExecutionRequest;
use worth_query::facade::{certification, domain};

use super::installed_operation_fixture::{
    artifact_move_workspace, bind_artifact_workflow, move_intent,
};

#[test]
fn fresh_runs_ignore_operational_artifact_identity_but_compare_canonical_meaning() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-reexecution").unwrap();
    let original = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            move_intent("produce"),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap();
    let reexecuted = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            move_intent("produce"),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap();
    let original_semantics = original.semantics();
    let reexecuted_semantics = reexecuted.semantics();
    let original_artifact = producer_artifact(&original_semantics);
    let reexecuted_artifact = producer_artifact(&reexecuted_semantics);

    assert_ne!(
        original_artifact.handle_identity(),
        reexecuted_artifact.handle_identity()
    );
    assert_ne!(
        original_artifact.occurrence_identity(),
        reexecuted_artifact.occurrence_identity()
    );
    assert_eq!(
        original_artifact.semantic_projection(),
        reexecuted_artifact.semantic_projection()
    );
    assert_eq!(
        domain::compare_exact_workflow_traces(
            &original_semantics,
            &reexecuted_semantics,
            Default::default(),
        ),
        domain::WorthQueryReplayComparison::Equivalent
    );
    assert_eq!(probe.allocations(), 2);
    assert_eq!(probe.projection_calls(), 2);
    assert_eq!(probe.disposals(), 2);
}

#[test]
fn certification_replay_reexecutes_from_intent_without_retaining_an_operational_handle() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-cert-replay").unwrap();
    let original = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            move_intent("produce"),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap();
    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        bind_artifact_workflow(&workspace),
        move_intent("produce"),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
        ExecutionRequest::serial(&workflow_request()),
    )
    .unwrap();

    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Equivalent
    );
    assert_ne!(
        replay.original_trace_identity(),
        replay.replay_trace_identity()
    );
    assert_eq!(probe.allocations(), 2);
    assert_eq!(probe.projection_calls(), 2);
    assert_eq!(probe.disposals(), 2);
}

fn producer_artifact(
    semantics: &domain::WorthQueryWorkflowTraceSemantics,
) -> &domain::WorthQueryArtifactTraceMeaning {
    let output = semantics
        .stages()
        .iter()
        .find(|stage| stage.stage_identity() == "produce")
        .expect("artifact trace retains the producer stage")
        .output();
    match output {
        domain::WorthQueryWorkflowSemanticValue::InstalledArtifact(artifact) => artifact,
        _ => panic!("producer trace did not retain artifact meaning"),
    }
}
