pub use worth_foundational::{
    FoundationalBoundaryEvidenceContinuityAttachmentScope,
    FoundationalBoundaryEvidenceLineageOutcomeKind,
};
pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::{domain, foundation, runtime};

pub use super::installed_operation_fixture::{
    lineage_workflow_workspace, workflow_workspace, GeometryDomain, LineageEvidenceScenario,
    ReadFamily, WorkflowRead,
};

pub fn mutation_authority(label: &str) -> runtime::WorthQueryMutationAuthorityIdentity {
    runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(
        runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(label).unwrap(),
    )
    .unwrap()
}

pub fn execute(
    workspace: &mut runtime::WorthQueryWorkspace,
) -> domain::WorthQueryCompletedWorkflowTrace<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::MutationPreparationLaneWitness,
> {
    bind(workspace)
        .admit_workflow_resources(
            crate::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .reexecute(intent(), workspace)
        .unwrap()
}

pub fn bind(
    workspace: &runtime::WorthQueryWorkspace,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::MutationPreparationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
}

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
