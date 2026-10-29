pub use worth_query::facade::{certification, domain};

pub use super::installed_operation_fixture::{
    artifact_move_workspace, bind_artifact_workflow, move_intent,
};

pub fn producer_artifact(
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
