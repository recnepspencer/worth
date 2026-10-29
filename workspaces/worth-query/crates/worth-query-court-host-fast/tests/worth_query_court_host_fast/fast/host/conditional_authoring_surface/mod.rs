pub(crate) use worth_query_worlds::facade::host::conditional_authoring_surface::*;

#[test]
fn host_audience_can_name_the_complete_portable_conditional_contract() {
    fn author(
        dependency: domain::WorthQuerySemanticTruthDependency,
    ) -> Result<domain::WorthQueryPortableConditionalNodeDeclaration, &'static str> {
        domain::WorthQueryPortableConditionalNodeDeclaration::declare(
            "host-computed-node",
            domain::WorthQueryConditionalNodeRole::Computed,
        )
        .dependencies([dependency.clone()])
        .outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput {
            projection_role: domain::WorthQueryOperationProjectionRole::new("result")?,
        }])
        .required_context([domain::WorthQueryConditionalNodeContext::Snapshot])
        .evaluation(
            domain::WorthQueryConditionalEvaluationCondition::domain_specific::<HostCondition>([
                domain::WorthQueryPortableConditionParameter::u64("limit", 1).unwrap(),
            ])
            .unwrap(),
            domain::WorthQueryConditionalTrigger::DependencyChange,
        )
        .comparison(
            domain::WorthQueryComparatorRequirement::FoundationalContractEquivalence,
            domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
        )
        .artifact_policy(
            domain::WorthQueryArtifactReuseEquivalence::NotReusable,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
            domain::WorthQueryArtifactPosture::Ephemeral,
        )
        .output_relationship(domain::WorthQueryOutputRelationship::ContributesToOperationOutput)
        .finish()
    }

    let _public_facade_contract = author;
}
