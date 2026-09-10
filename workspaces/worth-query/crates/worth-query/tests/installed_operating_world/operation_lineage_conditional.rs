use worth_proof::TransitionOutcome;
use worth_query::facade::{domain, foundation, runtime};

use super::conditional_node_contract::dependency;
use super::installed_operation_fixture::{
    reverted_conditional_lineage_workflow_workspace, GeometryDomain, ReadFamily, WorkflowRead,
};

#[test]
fn conditional_no_change_or_deferral_cannot_claim_fresh_lineage() {
    let mut workspace = reverted_conditional_lineage_workflow_workspace(
        "conditional-lineage-reverted",
        stage_node(),
    )
    .unwrap();

    assert!(matches!(
        bind(&workspace).admit_workflow_resources(crate::suite::installed_operation_fixture::execution_resource_request(), &workspace).unwrap().reexecute(intent(), &mut workspace),
        TransitionOutcome::Deferred(
            domain::WorthQueryWorkflowReexecutionStop::ConditionalDeferred { stage_identity, .. }
        ) if stage_identity == "publish"
    ));
}

fn stage_node() -> domain::WorthQueryPortableConditionalNodeDeclaration {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        "publish-lineage-when-changed",
        domain::WorthQueryConditionalNodeRole::WorkflowStage,
    )
    .dependencies([dependency.clone()])
    .outputs([
        domain::WorthQueryConditionalNodeOutput::WorkflowStageOutput {
            contract: domain::WorthQueryWorkflowValueContract::Projection,
        },
    ])
    .required_context([domain::WorthQueryConditionalNodeContext::WorkflowRun])
    .evaluation(
        domain::WorthQueryConditionalEvaluationCondition::aspect_filtered([dependency]).unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
    )
    .comparison(
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::NotReusable,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        domain::WorthQueryArtifactPosture::Ephemeral,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::IsWorkflowStageOutput)
    .finish()
    .unwrap()
}

fn bind(
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

fn intent() -> domain::WorthQueryNormalizedWorkflowIntent {
    use domain::{WorthQueryWorkflowIntentStage as Stage, WorthQueryWorkflowIntentValue as Value};
    domain::WorthQueryNormalizedWorkflowIntent::new(vec![
        Stage::new("start", Value::NotRequired),
        Stage::new("right", Value::Text("start".into())),
        Stage::new("left", Value::Text("start".into())),
        Stage::new("publish", Value::Text("join".into())),
    ])
    .unwrap()
}
