pub use worth_foundational::FoundationalBoundaryEvidenceContinuityAttachmentScope;
pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::{certification, domain, foundation};

pub use super::installed_operation_fixture::{
    conditional_workflow_workspace, missing_replay_comparator_workspace, workflow_workspace,
    GeometryDomain, ReadFamily, WorkflowRead,
};

pub fn intent() -> domain::WorthQueryNormalizedWorkflowIntent {
    use domain::{WorthQueryWorkflowIntentStage as Stage, WorthQueryWorkflowIntentValue as Value};
    domain::WorthQueryNormalizedWorkflowIntent::new(vec![
        Stage::new("start", Value::NotRequired),
        Stage::new("right", Value::Text("start".into())),
        Stage::new("left", Value::Text("start".into())),
        Stage::new("publish", Value::Text("join".into())),
    ])
    .unwrap()
}

pub fn bind(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
}

pub mod stage_retry;
