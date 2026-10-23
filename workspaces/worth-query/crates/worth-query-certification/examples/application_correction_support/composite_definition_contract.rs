//! Declared Signal gate over the numeric author's predicate parameter.
use super::{
    composite_definition_model::{Change, Tick},
    schema::NumericSchema,
};
use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
    domain_computation::{WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode},
};
use worth_query_host::facade::{domain, worth_query_conditional_node};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefinitionDomain;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefinitionOperation;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefinitionFamily;
worth_query_conditional_node!(pub ReadyNode in DefinitionDomain, DefinitionOperation, DefinitionFamily => operation "numeric-definition-ready");

pub fn binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
    NumericSchema,
    Tick,
    Change,
    DefinitionDomain,
    DefinitionOperation,
    DefinitionFamily,
> {
    domain::WorthQueryApplicationConditionalOperationBinding::declare(
        Tick::reference(),
        definition().reference(),
    )
}
pub fn definition() -> domain::WorthQueryDomainOperationDefinition<
    DefinitionDomain,
    DefinitionOperation,
    DefinitionFamily,
> {
    domain::WorthQueryDomainOperationDefinition::new(
        domain::WorthQueryDomainOperationIdentity::new("numeric-definition-operation", 1),
        domain::WorthQueryDomainOperationSemanticClosure {
            parameters: domain::WorthQueryOperationParameterContract::NotRequired,
            native_projection: projection(),
            canonical_query: canonical_query(),
            collection: domain::WorthQueryOperationCollectionContract::NotCollection,
            required_capabilities: vec![],
            required_domains: vec![],
            workflow: domain::WorthQueryOperationWorkflowContract::NotRequired,
            evidence: domain::WorthQueryDomainEvidenceContract::not_required(),
            conditional_nodes: vec![node()],
            graph_reads: domain::WorthQueryOperationGraphReadContract::DeclaredDomain {
                roles: vec![domain::WorthQueryDomainOperationGraphReadRole {
                    role: "primary".into(),
                    participation:
                        domain::WorthQueryOperationGraphParticipation::PrimaryLogicalGraph,
                    access: domain::WorthQueryOperationGraphAccess::Project,
                    semantic_reads: vec![projection()],
                }],
            },
            decision_facts: domain::WorthQueryOperationDecisionFactContract::NotRequired,
            touches: domain::WorthQueryOperationTouchContract::NotRequired,
            effects: domain::WorthQueryOperationEffectContract::NotRequired,
            invariants: domain::WorthQueryOperationInvariantContract::NotRequired,
            invariant_execution: domain::WorthQueryInvariantExecutionContract::NotRequired,
            replay: domain::WorthQueryOperationReplayContract::ReExecutable,
            aftermath: None,
            lineage: domain::WorthQueryOperationLineageContract::NotRequired,
            promotion: domain::WorthQueryOperationPromotionContract::NotRequired,
            publication: domain::WorthQueryOperationPublicationContract::DerivedProjection {
                projection_role: domain::WorthQueryOperationProjectionRole::new("definition")
                    .unwrap(),
            },
            projection_consumption:
                domain::WorthQueryOperationProjectionConsumptionContract::QueryReadAuthority,
            terminal: domain::WorthQueryOperationTerminalContract {
                result_states: vec![domain::WorthQueryOperationResultState::Ready],
                failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
            },
            cost: domain::WorthQueryOperationCostContract {
                lookup: domain::WorthQueryOperationCostClass::Constant,
                execution: domain::WorthQueryOperationCostClass::Constant,
                result_width: domain::WorthQueryOperationCostClass::Constant,
            },
            resources: resources(),
            support: support(),
            lowering: domain::WorthQueryOperationLoweringContract {
                family: "numeric-definition.v1".into(),
                deterministic: true,
            },
        },
    )
}
fn node() -> domain::WorthQueryPortableConditionalNodeDeclaration {
    let dependency = domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        aspect(),
        mask(),
        domain::AspectBinding::EntityField {
            field: domain::FieldKey::new("NumberFacts").unwrap(),
        },
        domain::WorthQuerySemanticLocality::SourceRecord,
        [domain::AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap();
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        "numeric-definition-ready",
        domain::WorthQueryConditionalNodeRole::OperationGate,
    )
    .dependencies([dependency])
    .outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput {
        projection_role: domain::WorthQueryOperationProjectionRole::new("definition").unwrap(),
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
        domain::WorthQueryArtifactReuseEquivalence::NotReusable,
        domain::WorthQueryMaintenancePosture::Temporal,
        domain::WorthQueryArtifactPosture::Ephemeral,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::IsOperationOutput)
    .finish()
    .unwrap()
}
fn aspect() -> domain::AspectContract {
    let fields = [
        ("Key", domain::ScalarAspectType::String),
        ("Primary", domain::ScalarAspectType::UInt64),
        ("Secondary", domain::ScalarAspectType::UInt64),
        ("ReadEnabled", domain::ScalarAspectType::Bool),
        ("Ceiling", domain::ScalarAspectType::UInt64),
        ("Calculated", domain::ScalarAspectType::UInt64),
    ]
    .map(|(name, scalar)| {
        domain::FieldDeclaration::new(
            domain::FieldKey::new(name).unwrap(),
            scalar,
            domain::FieldRequirement::Required,
            domain::AbsenceLaw::Required,
            domain::AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap()
    });
    domain::AspectContract::struct_aspect(
        domain::AspectKey::new("NumberFacts").unwrap(),
        domain::AspectIdentity(0x91800103),
        domain::AspectContractRevision(1),
        domain::StructAspectShape::new(fields).unwrap(),
    )
}
fn mask() -> domain::AspectMask<domain::ProjectionMask> {
    domain::AspectMask::new([domain::CanonicalFieldPath::single(
        domain::FieldKey::new("Primary").unwrap(),
    )])
}
fn projection() -> domain::WorthQueryOperationNativeProjectionContract {
    domain::WorthQueryOperationNativeProjectionContract::new(aspect(), mask()).unwrap()
}
fn canonical_query() -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new("Number").unwrap())
        .project(AspectFieldSelector::new("NumberFacts", "Primary").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("NumberFacts", "Primary", "definition").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
fn resources() -> domain::WorthQueryExecutionResourceContract {
    let envelope = domain::WorthQueryExecutionResourceEnvelope::bounded(
        1_000,
        1_000,
        WorthQueryExecutionMode::Synchronous,
        WorthQueryCancellationSafePointFamily::new(domain::APPLICATION_EXECUTION_SAFE_POINT_FAMILY)
            .unwrap(),
    );
    domain::WorthQueryExecutionResourceContract::declared([
        domain::WorthQueryExecutionStrategyContract::new(
            domain::WorthQueryExecutionStrategyName::new("numeric-definition-temporal").unwrap(),
            envelope,
            domain::WorthQueryExecutionProviderRequirements::new(
                domain::WorthQueryExecutionProviderFamily::new(
                    domain::APPLICATION_EXECUTION_PROVIDER_FAMILY,
                )
                .unwrap(),
                domain::WorthQueryExecutionAccessProductFamily::new(
                    domain::APPLICATION_EXECUTION_ACCESS_PRODUCT_FAMILY,
                )
                .unwrap(),
                domain::WorthQueryExecutionAllocatorFamily::new(
                    domain::APPLICATION_EXECUTION_ALLOCATOR_FAMILY,
                )
                .unwrap(),
            ),
        ),
    ])
    .unwrap()
}
fn support() -> domain::WorthQueryOperationSupportRequirements {
    let yes = domain::WorthQuerySupportRequirement::Required;
    let no = domain::WorthQuerySupportRequirement::NotRequired;
    domain::WorthQueryOperationSupportRequirements {
        live: yes,
        continuation: no,
        async_result_state: no,
        recovery: no,
        inspection: no,
        projection_consumption: yes,
        dependency_impact: yes,
        sharing: yes,
        invalidation: yes,
        collection_delivery: no,
        conditional_evaluation: yes,
        conditional_comparator: yes,
        conditional_trigger: yes,
        conditional_temporal_or_on_demand: yes,
    }
}
