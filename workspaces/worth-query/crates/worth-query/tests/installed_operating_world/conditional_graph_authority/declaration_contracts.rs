use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType, StructAspectShape,
};
use worth_query::facade::domain;

use super::super::conditional_node_contract::{dependency, dependency_for_role};
use super::super::installed_operation_fixture::{
    conditional_workflow_workspace, conditional_workspace, GeometryDomain, ReadFamily, WorkflowRead,
};

#[test]
fn dependency_outside_operation_graph_scope_fails_package_installation() {
    let dependency = dependency_for_role(
        "foreign-model",
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let node = conditional_declaration(
        "foreign-read-role",
        domain::WorthQueryConditionalNodeRole::Computed,
        vec![dependency],
        vec![domain::WorthQueryConditionalNodeOutput::OperationOutput {
            projection_role: domain::WorthQueryOperationProjectionRole::new("vertex").unwrap(),
        }],
        domain::WorthQueryConditionalNodeContext::Basis,
        domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
    )
    .unwrap();

    assert!(conditional_workspace("foreign-conditional-read-role", node).is_err());
}

#[test]
fn undeclared_operation_output_role_fails_package_installation() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_declaration(
        "foreign-output-role",
        domain::WorthQueryConditionalNodeRole::Computed,
        vec![dependency],
        vec![domain::WorthQueryConditionalNodeOutput::OperationOutput {
            projection_role: domain::WorthQueryOperationProjectionRole::new("foreign").unwrap(),
        }],
        domain::WorthQueryConditionalNodeContext::Basis,
        domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
    )
    .unwrap();

    assert!(conditional_workspace("foreign-conditional-output-role", node).is_err());
}

#[test]
fn derived_consequence_cannot_invent_touch_authority() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let touch = domain::WorthQueryConditionalTouchRole::new("model", "geometry").unwrap();
    let node = conditional_declaration(
        "foreign-touch-role",
        domain::WorthQueryConditionalNodeRole::Computed,
        vec![dependency],
        vec![
            domain::WorthQueryConditionalNodeOutput::DerivedAspect {
                contract: derived_contract(),
                locality: domain::WorthQuerySemanticLocality::SourceRecord,
                consequences: vec![domain::WorthQueryConditionalConsequenceRole::Touch(touch)],
            },
            domain::WorthQueryConditionalNodeOutput::OperationOutput {
                projection_role: domain::WorthQueryOperationProjectionRole::new("vertex").unwrap(),
            },
        ],
        domain::WorthQueryConditionalNodeContext::Basis,
        domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
    )
    .unwrap();

    assert!(conditional_workspace("foreign-conditional-touch-role", node).is_err());
}

#[test]
fn conditional_role_and_attachment_are_one_installed_meaning() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let stage_node = conditional_declaration(
        "publish-when-changed",
        domain::WorthQueryConditionalNodeRole::WorkflowStage,
        vec![dependency],
        vec![
            domain::WorthQueryConditionalNodeOutput::WorkflowStageOutput {
                contract: domain::WorthQueryWorkflowValueContract::Projection,
            },
        ],
        domain::WorthQueryConditionalNodeContext::WorkflowRun,
        domain::WorthQueryOutputRelationship::IsWorkflowStageOutput,
    )
    .unwrap();

    let workspace =
        conditional_workflow_workspace("conditional-workflow-stage", stage_node.clone())
            .expect("workflow-stage conditional installs through the ordinary package path");
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let operating_world = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    let operation = operating_world
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap();
    let domain::WorthQueryOperationWorkflowContract::Declared(workflow) =
        &operation.definition().semantics().workflow
    else {
        panic!("installed workflow contract")
    };
    assert_eq!(
        workflow
            .stages()
            .iter()
            .find(|stage| stage.identity() == "publish")
            .unwrap()
            .semantics()
            .conditional_nodes,
        vec![stage_node.clone()]
    );

    assert!(conditional_workspace("stage-role-at-operation", stage_node).is_err());
}

#[test]
fn output_relationship_cannot_claim_an_output_the_node_does_not_declare() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let result = conditional_declaration(
        "dishonest-output-relationship",
        domain::WorthQueryConditionalNodeRole::Computed,
        vec![dependency],
        vec![domain::WorthQueryConditionalNodeOutput::DerivedAspect {
            contract: derived_contract(),
            locality: domain::WorthQuerySemanticLocality::SourceRecord,
            consequences: vec![domain::WorthQueryConditionalConsequenceRole::DerivedOnly],
        }],
        domain::WorthQueryConditionalNodeContext::Basis,
        domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
    );

    assert_eq!(
        result.unwrap_err(),
        "conditional-node-output-relationship-missing-output"
    );
}

fn conditional_declaration(
    identity: &str,
    role: domain::WorthQueryConditionalNodeRole,
    dependencies: Vec<domain::WorthQuerySemanticTruthDependency>,
    outputs: Vec<domain::WorthQueryConditionalNodeOutput>,
    context: domain::WorthQueryConditionalNodeContext,
    relationship: domain::WorthQueryOutputRelationship,
) -> Result<domain::WorthQueryPortableConditionalNodeDeclaration, &'static str> {
    let condition =
        domain::WorthQueryConditionalEvaluationCondition::aspect_filtered(dependencies.clone())
            .unwrap();
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(identity, role)
        .dependencies(dependencies)
        .outputs(outputs)
        .required_context([context])
        .evaluation(
            condition,
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
        .output_relationship(relationship)
        .finish()
}

fn derived_contract() -> AspectContract {
    let field = FieldDeclaration::new(
        FieldKey::new("derived-id").unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap();
    AspectContract::struct_aspect(
        AspectKey::new("derived-identity").unwrap(),
        AspectIdentity(0x9140_0002),
        AspectContractRevision(1),
        StructAspectShape::new([field]).unwrap(),
    )
}
