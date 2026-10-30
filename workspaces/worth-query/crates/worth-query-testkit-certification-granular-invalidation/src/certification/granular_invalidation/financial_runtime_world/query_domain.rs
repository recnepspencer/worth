pub use std::sync::OnceLock;

pub use worth_query::facade::{domain, read};
pub use worth_query_host::facade::domain::APPLICATION_EXECUTION_SAFE_POINT_FAMILY;

pub use super::contract::{FinancialDomain, FinancialFamily, FinancialOperation};

#[path = "query_domain/portfolio.rs"]
pub mod portfolio;

impl domain::WorthQueryDomainEntryMarker for FinancialDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.certification.financial-risk"
    }

    fn display_name(&self) -> &'static str {
        "Financial risk"
    }

    fn required_capability_families(&self) -> &'static [domain::WorthQueryCapabilityFamily] {
        &[]
    }
}

impl domain::WorthQueryExecutableDomainOperation<FinancialDomain, FinancialFamily>
    for FinancialOperation
{
    type Input = ();
    type Output = read::WorthQueryReadCompletion;
    type Publication = domain::WorthQueryPublishingOperation;
    type Execution = domain::WorthQueryDirectOperation;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinancialQueryProfile {
    CurveRisk,
    CurveRecordRisk,
    QuoteRisk,
    OrderedPortfolio,
}

pub fn package(profile: FinancialQueryProfile) -> domain::WorthQueryDomainPackage<FinancialDomain> {
    domain::WorthQueryDomainPackage::declare(
        FinancialDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.certification").unwrap(),
            domain::WorthQueryDomainIdentityName::new("financial-risk").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(operation_definition(profile))
    .operation_graph_participation::<FinancialOperation, FinancialFamily, super::query::FinancialGraph>(
        "primary",
    )
}

pub fn operation_definition(
    profile: FinancialQueryProfile,
) -> domain::WorthQueryDomainOperationDefinition<FinancialDomain, FinancialOperation, FinancialFamily>
{
    let declared = super::contract::operation_definition();
    let mut semantics = declared.semantics().clone();
    let node_identity = match profile {
        FinancialQueryProfile::CurveRisk => "curve-risk",
        FinancialQueryProfile::CurveRecordRisk => "curve-record-risk",
        FinancialQueryProfile::QuoteRisk => "quote-risk",
        FinancialQueryProfile::OrderedPortfolio => "portfolio-risk",
    };
    let node = if profile == FinancialQueryProfile::CurveRecordRisk {
        super::contract::curve_record_node()
    } else {
        semantics
            .conditional_nodes
            .iter()
            .find(|node| node.identity() == node_identity)
            .cloned()
            .expect("the financial profile must name an installed conditional node")
    };
    semantics.conditional_nodes = vec![node];
    if profile == FinancialQueryProfile::OrderedPortfolio {
        semantics.native_projection = domain::WorthQueryOperationNativeProjectionContract::new(
            super::contract::portfolio_contract(),
            worth_foundational::facade::AspectMask::new([
                worth_foundational::facade::CanonicalFieldPath::single(
                    worth_foundational::facade::FieldKey::new("PortfolioValueField").unwrap(),
                ),
                worth_foundational::facade::CanonicalFieldPath::single(
                    worth_foundational::facade::FieldKey::new("PortfolioDeskField").unwrap(),
                ),
                worth_foundational::facade::CanonicalFieldPath::single(
                    worth_foundational::facade::FieldKey::new("PortfolioRankField").unwrap(),
                ),
            ]),
        )
        .unwrap();
        semantics.canonical_query = portfolio::canonical_query();
        semantics.collection = portfolio::collection_contract();
        semantics.support.continuation = domain::WorthQuerySupportRequirement::Required;
    }
    domain::WorthQueryDomainOperationDefinition::new(declared.identity().clone(), semantics)
}

#[derive(Clone, Copy)]
pub struct FinancialExecutor(pub FinancialQueryProfile);

impl domain::WorthQueryDomainOperationExecutor<FinancialDomain, FinancialOperation, FinancialFamily>
    for FinancialExecutor
{
    const LOWERING_FAMILY: &'static str = "financial-risk-courtroom-v1";
    const DETERMINISTIC: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::Constant;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::Constant;

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        crate::facade::certification::query_runtime_world::resource_support()
    }

    fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
        Some(match self.0 {
            FinancialQueryProfile::CurveRisk
            | FinancialQueryProfile::CurveRecordRisk
            | FinancialQueryProfile::QuoteRisk => read_declaration(),
            FinancialQueryProfile::OrderedPortfolio => portfolio::read_declaration(),
        })
    }

    fn execute(
        &self,
        _: (),
        context: &domain::WorthQueryOperationExecutionContext<'_>,
        workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
        domain::WorthQueryOperationExecutorFailure,
    > {
        Ok(domain::WorthQueryOperationExecutionMaterial::new(
            context.execute_installed_read(workspace)?,
            domain::WorthQueryOperationResultState::Ready,
        ))
    }
}

pub fn read_declaration() -> &'static read::WorthQueryReadDeclaration {
    static DECLARATION: OnceLock<read::WorthQueryReadDeclaration> = OnceLock::new();
    DECLARATION.get_or_init(|| {
        read::declare(|builder| {
            builder.local_detail(
                "MarketObservation",
                schema_view(),
                |query| query.project(field()),
                |shape| shape.field(result_field()),
            )
        })
        .expect("the financial risk read must remain canonical")
    })
}

pub fn schema_view() -> read::QuerySchemaView {
    read::QuerySchemaView::new(
        "financial-primary",
        [read::SchemaFieldView::new(
            read::AspectName::new("RiskFacts").unwrap(),
            read::FieldName::new("RiskValueField").unwrap(),
            read::ScalarAspectType::UInt64,
        )],
        [],
    )
}

pub fn field() -> worth_query_host::facade::declaration::authoring::AspectFieldSelector {
    worth_query_host::facade::declaration::authoring::AspectFieldSelector::new(
        "RiskFacts",
        "RiskValueField",
    )
    .unwrap()
}

pub fn result_field() -> worth_query_host::facade::declaration::authoring::AuthoredResultShapeField
{
    worth_query_host::facade::declaration::authoring::AuthoredResultShapeField::new(
        "RiskFacts",
        "RiskValueField",
        "risk",
    )
    .unwrap()
}

pub fn resource_request() -> domain::WorthQueryExecutionResourceRequest {
    domain::WorthQueryExecutionResourceRequest::bounded(
        1_000,
        1_000,
        domain::WorthQueryCancellationSafePointFamily::new(APPLICATION_EXECUTION_SAFE_POINT_FAMILY)
            .unwrap(),
    )
}
