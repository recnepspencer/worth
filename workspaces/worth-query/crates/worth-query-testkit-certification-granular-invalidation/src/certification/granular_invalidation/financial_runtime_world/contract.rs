pub use worth_query_host::facade::declaration::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode,
};
pub use worth_query_host::facade::declaration::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};
pub use worth_query_host::facade::domain::{
    AspectBinding, AspectContract, AuthoritativeAspectChangeKind, FieldKey,
};
pub use worth_query_host::facade::{domain, worth_query_conditional_node};

pub use super::schema::{ExecuteFinancial, FinancialHostSchema, FinancialInput};

#[path = "contract/aspects.rs"]
pub mod aspects;
#[path = "contract/curve.rs"]
pub mod curve;
pub use aspects::portfolio_contract;
pub use aspects::{
    audit_contract, curve_contract, field_mask, price_contract, projection_contract, risk_contract,
    volatility_contract,
};
pub use curve::curve_record_node;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FinancialDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FinancialOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FinancialFamily;
pub struct QuoteToleranceComparatorFamily;

impl domain::WorthQueryComparatorFamily for QuoteToleranceComparatorFamily {
    const PORTABLE_IDENTITY: &'static str = "worth.query.financial.quote-tolerance-5";
}

worth_query_conditional_node!(
    pub CurveRiskNode in FinancialDomain, FinancialOperation, FinancialFamily
    => operation "curve-risk"
);
worth_query_conditional_node!(
    pub QuoteRiskNode in FinancialDomain, FinancialOperation, FinancialFamily
    => operation "quote-risk"
);
worth_query_conditional_node!(
    pub PortfolioRiskNode in FinancialDomain, FinancialOperation, FinancialFamily
    => operation "portfolio-risk"
);
worth_query_conditional_node!(
    pub PortfolioSiblingRiskNode in FinancialDomain, FinancialOperation, FinancialFamily
    => operation "portfolio-sibling-risk"
);

pub fn conditional_binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
    FinancialHostSchema,
    ExecuteFinancial,
    FinancialInput,
    FinancialDomain,
    FinancialOperation,
    FinancialFamily,
> {
    domain::WorthQueryApplicationConditionalOperationBinding::declare(
        ExecuteFinancial::reference(),
        operation_definition().reference(),
    )
}

pub fn operation_definition(
) -> domain::WorthQueryDomainOperationDefinition<FinancialDomain, FinancialOperation, FinancialFamily>
{
    domain::WorthQueryDomainOperationDefinition::new(
        domain::WorthQueryDomainOperationIdentity::new("financial-risk", 1),
        domain::WorthQueryDomainOperationSemanticClosure {
            parameters: domain::WorthQueryOperationParameterContract::NotRequired,
            native_projection: projection_contract(risk_contract(), "RiskValueField"),
            canonical_query: canonical_query(),
            collection: domain::WorthQueryOperationCollectionContract::NotCollection,
            required_capabilities: Vec::new(),
            required_domains: Vec::new(),
            workflow: domain::WorthQueryOperationWorkflowContract::NotRequired,
            evidence: domain::WorthQueryDomainEvidenceContract::not_required(),
            conditional_nodes: vec![
                curve::curve_node(),
                financial_node(
                    "quote-risk",
                    price_contract(),
                    "PriceFacts",
                    "QuoteMidField",
                    domain::WorthQueryOutputEquivalenceRequirement::registered::<
                        QuoteToleranceComparatorFamily,
                    >(),
                ),
                portfolio_node("portfolio-risk"),
                portfolio_node("portfolio-sibling-risk"),
            ],
            graph_reads: domain::WorthQueryOperationGraphReadContract::DeclaredDomain {
                roles: vec![domain::WorthQueryDomainOperationGraphReadRole {
                    role: "primary".into(),
                    participation:
                        domain::WorthQueryOperationGraphParticipation::PrimaryLogicalGraph,
                    access: domain::WorthQueryOperationGraphAccess::Project,
                    semantic_reads: vec![
                        projection_contract(curve_contract(), "CurveZeroRateField"),
                        projection_contract(volatility_contract(), "VolatilitySurfaceField"),
                        projection_contract(audit_contract(), "AuditLabelField"),
                        projection_contract(price_contract(), "QuoteMidField"),
                        projection_contract(risk_contract(), "RiskValueField"),
                        projection_contract(portfolio_contract(), "PortfolioValueField"),
                        projection_contract(portfolio_contract(), "PortfolioDeskField"),
                        projection_contract(portfolio_contract(), "PortfolioRankField"),
                    ],
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
                projection_role: domain::WorthQueryOperationProjectionRole::new("risk").unwrap(),
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
                family: "financial-risk-courtroom-v1".into(),
                deterministic: true,
            },
        },
    )
}

pub fn financial_node(
    identity: &'static str,
    contract: AspectContract,
    aspect: &'static str,
    field: &'static str,
    output_equivalence: domain::WorthQueryOutputEquivalenceRequirement,
) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    let dependency = domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        contract,
        field_mask(field),
        AspectBinding::EntityField {
            field: FieldKey::new(aspect).unwrap(),
        },
        domain::WorthQuerySemanticLocality::SourceRecord,
        [AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap();
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        identity,
        domain::WorthQueryConditionalNodeRole::OperationGate,
    )
    .dependencies([dependency])
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
        output_equivalence,
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

pub fn portfolio_node(identity: &str) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    let contract = portfolio_contract();
    let dependencies = [
        "PortfolioValueField",
        "PortfolioDeskField",
        "PortfolioRankField",
    ]
    .map(|field| {
        domain::WorthQuerySemanticTruthDependency::new(
            domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
            contract.clone(),
            field_mask(field),
            AspectBinding::EntityField {
                field: FieldKey::new("PortfolioFacts").unwrap(),
            },
            domain::WorthQuerySemanticLocality::SourcePartition(
                worth_foundational::facade::TruthPartitionRole::new("usd-rates").unwrap(),
            ),
            [AuthoritativeAspectChangeKind::FieldSet],
        )
        .unwrap()
    });
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

pub fn canonical_query(
) -> worth_query_host::facade::declaration::canonicalization::CanonicalQueryBundle {
    let query = DetailQueryBuilder::new(RootEntityKey::new("MarketObservation").unwrap())
        .project(AspectFieldSelector::new("RiskFacts", "RiskValueField").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("RiskFacts", "RiskValueField", "risk").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}

pub fn resource_contract() -> domain::WorthQueryExecutionResourceContract {
    let envelope = domain::WorthQueryExecutionResourceEnvelope::bounded(
        1_000,
        1_000,
        WorthQueryExecutionMode::Synchronous,
        WorthQueryCancellationSafePointFamily::new(domain::APPLICATION_EXECUTION_SAFE_POINT_FAMILY)
            .unwrap(),
    );
    domain::WorthQueryExecutionResourceContract::declared([
        domain::WorthQueryExecutionStrategyContract::new(
            domain::WorthQueryExecutionStrategyName::new("financial-risk").unwrap(),
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

pub fn support_contract() -> domain::WorthQueryOperationSupportRequirements {
    let no = domain::WorthQuerySupportRequirement::NotRequired;
    let required = domain::WorthQuerySupportRequirement::Required;
    domain::WorthQueryOperationSupportRequirements {
        live: required,
        continuation: no,
        async_result_state: no,
        recovery: no,
        inspection: no,
        projection_consumption: required,
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
