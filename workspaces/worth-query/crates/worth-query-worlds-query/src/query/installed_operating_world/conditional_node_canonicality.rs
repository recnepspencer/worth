pub use worth_foundational::facade::{AspectIdentity, AspectValue, CanonicalF64};
pub use worth_query::facade::domain;
pub use worth_relational::facade::schema::{AspectBinding, RelationalAspectChangeKind};

pub use super::conditional_node_contract::{
    canonical_identity, conditional_node_result, definition, dependency, identity_contract,
    identity_mask, representative_nodes, threshold, GeometryCondition, GeometryTolerance,
    InvalidUnitIdentity, ManualRefresh, Millimeters, Seconds,
};

pub struct AlternateRefresh;
impl domain::WorthQueryOnDemandTriggerFamily for AlternateRefresh {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.triggers.alternate-refresh";
}

pub fn assert_drift(
    left: domain::WorthQueryPortableConditionalNodeDeclaration,
    right: domain::WorthQueryPortableConditionalNodeDeclaration,
) {
    assert_ne!(
        canonical_identity(vec![left]),
        canonical_identity(vec![right])
    );
}

pub fn aspect_condition() -> domain::WorthQueryConditionalEvaluationCondition {
    domain::WorthQueryConditionalEvaluationCondition::aspect_filtered([dependency(
        domain::WorthQuerySemanticLocality::SourceRecord,
    )])
    .unwrap()
}

pub fn on_demand_node<Owner: domain::WorthQueryOnDemandTriggerFamily>(
) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    authored_node(
        domain::WorthQueryConditionalEvaluationCondition::on_demand(),
        domain::WorthQueryConditionalTrigger::on_demand::<Owner>(),
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
        domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
        domain::WorthQueryMaintenancePosture::OnDemandOnly,
        domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
        domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
    )
}

pub fn temporal_node(duration: u64) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    authored_node(
        domain::WorthQueryConditionalEvaluationCondition::temporal(
            domain::WorthQueryTemporalCondition::IntervalNanoseconds(duration),
        ),
        domain::WorthQueryConditionalTrigger::Temporal(
            domain::WorthQueryTemporalWake::MonotonicClock,
        ),
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
        domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
        domain::WorthQueryMaintenancePosture::Temporal,
        domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
        domain::WorthQueryOutputRelationship::ContributesToOperationOutput,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn authored_node(
    condition: domain::WorthQueryConditionalEvaluationCondition,
    trigger: domain::WorthQueryConditionalTrigger,
    comparator: domain::WorthQueryComparatorRequirement,
    output_equivalence: domain::WorthQueryOutputEquivalenceRequirement,
    reuse: domain::WorthQueryArtifactReuseEquivalence,
    maintenance: domain::WorthQueryMaintenancePosture,
    artifact: domain::WorthQueryArtifactPosture,
    relationship: domain::WorthQueryOutputRelationship,
) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        "drift",
        domain::WorthQueryConditionalNodeRole::Computed,
    )
    .dependencies([dependency])
    .outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput {
        projection_role: domain::WorthQueryOperationProjectionRole::new("vertex").unwrap(),
    }])
    .required_context([domain::WorthQueryConditionalNodeContext::Basis])
    .evaluation(condition, trigger)
    .comparison(comparator, output_equivalence)
    .artifact_policy(reuse, maintenance, artifact)
    .output_relationship(relationship)
    .finish()
    .unwrap()
}
