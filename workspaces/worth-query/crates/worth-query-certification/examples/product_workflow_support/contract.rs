use domain::{
    AbsenceLaw, AspectBinding, AspectContract, AspectContractRevision, AspectEvolutionPolicy,
    AspectIdentity, AspectKey, AspectMask, AuthoritativeAspectChangeKind, FieldDeclaration,
    FieldKey, FieldRequirement, ProjectionMask, ScalarAspectType, StructAspectShape,
};
use worth_query_host::facade::declaration::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode,
};
use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};
use worth_query_host::facade::{domain, worth_query_conditional_node};

use super::schema::{ExecuteTemporal, TemporalHostSchema, TemporalInput};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporalDomain;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporalDomainOperation;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporalDomainFamily;

worth_query_conditional_node!(
    pub TemporalReadyNode in TemporalDomain, TemporalDomainOperation, TemporalDomainFamily
    => operation "temporal-ready"
);

pub fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
    TemporalHostSchema,
    ExecuteTemporal,
    TemporalInput,
    TemporalDomain,
    TemporalDomainOperation,
    TemporalDomainFamily,
> {
    domain::WorthQueryApplicationConditionalOperationBinding::declare(
        ExecuteTemporal::reference(),
        operation_definition().reference(),
    )
}

pub fn operation_definition() -> domain::WorthQueryDomainOperationDefinition<
    TemporalDomain,
    TemporalDomainOperation,
    TemporalDomainFamily,
> {
    domain::WorthQueryDomainOperationDefinition::new(
        domain::WorthQueryDomainOperationIdentity::new("temporal-operation", 1),
        domain::WorthQueryDomainOperationSemanticClosure {
            parameters: domain::WorthQueryOperationParameterContract::NotRequired,
            native_projection: native_projection(),
            canonical_query: canonical_query(),
            collection: domain::WorthQueryOperationCollectionContract::NotCollection,
            required_capabilities: Vec::new(),
            required_domains: Vec::new(),
            workflow: domain::WorthQueryOperationWorkflowContract::NotRequired,
            evidence: domain::WorthQueryDomainEvidenceContract::not_required(),
            conditional_nodes: vec![temporal_node()],
            graph_reads: domain::WorthQueryOperationGraphReadContract::DeclaredDomain {
                roles: vec![domain::WorthQueryDomainOperationGraphReadRole {
                    role: "primary".into(),
                    participation:
                        domain::WorthQueryOperationGraphParticipation::PrimaryLogicalGraph,
                    access: domain::WorthQueryOperationGraphAccess::Project,
                    semantic_reads: vec![gate_projection()],
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
                projection_role: domain::WorthQueryOperationProjectionRole::new("intent").unwrap(),
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
            resources: resource_contract(),
            support: support_contract(),
            lowering: domain::WorthQueryOperationLoweringContract {
                family: "temporal-host-courtroom-v1".into(),
                deterministic: true,
            },
        },
    )
}

fn temporal_node() -> domain::WorthQueryPortableConditionalNodeDeclaration {
    let dependency = domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        gate_contract(),
        gate_mask(),
        AspectBinding::EntityField {
            field: FieldKey::new("IntentFacts").unwrap(),
        },
        domain::WorthQuerySemanticLocality::SourceRecord,
        [AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap();
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        "temporal-ready",
        domain::WorthQueryConditionalNodeRole::OperationGate,
    )
    .dependencies([dependency])
    .outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput {
        projection_role: domain::WorthQueryOperationProjectionRole::new("intent").unwrap(),
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

fn gate_contract() -> AspectContract {
    let required = |name: &str, family| {
        FieldDeclaration::new(
            FieldKey::new(name).unwrap(),
            family,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap()
    };
    AspectContract::struct_aspect(
        AspectKey::new("IntentFacts").unwrap(),
        AspectIdentity(0x9161_1044),
        AspectContractRevision(1),
        StructAspectShape::new([
            required("IntentDueField", ScalarAspectType::UInt64),
            required("IntentEffectField", ScalarAspectType::String),
            required("IntentGateField", ScalarAspectType::String),
            required("IntentIdentityField", ScalarAspectType::String),
            required("IntentInputField", ScalarAspectType::String),
            required("IntentLifecycleField", ScalarAspectType::String),
            required("IntentRevisionField", ScalarAspectType::UInt64),
        ])
        .unwrap(),
    )
}

fn gate_mask() -> AspectMask<ProjectionMask> {
    AspectMask::new([domain::CanonicalFieldPath::single(
        FieldKey::new("IntentGateField").unwrap(),
    )])
}

fn native_projection() -> domain::WorthQueryOperationNativeProjectionContract {
    let field = FieldDeclaration::new(
        FieldKey::new("IntentIdentityField").unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap();
    domain::WorthQueryOperationNativeProjectionContract::new(
        AspectContract::struct_aspect(
            AspectKey::new("IntentFacts").unwrap(),
            AspectIdentity(0x9160_0002),
            AspectContractRevision(1),
            StructAspectShape::new([field]).unwrap(),
        ),
        AspectMask::<ProjectionMask>::whole_aspect(),
    )
    .unwrap()
}

fn gate_projection() -> domain::WorthQueryOperationNativeProjectionContract {
    domain::WorthQueryOperationNativeProjectionContract::new(gate_contract(), gate_mask()).unwrap()
}

fn canonical_query() -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new("TemporalIntent").unwrap())
        .project(AspectFieldSelector::new("IntentFacts", "IntentIdentityField").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new("IntentFacts", "IntentIdentityField", "identity")
                .unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}

fn resource_contract() -> domain::WorthQueryExecutionResourceContract {
    let envelope = domain::WorthQueryExecutionResourceEnvelope::bounded(
        1_000,
        1_000,
        WorthQueryExecutionMode::Synchronous,
        WorthQueryCancellationSafePointFamily::new(domain::APPLICATION_EXECUTION_SAFE_POINT_FAMILY)
            .unwrap(),
    );
    domain::WorthQueryExecutionResourceContract::declared([
        domain::WorthQueryExecutionStrategyContract::new(
            domain::WorthQueryExecutionStrategyName::new("application-temporal").unwrap(),
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

fn support_contract() -> domain::WorthQueryOperationSupportRequirements {
    let no = domain::WorthQuerySupportRequirement::NotRequired;
    let required = domain::WorthQuerySupportRequirement::Required;
    domain::WorthQueryOperationSupportRequirements {
        live: required,
        continuation: no,
        async_result_state: no,
        recovery: no,
        inspection: no,
        projection_consumption: domain::WorthQuerySupportRequirement::Required,
        dependency_impact: required,
        sharing: required,
        invalidation: required,
        collection_delivery: no,
        conditional_evaluation: required,
        conditional_comparator: required,
        conditional_trigger: required,
        conditional_temporal_or_on_demand: required,
    }
}
