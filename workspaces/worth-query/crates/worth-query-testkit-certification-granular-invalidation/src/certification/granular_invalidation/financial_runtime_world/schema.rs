#[path = "schema/records.rs"]
mod storage_model;
pub use storage_model::*;
pub use worth_query_host::facade::declaration::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
    ApplicationQueryLaneEligibility, ApplicationQueryResultFieldRef,
    ApplicationQueryResultShapeBuilder,
};
pub use worth_query_host::facade::{declaration, primary_graph};
pub use worth_query_host::facade::{
    worth_query_application_query, worth_query_operation, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_portable_type, worth_query_principal_binding,
};

use worth_query_host::facade::declaration::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct FinancialHostSchema;
impl Fragment for FinancialHostSchema {
    const NAME: &'static str = "FinancialHostSchema";
    type Bindings = NoBindings;
    type Members = (FinancialModel, Contracts);
}
impl ApplicationSchema for FinancialHostSchema {
    const OWNER: &'static str = "granular_financial_courtroom";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl FinancialHostSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct Contracts;
impl Registration<FinancialHostSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<FinancialHostSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<FinancialHostSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .principal_binding(FinancialPrincipalBinding::reference())
            .operation(
                ExecuteFinancial::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation(
                AmendMarket::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(ExecuteFinancial::reference(), 16)
            .operation_read_field(
                ExecuteFinancial::reference(),
                MarketIdentityField::at("MarketIdentityField"),
            )
            .operation_read_field(
                ExecuteFinancial::reference(),
                MarketRevisionField::at("MarketRevisionField"),
            )
            .operation_read_field(
                ExecuteFinancial::reference(),
                MarketLifecycleField::at("MarketLifecycleField"),
            )
            .operation_read_field(
                ExecuteFinancial::reference(),
                RiskValueField::at("RiskValueField"),
            )
            .operation_write(
                ExecuteFinancial::reference(),
                MarketRevisionField::at("MarketRevisionField"),
            )
            .operation_write(
                ExecuteFinancial::reference(),
                MarketLifecycleField::at("MarketLifecycleField"),
            )
            .operation_write(
                ExecuteFinancial::reference(),
                RiskValueField::at("RiskValueField"),
            )
            .operation_projection_work_budget(AmendMarket::reference(), 16)
            .operation_read_field(
                AmendMarket::reference(),
                MarketRevisionField::at("MarketRevisionField"),
            )
            .operation_read_field(
                AmendMarket::reference(),
                MarketDueField::at("MarketDueField"),
            )
            .operation_read_field(
                AmendMarket::reference(),
                MarketLifecycleField::at("MarketLifecycleField"),
            )
            .operation_read_field(
                AmendMarket::reference(),
                MarketGateField::at("MarketGateField"),
            )
            .operation_read_field(
                AmendMarket::reference(),
                MarketInputField::at("MarketInputField"),
            )
            .operation_read_field(
                AmendMarket::reference(),
                CurveZeroRateField::at("CurveZeroRateField"),
            )
            .operation_read_field(AmendMarket::reference(), QuoteMidField::at("QuoteMidField"))
            .operation_read_field(
                AmendMarket::reference(),
                RiskValueField::at("RiskValueField"),
            )
            .operation_read_field(
                AmendMarket::reference(),
                PortfolioValueField::at("PortfolioValueField"),
            )
            .operation_read_field(
                AmendMarket::reference(),
                PortfolioDeskField::at("PortfolioDeskField"),
            )
            .operation_read_field(
                AmendMarket::reference(),
                PortfolioRankField::at("PortfolioRankField"),
            )
            .operation_write(
                AmendMarket::reference(),
                MarketRevisionField::at("MarketRevisionField"),
            )
            .operation_write(
                AmendMarket::reference(),
                MarketDueField::at("MarketDueField"),
            )
            .operation_write(
                AmendMarket::reference(),
                MarketLifecycleField::at("MarketLifecycleField"),
            )
            .operation_write(
                AmendMarket::reference(),
                MarketGateField::at("MarketGateField"),
            )
            .operation_write(
                AmendMarket::reference(),
                MarketInputField::at("MarketInputField"),
            )
            .operation_write(
                AmendMarket::reference(),
                CurveZeroRateField::at("CurveZeroRateField"),
            )
            .operation_write(AmendMarket::reference(), QuoteMidField::at("QuoteMidField"))
            .operation_write(
                AmendMarket::reference(),
                RiskValueField::at("RiskValueField"),
            )
            .operation_write(
                AmendMarket::reference(),
                PortfolioValueField::at("PortfolioValueField"),
            )
            .operation_write(
                AmendMarket::reference(),
                PortfolioDeskField::at("PortfolioDeskField"),
            )
            .operation_write(
                AmendMarket::reference(),
                PortfolioRankField::at("PortfolioRankField"),
            )
            .application_query(financial_intent_query_definition()))
    }
}

pub struct FinancialPrincipalBinding;
impl FinancialPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        FinancialHostSchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "FinancialPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinancialInput(pub String);
worth_query_portable_type!(FinancialInput => "worth.query.test.certification.financial.input.v1");
worth_query_host::facade::worth_query_structured_value_binding!(pub FinancialInputBinding for FinancialInput { identity: "worth.query.test.certification.financial.input.v1" });

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmendMarketInput;
worth_query_portable_type!(AmendMarketInput => "worth.query.test.certification.financial.amend_input.v1");
worth_query_host::facade::worth_query_structured_value_binding!(pub AmendMarketInputBinding for AmendMarketInput { identity: "worth.query.test.certification.financial.amend_input.v1" });

worth_query_operation!(pub ExecuteFinancial for FinancialHostSchema, input FinancialInputBinding);

worth_query_operation!(pub AmendMarket for FinancialHostSchema, input AmendMarketInputBinding);

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

pub type ResultField<Slot, Field> =
    ApplicationQueryResultFieldRef<FinancialIntentQuery, Slot, FinancialHostSchema, Field>;

pub fn financial_intent_query_definition() -> ApplicationQueryDefinition<
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

pub fn identity_result() -> ResultField<IdentitySlot, MarketIdentityField> {
    ApplicationQueryResultFieldRef::new("identity", MarketIdentityField::at("MarketIdentityField"))
}

pub fn revision_result() -> ResultField<RevisionSlot, MarketRevisionField> {
    ApplicationQueryResultFieldRef::new("revision", MarketRevisionField::at("MarketRevisionField"))
}

pub fn due_result() -> ResultField<DueSlot, MarketDueField> {
    ApplicationQueryResultFieldRef::new("due", MarketDueField::at("MarketDueField"))
}

pub fn lifecycle_result() -> ResultField<LifecycleSlot, MarketLifecycleField> {
    ApplicationQueryResultFieldRef::new(
        "lifecycle",
        MarketLifecycleField::at("MarketLifecycleField"),
    )
}

pub fn input_result() -> ResultField<InputSlot, MarketInputField> {
    ApplicationQueryResultFieldRef::new("input", MarketInputField::at("MarketInputField"))
}
