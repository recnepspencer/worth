//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_host::facade::declaration::application_schema::{fragment::*, *};
pub struct FinancialModel;
impl Fragment for FinancialModel {
    const NAME: &'static str = "financial_model";
    type Bindings = FinancialModelBindings;
    type Members = (ExternalMapping, Principal, MarketObservation);
}
pub struct FinancialModelBindings;
impl BindingSet for FinancialModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<declaration::authentication::WorthQueryExternalPrincipalIdentity>
    for FinancialModelBindings
{
    type Binding = declaration::authentication::WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<declaration::authentication::WorthQueryPrincipalMappingStatus>
    for FinancialModelBindings
{
    type Binding = declaration::authentication::WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for FinancialModelBindings {
    type Binding = declaration::application_schema::U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<String> for FinancialModelBindings {
    type Binding = declaration::application_schema::StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}

#[path = "records/entity_members.rs"]
mod entity_members;
pub use entity_members::{ExternalMapping, MarketObservation, Principal};
#[path = "records/value_rows.rs"]
mod value_rows;
pub use value_rows::{
    AuditFacts, AuditFactsValue, CurveFacts, CurveFactsValue, ExternalIdentity,
    ExternalIdentityValue, MarketIdentity, MarketIdentityValue, PortfolioFacts,
    PortfolioFactsValue, PriceFacts, PriceFactsValue, PrincipalFacts, PrincipalFactsValue,
    RiskFacts, RiskFactsValue, VolatilityFacts, VolatilityFactsValue,
};
#[path = "records/selections.rs"]
mod selections;
pub use selections::{
    AuditLabelField, CurveDetailField, CurvePartitionField, CurveZeroRateField,
    ExternalIdentityField, MappingStatusField, MarketDueField, MarketGateField,
    MarketIdentityField, MarketInputField, MarketLifecycleField, MarketRevisionField,
    PortfolioDeskField, PortfolioRankField, PortfolioValueField, PrincipalIdentityField,
    QuoteMidField, RiskValueField, VolatilitySurfaceField,
};
#[path = "records/relations.rs"]
mod relations;
pub use relations::MappingTarget;
