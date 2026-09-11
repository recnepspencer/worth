use worth_query_host::facade::declaration::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
};
use worth_query_host::facade::{declaration, primary_graph};
use worth_query_host::facade::{
    worth_query_application_query, worth_query_application_schema, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_operation, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_portable_type, worth_query_principal_binding,
    worth_query_relation,
};

worth_query_application_schema! {
    pub schema FinancialHostSchema {
        owner: granular_financial_courtroom,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(ExternalMapping::reference())
                .entity(Principal::reference())
                .entity(MarketObservation::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalFacts::reference())
                .aspect(MarketObservation::reference(), MarketIdentity::reference())
                .aspect(MarketObservation::reference(), CurveFacts::reference())
                .aspect(MarketObservation::reference(), VolatilityFacts::reference())
                .aspect(MarketObservation::reference(), PriceFacts::reference())
                .aspect(MarketObservation::reference(), RiskFacts::reference())
                .aspect(MarketObservation::reference(), PortfolioFacts::reference())
                .aspect(MarketObservation::reference(), AuditFacts::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .field(MarketObservation::reference(), MarketIdentityField::reference())
                .field(MarketObservation::reference(), MarketRevisionField::reference())
                .field(MarketObservation::reference(), MarketDueField::reference())
                .field(MarketObservation::reference(), MarketLifecycleField::reference())
                .field(MarketObservation::reference(), MarketInputField::reference())
                .field(MarketObservation::reference(), MarketGateField::reference())
                .field(MarketObservation::reference(), CurvePartitionField::reference())
                .field(MarketObservation::reference(), CurveDetailField::reference())
                .field(MarketObservation::reference(), CurveZeroRateField::reference())
                .field(MarketObservation::reference(), VolatilitySurfaceField::reference())
                .field(MarketObservation::reference(), QuoteMidField::reference())
                .field(MarketObservation::reference(), RiskValueField::reference())
                .field(MarketObservation::reference(), PortfolioValueField::reference())
                .field(MarketObservation::reference(), PortfolioDeskField::reference())
                .field(MarketObservation::reference(), PortfolioRankField::reference())
                .field(MarketObservation::reference(), AuditLabelField::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .principal_binding(FinancialPrincipalBinding::reference())
                .operation(
                    ExecuteFinancial::reference().definition().no_external_effect().no_aftermath().finish(),
                )
                .operation(
                    AmendMarket::reference().definition().no_external_effect().no_aftermath().finish(),
                )
                .operation_decision_fact_budget(ExecuteFinancial::reference(), 4)
                .operation_projection_work_budget(ExecuteFinancial::reference(), 16)
                .operation_read_field(ExecuteFinancial::reference(), MarketIdentityField::reference())
                .operation_read_field(ExecuteFinancial::reference(), MarketRevisionField::reference())
                .operation_read_field(ExecuteFinancial::reference(), MarketLifecycleField::reference())
                .operation_read_field(ExecuteFinancial::reference(), RiskValueField::reference())
                .operation_write(ExecuteFinancial::reference(), MarketRevisionField::reference())
                .operation_write(ExecuteFinancial::reference(), MarketLifecycleField::reference())
                .operation_write(ExecuteFinancial::reference(), RiskValueField::reference())
                .operation_decision_fact_budget(AmendMarket::reference(), 10)
                .operation_projection_work_budget(AmendMarket::reference(), 16)
                .operation_read_field(AmendMarket::reference(), MarketRevisionField::reference())
                .operation_read_field(AmendMarket::reference(), MarketDueField::reference())
                .operation_read_field(AmendMarket::reference(), MarketLifecycleField::reference())
                .operation_read_field(AmendMarket::reference(), MarketGateField::reference())
                .operation_read_field(AmendMarket::reference(), MarketInputField::reference())
                .operation_read_field(AmendMarket::reference(), CurveZeroRateField::reference())
                .operation_read_field(AmendMarket::reference(), QuoteMidField::reference())
                .operation_read_field(AmendMarket::reference(), RiskValueField::reference())
                .operation_read_field(AmendMarket::reference(), PortfolioValueField::reference())
                .operation_read_field(AmendMarket::reference(), PortfolioDeskField::reference())
                .operation_read_field(AmendMarket::reference(), PortfolioRankField::reference())
                .operation_write(AmendMarket::reference(), MarketRevisionField::reference())
                .operation_write(AmendMarket::reference(), MarketDueField::reference())
                .operation_write(AmendMarket::reference(), MarketLifecycleField::reference())
                .operation_write(AmendMarket::reference(), MarketGateField::reference())
                .operation_write(AmendMarket::reference(), MarketInputField::reference())
                .operation_write(AmendMarket::reference(), CurveZeroRateField::reference())
                .operation_write(AmendMarket::reference(), QuoteMidField::reference())
                .operation_write(AmendMarket::reference(), RiskValueField::reference())
                .operation_write(AmendMarket::reference(), PortfolioValueField::reference())
                .operation_write(AmendMarket::reference(), PortfolioDeskField::reference())
                .operation_write(AmendMarket::reference(), PortfolioRankField::reference())
                .application_query(financial_intent_query_definition())
        }
    }
}

worth_query_entity!(pub ExternalMapping for FinancialHostSchema);
worth_query_entity!(pub Principal for FinancialHostSchema);
worth_query_entity!(pub MarketObservation for FinancialHostSchema);
worth_query_aspect!(pub ExternalIdentity for FinancialHostSchema, ExternalMapping; identity = AspectIdentity(0x9161101b), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PrincipalFacts for FinancialHostSchema, Principal; identity = AspectIdentity(0x9161101c), revision = AspectContractRevision(1),);
worth_query_aspect!(pub MarketIdentity for FinancialHostSchema, MarketObservation; identity = AspectIdentity(0x9161101d), revision = AspectContractRevision(1),);
worth_query_aspect!(pub CurveFacts for FinancialHostSchema, MarketObservation; identity = AspectIdentity(0x9161101e), revision = AspectContractRevision(1),);
worth_query_aspect!(pub VolatilityFacts for FinancialHostSchema, MarketObservation; identity = AspectIdentity(0x9161101f), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PriceFacts for FinancialHostSchema, MarketObservation; identity = AspectIdentity(0x91611020), revision = AspectContractRevision(1),);
worth_query_aspect!(pub RiskFacts for FinancialHostSchema, MarketObservation; identity = AspectIdentity(0x91611021), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PortfolioFacts for FinancialHostSchema, MarketObservation; identity = AspectIdentity(0x91611022), revision = AspectContractRevision(1),);
worth_query_aspect!(pub AuditFacts for FinancialHostSchema, MarketObservation; identity = AspectIdentity(0x91611023), revision = AspectContractRevision(1),);
worth_query_field!(pub ExternalIdentityField for FinancialHostSchema, ExternalMapping, ExternalIdentity: declaration::authentication::WorthQueryExternalPrincipalIdentity => declaration::authentication::WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for FinancialHostSchema, ExternalMapping, ExternalIdentity: declaration::authentication::WorthQueryPrincipalMappingStatus => declaration::authentication::WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for FinancialHostSchema, Principal, PrincipalFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub MarketIdentityField for FinancialHostSchema, MarketObservation, MarketIdentity: String => declaration::application_schema::StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub MarketRevisionField for FinancialHostSchema, MarketObservation, MarketIdentity: u64 => declaration::application_schema::U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub MarketDueField for FinancialHostSchema, MarketObservation, MarketIdentity: u64 => declaration::application_schema::U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub MarketLifecycleField for FinancialHostSchema, MarketObservation, MarketIdentity: String => declaration::application_schema::StringApplicationValueBinding, read_write, equality);
worth_query_field!(pub MarketInputField for FinancialHostSchema, MarketObservation, MarketIdentity: String => declaration::application_schema::StringApplicationValueBinding, read_write, equality);
worth_query_field!(pub MarketGateField for FinancialHostSchema, MarketObservation, MarketIdentity: String => declaration::application_schema::StringApplicationValueBinding, read_write, equality);
worth_query_field!(pub CurvePartitionField for FinancialHostSchema, MarketObservation, CurveFacts: String => declaration::application_schema::StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub CurveDetailField for FinancialHostSchema, MarketObservation, CurveFacts: String => declaration::application_schema::StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub CurveZeroRateField for FinancialHostSchema, MarketObservation, CurveFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub VolatilitySurfaceField for FinancialHostSchema, MarketObservation, VolatilityFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub QuoteMidField for FinancialHostSchema, MarketObservation, PriceFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub RiskValueField for FinancialHostSchema, MarketObservation, RiskFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub PortfolioValueField for FinancialHostSchema, MarketObservation, PortfolioFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub PortfolioDeskField for FinancialHostSchema, MarketObservation, PortfolioFacts: String => declaration::application_schema::StringApplicationValueBinding, read_write, equality);
worth_query_field!(pub PortfolioRankField for FinancialHostSchema, MarketObservation, PortfolioFacts: u64 => declaration::application_schema::U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub AuditLabelField for FinancialHostSchema, MarketObservation, AuditFacts: String => declaration::application_schema::StringApplicationValueBinding, read_only, equality);
worth_query_relation!(pub MappingTarget in FinancialHostSchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub FinancialPrincipalBinding in FinancialHostSchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinancialInput(pub String);
worth_query_portable_type!(FinancialInput => "worth.query.test.certification.financial.input.v1");
worth_query_host::facade::worth_query_structured_value_binding!(pub FinancialInputBinding for FinancialInput { identity: "worth.query.test.certification.financial.input.v1" });

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmendMarketInput;
worth_query_portable_type!(AmendMarketInput => "worth.query.test.certification.financial.amend_input.v1");
worth_query_host::facade::worth_query_structured_value_binding!(pub AmendMarketInputBinding for AmendMarketInput { identity: "worth.query.test.certification.financial.amend_input.v1" });

worth_query_operation!(pub ExecuteFinancial for FinancialHostSchema, input FinancialInputBinding);
worth_query_operation_reads!(ExecuteFinancial => [MarketIdentityField, MarketRevisionField, MarketLifecycleField, RiskValueField]);
worth_query_operation_writes!(ExecuteFinancial => [MarketRevisionField, MarketLifecycleField, RiskValueField]);
worth_query_operation!(pub AmendMarket for FinancialHostSchema, input AmendMarketInputBinding);
worth_query_operation_reads!(AmendMarket => [MarketRevisionField, MarketDueField, MarketLifecycleField, MarketGateField, MarketInputField, CurveZeroRateField, QuoteMidField, RiskValueField, PortfolioValueField, PortfolioDeskField, PortfolioRankField]);
worth_query_operation_writes!(AmendMarket => [MarketRevisionField, MarketDueField, MarketLifecycleField, MarketGateField, MarketInputField, CurveZeroRateField, QuoteMidField, RiskValueField, PortfolioValueField, PortfolioDeskField, PortfolioRankField]);

pub struct FinancialIntentParameters;
pub struct IdentitySlot;
pub struct RevisionSlot;
pub struct DueSlot;
pub struct LifecycleSlot;
pub struct InputSlot;
worth_query_portable_type!(IdentitySlot => "worth.query.test.certification.financial.identity_slot.v1");
worth_query_portable_type!(RevisionSlot => "worth.query.test.certification.financial.revision_slot.v1");
worth_query_portable_type!(DueSlot => "worth.query.test.certification.financial.due_slot.v1");
worth_query_portable_type!(LifecycleSlot => "worth.query.test.certification.financial.lifecycle_slot.v1");
worth_query_portable_type!(InputSlot => "worth.query.test.certification.financial.input_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinancialIntentResult {
    pub identity: String,
    pub revision: u64,
    pub due: u64,
    pub lifecycle: String,
    pub input: String,
}
worth_query_portable_type!(FinancialIntentResult => "worth.query.test.certification.financial.intent_result.v1");
worth_query_host::facade::worth_query_structured_value_binding!(pub FinancialIntentParametersBinding for FinancialIntentParameters { identity: "FinancialIntentParameters" });
worth_query_host::facade::worth_query_structured_value_binding!(pub FinancialIntentResultBinding for FinancialIntentResult { identity: "worth.query.test.certification.financial.intent_result.v1" });

worth_query_application_query!(
    pub FinancialIntentQuery for FinancialHostSchema,
    identity "FinancialIntentQuery",
    parameters FinancialIntentParametersBinding,
    result FinancialIntentResultBinding,
    scope MarketObservation => "MarketObservation",
    name "financial_intent_query"
);

type ResultField<Slot, Field, Value, Write> = ApplicationQueryResultFieldRef<
    FinancialIntentQuery,
    Slot,
    FinancialHostSchema,
    MarketObservation,
    MarketIdentity,
    Field,
    Value,
    Write,
    declaration::application_schema::EqualityPredicate,
    declaration::application_schema::NoApplicationUnit,
>;

fn financial_intent_query_definition() -> ApplicationQueryDefinition<
    FinancialHostSchema,
    FinancialIntentQuery,
    FinancialIntentParameters,
    FinancialIntentResult,
    MarketObservation,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        FinancialHostSchema,
        FinancialIntentQuery,
        MarketObservation,
        FinancialIntentResult,
        FinancialIntentResultBinding,
    >::new(MarketObservation::reference())
    .field(identity_result())
    .field(revision_result())
    .field(due_result())
    .field(lifecycle_result())
    .field(input_result())
    .build();
    ApplicationQueryDefinitionBuilder::declare(FinancialIntentQuery::reference())
        .root(MarketObservation::reference())
        .scope(MarketObservation::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 5))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("financial intent query is canonical")
}

impl primary_graph::WorthQueryApplicationProjection<FinancialHostSchema, FinancialIntentQuery>
    for FinancialIntentResult
{
    fn project(
        row: &primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            FinancialHostSchema,
            FinancialIntentQuery,
        >,
    ) -> Result<Self, primary_graph::WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(identity_result())?,
            revision: row.field(revision_result())?,
            due: row.field(due_result())?,
            lifecycle: row.field(lifecycle_result())?,
            input: row.field(input_result())?,
        })
    }
}

fn identity_result(
) -> ResultField<IdentitySlot, MarketIdentityField, String, declaration::application_schema::ReadOnly>
{
    ApplicationQueryResultFieldRef::new("identity", MarketIdentityField::reference())
}

fn revision_result(
) -> ResultField<RevisionSlot, MarketRevisionField, u64, declaration::application_schema::ReadWrite>
{
    ApplicationQueryResultFieldRef::new("revision", MarketRevisionField::reference())
}

fn due_result(
) -> ResultField<DueSlot, MarketDueField, u64, declaration::application_schema::ReadWrite> {
    ApplicationQueryResultFieldRef::new("due", MarketDueField::reference())
}

fn lifecycle_result() -> ResultField<
    LifecycleSlot,
    MarketLifecycleField,
    String,
    declaration::application_schema::ReadWrite,
> {
    ApplicationQueryResultFieldRef::new("lifecycle", MarketLifecycleField::reference())
}

fn input_result(
) -> ResultField<InputSlot, MarketInputField, String, declaration::application_schema::ReadWrite> {
    ApplicationQueryResultFieldRef::new("input", MarketInputField::reference())
}
