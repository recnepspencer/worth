pub use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType, StructAspectShape,
};
pub use worth_query::facade::domain;

pub use super::super::conditional_node_contract::{dependency, dependency_for_role};
pub use super::super::installed_operation_fixture::{
    conditional_workflow_workspace, conditional_workspace, GeometryDomain, ReadFamily, WorkflowRead,
};

pub fn conditional_declaration(
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

pub fn derived_contract() -> AspectContract {
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
