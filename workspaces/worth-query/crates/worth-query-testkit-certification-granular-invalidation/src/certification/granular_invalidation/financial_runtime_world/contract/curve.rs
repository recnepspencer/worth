pub use worth_query_host::facade::domain;
pub use worth_query_host::facade::domain::{
    AspectBinding, AuthoritativeAspectChangeKind, FieldKey,
};

pub fn curve_node() -> domain::WorthQueryPortableConditionalNodeDeclaration {
    let mut dependencies = curve_localities()
        .into_iter()
        .map(|locality| dependency(super::curve_contract(), "CurveZeroRateField", locality))
        .collect::<Vec<_>>();
    dependencies.push(dependency(
        super::volatility_contract(),
        "VolatilitySurfaceField",
        domain::WorthQuerySemanticLocality::SourceRecord,
    ));
    dependencies.push(dependency(
        super::audit_contract(),
        "AuditLabelField",
        domain::WorthQuerySemanticLocality::SourceRecord,
    ));
    conditional_node("curve-risk", dependencies)
}

pub fn curve_record_node() -> domain::WorthQueryPortableConditionalNodeDeclaration {
    conditional_node(
        "curve-record-risk",
        vec![dependency(
            super::curve_contract(),
            "CurveZeroRateField",
            domain::WorthQuerySemanticLocality::SourceRecord,
        )],
    )
}

pub fn conditional_node(
    identity: &'static str,
    dependencies: Vec<domain::WorthQuerySemanticTruthDependency>,
) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        identity,
        domain::WorthQueryConditionalNodeRole::OperationGate,
    )
    .dependencies(dependencies)
    .outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput {
        projection_role: domain::WorthQueryOperationProjectionRole::new("risk").unwrap(),
    }])
    .required_context([
        domain::WorthQueryConditionalNodeContext::Snapshot,
        domain::WorthQueryConditionalNodeContext::OperationInput,
    ])
    .evaluation(
        domain::WorthQueryConditionalEvaluationCondition::temporal(
            domain::WorthQueryTemporalCondition::AfterNanoseconds(1),
        ),
        domain::WorthQueryConditionalTrigger::Temporal(
            domain::WorthQueryTemporalWake::MonotonicClock,
        ),
    )
    .comparison(
        domain::WorthQueryComparatorRequirement::FoundationalContractEquivalence,
        domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
        domain::WorthQueryMaintenancePosture::Temporal,
        domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::IsOperationOutput)
    .finish()
    .unwrap()
}

pub fn curve_localities() -> [domain::WorthQuerySemanticLocality; 3] {
    [
        domain::WorthQuerySemanticLocality::SourceRecord,
        domain::WorthQuerySemanticLocality::SourcePartition(
            worth_foundational::facade::TruthPartitionRole::new("usd-rates").unwrap(),
        ),
        domain::WorthQuerySemanticLocality::WholeLogicalGraph,
    ]
}

pub fn dependency(
    contract: domain::AspectContract,
    field: &'static str,
    locality: domain::WorthQuerySemanticLocality,
) -> domain::WorthQuerySemanticTruthDependency {
    domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        contract.clone(),
        super::field_mask(field),
        AspectBinding::EntityField {
            field: FieldKey::new(contract.key().as_str()).unwrap(),
        },
        locality,
        [AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap()
}
