use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, AspectMask, AspectValue, CanonicalF64, CanonicalFieldPath, FieldDeclaration,
    FieldKey, FieldRequirement, ProjectionMask, ScalarAspectType, StructAspectShape,
};
use worth_query::facade::domain;
use worth_relational::facade::schema::{AspectBinding, RelationalAspectChangeKind};

use super::installed_operation_fixture::{
    canonical_bundle, conditional_workspace, semantic_closure, GeometryDomain, ReadFamily,
    ReadVertex,
};

pub(super) struct Millimeters;
impl domain::WorthQueryQuantityUnit for Millimeters {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.units.millimeters";
    const VALUE_FAMILY: domain::WorthQueryQuantityValueFamily =
        domain::WorthQueryQuantityValueFamily::Float64;
}
pub(super) struct Seconds;
impl domain::WorthQueryQuantityUnit for Seconds {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.units.seconds";
    const VALUE_FAMILY: domain::WorthQueryQuantityValueFamily =
        domain::WorthQueryQuantityValueFamily::Float64;
}
pub(super) struct InvalidUnitIdentity;
impl domain::WorthQueryQuantityUnit for InvalidUnitIdentity {
    const PORTABLE_IDENTITY: &'static str = "invalid unit";
    const VALUE_FAMILY: domain::WorthQueryQuantityValueFamily =
        domain::WorthQueryQuantityValueFamily::Float64;
}
pub(super) struct ManualRefresh;
impl domain::WorthQueryOnDemandTriggerFamily for ManualRefresh {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.triggers.manual-refresh";
}
pub(super) struct GeometryTolerance;
impl domain::WorthQueryComparatorFamily for GeometryTolerance {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.comparators.geometry-tolerance";
}
pub(super) struct GeometryCondition;
impl domain::WorthQueryDomainConditionFamily for GeometryCondition {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.conditions.geometry";
}

#[test]
fn installed_conditional_meaning_binds_only_after_exact_lowering_is_present() {
    let workspace = conditional_workspace(
        "conditional-lowering-not-installed",
        node(
            "geometry",
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQuerySemanticLocality::SourceRecord,
        ),
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .expect("the fixture installs the exact Phase 10 lowering before binding");
    assert!(!bound.binding_identity().is_empty());
}

pub(super) fn representative_nodes() -> Vec<domain::WorthQueryPortableConditionalNodeDeclaration> {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let threshold_dependency = distance_dependency();
    vec![
        node(
            "aspect-filtered",
            domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
            domain::WorthQuerySemanticLocality::SourceRecord,
        ),
        conditional_node(
            "threshold",
            threshold_dependency.clone(),
            domain::WorthQueryConditionalEvaluationCondition::delta_threshold(
                threshold_dependency,
                threshold::<Millimeters>(),
            ),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        ),
        conditional_node(
            "on-demand",
            dependency.clone(),
            domain::WorthQueryConditionalEvaluationCondition::on_demand(),
            domain::WorthQueryConditionalTrigger::on_demand::<ManualRefresh>(),
            domain::WorthQueryMaintenancePosture::OnDemandOnly,
        ),
        conditional_node(
            "temporal",
            dependency.clone(),
            domain::WorthQueryConditionalEvaluationCondition::temporal(
                domain::WorthQueryTemporalCondition::IntervalNanoseconds(1_000_000),
            ),
            domain::WorthQueryConditionalTrigger::Temporal(
                domain::WorthQueryTemporalWake::MonotonicClock,
            ),
            domain::WorthQueryMaintenancePosture::Temporal,
        ),
        conditional_node(
            "domain-specific",
            dependency,
            domain::WorthQueryConditionalEvaluationCondition::domain_specific::<GeometryCondition>(
                [domain::WorthQueryPortableConditionParameter::u64("threshold", 7).unwrap()],
            )
            .unwrap(),
            domain::WorthQueryConditionalTrigger::DependencyChange,
            domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        ),
    ]
}

pub(super) fn distance_dependency() -> domain::WorthQuerySemanticTruthDependency {
    domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("model").unwrap(),
        AspectContract::scalar(
            AspectKey::new("distance").unwrap(),
            AspectIdentity(0x9140_0002),
            AspectContractRevision(1),
            ScalarAspectType::Float64,
        ),
        AspectMask::whole_aspect(),
        AspectBinding::EntityField {
            field: FieldKey::new("distance").unwrap(),
        },
        domain::WorthQuerySemanticLocality::SourceRecord,
        [RelationalAspectChangeKind::FieldSet],
    )
    .unwrap()
}

pub(super) fn node(
    identity: &str,
    comparator: domain::WorthQueryComparatorRequirement,
    locality: domain::WorthQuerySemanticLocality,
) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    let dependency = dependency(locality);
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        identity,
        domain::WorthQueryConditionalNodeRole::Computed,
    )
    .dependencies([dependency.clone()])
    .outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput {
        projection_role: domain::WorthQueryOperationProjectionRole::new("vertex").unwrap(),
    }])
    .required_context([domain::WorthQueryConditionalNodeContext::Basis])
    .evaluation(
        domain::WorthQueryConditionalEvaluationCondition::aspect_filtered([dependency]).unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
    )
    .comparison(
        comparator,
        domain::WorthQueryOutputEquivalenceRequirement::FoundationalContractEquivalence,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::ContributesToOperationOutput)
    .finish()
    .unwrap()
}

pub(super) fn resource_lifecycle_node() -> domain::WorthQueryPortableConditionalNodeDeclaration {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let output = AspectContract::scalar(
        AspectKey::new("conditional-resource-output").unwrap(),
        AspectIdentity(0x9173_0001),
        AspectContractRevision(1),
        ScalarAspectType::UInt64,
    );
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        "conditional-resource",
        domain::WorthQueryConditionalNodeRole::Computed,
    )
    .dependencies([dependency.clone()])
    .outputs([domain::WorthQueryConditionalNodeOutput::DerivedAspect {
        contract: output,
        locality: domain::WorthQuerySemanticLocality::SourceRecord,
        consequences: vec![domain::WorthQueryConditionalConsequenceRole::DerivedOnly],
    }])
    .required_context([domain::WorthQueryConditionalNodeContext::Basis])
    .evaluation(
        domain::WorthQueryConditionalEvaluationCondition::aspect_filtered([dependency]).unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
    )
    .comparison(
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQueryOutputEquivalenceRequirement::FoundationalContractEquivalence,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        domain::WorthQueryArtifactPosture::ReusableWhenEquivalent,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::IntermediateOnly)
    .finish()
    .unwrap()
}

fn conditional_node(
    identity: &str,
    dependency: domain::WorthQuerySemanticTruthDependency,
    condition: domain::WorthQueryConditionalEvaluationCondition,
    trigger: domain::WorthQueryConditionalTrigger,
    maintenance: domain::WorthQueryMaintenancePosture,
) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    conditional_node_result(identity, dependency, condition, trigger, maintenance).unwrap()
}

pub(super) fn conditional_node_result(
    identity: &str,
    dependency: domain::WorthQuerySemanticTruthDependency,
    condition: domain::WorthQueryConditionalEvaluationCondition,
    trigger: domain::WorthQueryConditionalTrigger,
    maintenance: domain::WorthQueryMaintenancePosture,
) -> Result<domain::WorthQueryPortableConditionalNodeDeclaration, &'static str> {
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        identity,
        domain::WorthQueryConditionalNodeRole::Computed,
    )
    .dependencies([dependency])
    .outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput {
        projection_role: domain::WorthQueryOperationProjectionRole::new("vertex").unwrap(),
    }])
    .required_context([domain::WorthQueryConditionalNodeContext::Snapshot])
    .evaluation(condition, trigger)
    .comparison(
        domain::WorthQueryComparatorRequirement::FoundationalContractEquivalence,
        domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::NotReusable,
        maintenance,
        domain::WorthQueryArtifactPosture::Ephemeral,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::ContributesToOperationOutput)
    .finish()
}

pub(super) fn dependency(
    locality: domain::WorthQuerySemanticLocality,
) -> domain::WorthQuerySemanticTruthDependency {
    dependency_for_role("model", locality)
}

pub(super) fn dependency_for_role(
    graph_role: &str,
    locality: domain::WorthQuerySemanticLocality,
) -> domain::WorthQuerySemanticTruthDependency {
    domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new(graph_role).unwrap(),
        identity_contract(),
        identity_mask(),
        AspectBinding::EntityField {
            field: FieldKey::new("id").unwrap(),
        },
        locality,
        [RelationalAspectChangeKind::FieldSet],
    )
    .unwrap()
}

pub(super) fn threshold<Unit: domain::WorthQueryQuantityUnit>() -> domain::WorthQueryDeltaThreshold
{
    domain::WorthQueryDeltaThreshold::new::<Unit>(
        AspectValue::Float64(CanonicalF64::from_f64(0.01)),
        domain::WorthQueryDeltaComparisonDomain::AbsoluteDifference,
        domain::WorthQueryThresholdBoundary::Inclusive,
    )
    .unwrap()
}

pub(super) fn definition(
    nodes: Vec<domain::WorthQueryPortableConditionalNodeDeclaration>,
) -> domain::WorthQueryDomainOperationDefinition<GeometryDomain, ReadVertex, ReadFamily> {
    let mut semantics = semantic_closure(
        canonical_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::Required,
        true,
    );
    semantics.conditional_nodes = nodes;
    domain::WorthQueryDomainOperationDefinition::new(
        domain::WorthQueryDomainOperationIdentity::new("conditional-read", 1),
        semantics,
    )
}

pub(super) fn canonical_identity(
    nodes: Vec<domain::WorthQueryPortableConditionalNodeDeclaration>,
) -> String {
    definition(nodes)
        .into_portable()
        .canonical_identity()
        .to_string()
}

pub(super) fn identity_contract() -> AspectContract {
    let field = FieldDeclaration::new(
        FieldKey::new("id").unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap();
    AspectContract::struct_aspect(
        AspectKey::new("identity").unwrap(),
        AspectIdentity(0x9140_0001),
        AspectContractRevision(1),
        StructAspectShape::new([field]).unwrap(),
    )
}

pub(super) fn identity_mask() -> AspectMask<ProjectionMask> {
    AspectMask::new([CanonicalFieldPath::single(FieldKey::new("id").unwrap())])
}
